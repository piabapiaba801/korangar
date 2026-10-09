"""Author original FreokRO RSW, GND and GAT files from a hand-written map specification.

The input is a FreokRO design file. No existing map, GRF or map cache is read.
The generated files are a navigable baseline for further BrowEdit3 art direction.
"""

from __future__ import annotations

import argparse
import json
import re
import struct
from pathlib import Path

TEXTURES = (
    "freokro_ground_base.png",
    "freokro_ground_light.png",
    "freokro_ground_dark.png",
    "freokro_ground_transition.png",
)
NAME = re.compile(r"^[a-z0-9_]{1,12}$")


def fixed(value: str, width: int) -> bytes:
    encoded = value.encode("ascii")
    if len(encoded) > width:
        raise ValueError(f"Value exceeds {width} bytes: {value}")
    return encoded.ljust(width, b"\0")


def validate(spec: dict) -> tuple[str, int, int, int, int, int]:
    name = spec["name"]
    width, height = int(spec["gat_width"]), int(spec["gat_height"])
    cx, cy = map(int, spec["plaza_center"])
    road = int(spec["road_half_width"])
    if not NAME.fullmatch(name):
        raise ValueError(f"Invalid map name: {name!r}")
    if not (32 <= width <= 800 and 32 <= height <= 800 and width % 2 == 0 and height % 2 == 0):
        raise ValueError("GAT dimensions must be even and between 32 and 800")
    if not (0 <= cx < width and 0 <= cy < height and 4 <= road <= 40):
        raise ValueError("Plaza or road parameters are outside the map")
    return name, width, height, cx, cy, road


def write_rsw(path: Path, name: str, width: int, height: int) -> None:
    with path.open("wb") as file:
        file.write(b"GRSW" + bytes((1, 9)))
        for value in ("", f"{name}.gnd", f"{name}.gat", ""):
            file.write(fixed(value, 40))
        file.write(struct.pack("<fifffI", -1000.0, 0, 0.0, 0.0, 0.0, 100))
        file.write(struct.pack("<ii7f", 45, 45, 1.0, 1.0, 1.0, 0.8, 0.8, 0.8, 0.5))
        file.write(struct.pack("<4iI", height, 0, 0, width, 0))


def write_gat(path: Path, width: int, height: int) -> None:
    # Every cell is walkable during initial design. BrowEdit3 can paint walls
    # and obstacles into this original GAT after playtesting NPC positions.
    walkable = struct.pack("<4fB3x", 0.0, 0.0, 0.0, 0.0, 0)
    with path.open("wb") as file:
        file.write(b"GRAT" + bytes((1, 2)) + struct.pack("<ii", width, height))
        for _ in range(height):
            file.write(walkable * width)


def texture_for_tile(x: int, y: int, cx: int, cy: int, road: int) -> int:
    gx, gy = x * 2, y * 2
    dx, dy = abs(gx - cx), abs(gy - cy)
    plaza = (dx / 58.0) ** 2 + (dy / 48.0) ** 2
    if plaza <= 1.0:
        return 1 if (x * 7 + y * 11) % 13 else 3
    if dx <= road or dy <= road:
        return 1 if (x * 13 + y * 3) % 9 else 3
    if dx <= road + 6 or dy <= road + 6:
        return 2
    noise = (x * 73856093 ^ y * 19349663) & 255
    return 0 if noise < 222 else 3 if noise < 247 else 2


def write_gnd(path: Path, width: int, height: int, cx: int, cy: int, road: int) -> None:
    gw, gh = width // 2, height // 2
    with path.open("wb") as file:
        file.write(b"GRGN" + bytes((1, 7)))
        file.write(struct.pack("<iifi", gw, gh, 1.0, len(TEXTURES)))
        # BrowEdit3 stores both a 40-byte filename and a 40-byte display name
        # for each texture. Korangar uses this field as one 80-byte string and
        # stops at the first NUL, so the same file works in both readers.
        file.write(struct.pack("<i", 80))
        for texture in TEXTURES:
            file.write(fixed(texture, 40))
            file.write(fixed(texture, 40))
        # A neutral lightmap keeps BrowEdit3's renderer and tile references
        # valid before any authored lighting is added in the editor.
        file.write(struct.pack("<4i", 1, 8, 8, 1))
        file.write(bytes([255] * 64 + [0] * 192))
        file.write(struct.pack("<i", len(TEXTURES)))
        for index in range(len(TEXTURES)):
            file.write(struct.pack("<8fhh4B", 0, 1, 0, 1, 1, 1, 0, 0, index, 0, 255, 255, 255, 255))
        for y in range(gh):
            row = bytearray()
            for x in range(gw):
                texture = texture_for_tile(x, y, cx, cy, road)
                row.extend(struct.pack("<4f3i", 0, 0, 0, 0, texture, -1, -1))
            file.write(row)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True, help="FreokRO authored JSON design")
    parser.add_argument("--output", type=Path, required=True, help="Output data directory")
    args = parser.parse_args()
    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    name, width, height, cx, cy, road = validate(spec)
    args.output.mkdir(parents=True, exist_ok=True)
    paths = {suffix: args.output / f"{name}.{suffix}" for suffix in ("rsw", "gnd", "gat")}
    existing = [str(path) for path in paths.values() if path.exists()]
    if existing:
        raise SystemExit(f"Refusing to overwrite existing maps: {existing}")
    write_rsw(paths["rsw"], name, width, height)
    write_gnd(paths["gnd"], width, height, cx, cy, road)
    write_gat(paths["gat"], width, height)
    print(json.dumps({"name": name, "gat": [width, height], "source": "FreokRO authored design", "files": {key: str(path) for key, path in paths.items()}}))


if __name__ == "__main__":
    main()
