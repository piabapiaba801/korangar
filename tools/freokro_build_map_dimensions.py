"""Record active map dimensions without copying legacy geometry or cell data."""

from __future__ import annotations

import argparse
import csv
import re
import struct
from pathlib import Path

MAP_LINE = re.compile(r"^map:\s*([a-z0-9_@-]{1,12})\s*$", re.MULTILINE)


def read_cache_dimensions(path: Path) -> dict[str, tuple[int, int]]:
    data = path.read_bytes()
    declared_size, count = struct.unpack_from("<IH", data)
    if declared_size != len(data):
        raise ValueError("Map cache size does not match its header")
    dimensions: dict[str, tuple[int, int]] = {}
    offset = 8  # main_header has two padding bytes in this rAthena build
    for _ in range(count):
        raw_name, width, height, compressed_length = struct.unpack_from("<12shhi", data, offset)
        offset += 20 + compressed_length
        if offset > len(data) or width <= 0 or height <= 0:
            raise ValueError("Invalid map cache entry")
        name = raw_name.split(b"\0", 1)[0].decode("ascii")
        dimensions[name] = (width, height)
    if offset != len(data):
        raise ValueError("Map cache has unexpected trailing data")
    return dimensions


def read_inventory_dimensions(path: Path) -> dict[str, tuple[int, int, int, int]]:
    dimensions: dict[str, tuple[int, int, int, int]] = {}
    with path.open(encoding="utf-8-sig", newline="") as file:
        for row in csv.DictReader(file):
            if all(row[key] for key in ("gat_width", "gat_height", "gnd_width", "gnd_height")):
                dimensions[row["map_id"]] = tuple(
                    int(row[key]) for key in ("gat_width", "gat_height", "gnd_width", "gnd_height")
                )
    return dimensions


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--active-maps", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    active = set(MAP_LINE.findall(args.active_maps.read_text(encoding="utf-8", errors="replace")))
    inventory = read_inventory_dimensions(args.inventory)
    cache = read_cache_dimensions(args.cache)
    rows = []
    for name in sorted(active):
        inventoried = inventory.get(name)
        cached = cache.get(name)
        if inventoried:
            width, height, ground_width, ground_height = inventoried
            if cached and cached != (width, height):
                raise ValueError(f"Dimension mismatch for {name}: inventory {width}x{height}, cache {cached}")
            source = "extracted_headers"
        elif cached:
            width, height = cached
            ground_width, ground_height = width // 2, height // 2
            source = "server_cache_header"
        else:
            raise ValueError(f"No verified dimensions for active map {name}")
        if width != 2 * ground_width or height != 2 * ground_height:
            raise ValueError(f"GND/GAT dimension mismatch for {name}")
        if not (32 <= width <= 800 and 32 <= height <= 800):
            raise ValueError(f"Unsupported dimensions for {name}: {width}x{height}")
        rows.append((name, width, height, ground_width, ground_height, source))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as file:
        writer = csv.writer(file)
        writer.writerow(("map_id", "gat_width", "gat_height", "gnd_width", "gnd_height", "dimension_source"))
        writer.writerows(rows)
    print(f"Recorded dimensions for {len(rows)} active maps in {args.output}")


if __name__ == "__main__":
    main()
