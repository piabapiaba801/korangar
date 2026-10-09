"""Build flat, walkable FreokRO maps from verified dimensions and original art."""

from __future__ import annotations

import argparse
import csv
import json
import os
import struct
import tempfile
from pathlib import Path

from freokro_generate_maps import validate, write_gat, write_gnd, write_rsw


def verify_existing(directory: Path, name: str, width: int, height: int) -> bool:
    files = [directory / f"{name}.{suffix}" for suffix in ("rsw", "gnd", "gat")]
    present = [path.exists() for path in files]
    if not any(present):
        return False
    if not all(present):
        raise ValueError(f"Partial map already exists: {name}")
    rsw, gnd, gat = (path.read_bytes() for path in files)
    if (
        len(rsw) != 246
        or rsw[:6] != b"GRSW\x01\x09"
        or struct.unpack_from("<f", rsw, 166)[0] != 1_000_000.0
        or struct.unpack_from("<I", rsw, 242)[0] != 0
    ):
        raise ValueError(f"Existing RSW is not the authored flat template: {name}")
    if (
        gnd[:6] != b"GRGN\x01\x07"
        or struct.unpack_from("<ii", gnd, 6) != (width // 2, height // 2)
        or struct.unpack_from("<ii", gnd, 18) != (4, 80)
        or len(gnd) != 782 + (width // 2) * (height // 2) * 28
    ):
        raise ValueError(f"Existing GND has different dimensions or format: {name}")
    for southwest, southeast, northwest, northeast, top, north, east in struct.iter_unpack("<4f3i", gnd[782:]):
        if (southwest, southeast, northwest, northeast) != (0.0, 0.0, 0.0, 0.0) or top not in range(4) or (north, east) != (-1, -1):
            raise ValueError(f"Existing GND contains relief or wall surfaces: {name}")
    if (
        gat[:6] != b"GRAT\x01\x02"
        or struct.unpack_from("<ii", gat, 6) != (width, height)
        or len(gat) != 14 + width * height * 20
        or gat[14:].strip(b"\0")
    ):
        raise ValueError(f"Existing GAT is not fully flat and walkable: {name}")
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dimensions", type=Path, required=True)
    parser.add_argument("--spec-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--limit", type=int, default=0, help="Maximum new maps to generate; 0 means all")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.limit < 0:
        raise ValueError("--limit cannot be negative")

    with args.dimensions.open(encoding="utf-8", newline="") as file:
        rows = list(csv.DictReader(file))
    args.output.mkdir(parents=True, exist_ok=True)
    generated = skipped = 0
    for row in rows:
        name = row["map_id"]
        width, height = int(row["gat_width"]), int(row["gat_height"])
        override = args.spec_dir / f"{name}.json"
        spec = json.loads(override.read_text(encoding="utf-8")) if override.exists() else {
            "name": name,
            "gat_width": width,
            "gat_height": height,
            "plaza_center": [width // 2, height // 2],
            "road_half_width": 8,
        }
        actual_name, actual_width, actual_height, cx, cy, road, anchors = validate(spec)
        if (actual_name, actual_width, actual_height) != (name, width, height):
            raise ValueError(f"Design dimensions disagree with the verified catalog: {name}")
        if verify_existing(args.output, name, width, height):
            skipped += 1
            continue
        if args.limit and generated >= args.limit:
            break
        if not args.dry_run:
            with tempfile.TemporaryDirectory(prefix="freokro-map-") as temp_name:
                temp = Path(temp_name)
                write_rsw(temp / f"{name}.rsw", name, width, height)
                write_gnd(temp / f"{name}.gnd", width, height, cx, cy, road, anchors)
                write_gat(temp / f"{name}.gat", width, height)
                for suffix in ("rsw", "gnd", "gat"):
                    os.replace(temp / f"{name}.{suffix}", args.output / f"{name}.{suffix}")
        generated += 1
        if generated % 50 == 0:
            print(f"Generated {generated} maps; skipped {skipped} existing maps", flush=True)
    print(f"Finished: generated={generated}, skipped={skipped}, catalog={len(rows)}, dry_run={args.dry_run}")


if __name__ == "__main__":
    main()
