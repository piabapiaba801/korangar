"""Package authored FreokRO maps for Korangar as a non-solid 7z archive."""

from __future__ import annotations

import argparse
import csv
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


TEXTURES = (
    "freokro_ground_base.png",
    "freokro_ground_dark.png",
    "freokro_ground_light.png",
    "freokro_ground_transition.png",
)


def run(sevenzip: str, arguments: list[str], directory: Path) -> str:
    result = subprocess.run(
        [sevenzip, *arguments], cwd=directory, check=True, capture_output=True, text=True
    )
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dimensions", type=Path, required=True)
    parser.add_argument("--archive-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--sevenzip", help="Path to 7z or 7zz; auto-detected when omitted")
    args = parser.parse_args()

    sevenzip = args.sevenzip or shutil.which("7zz") or shutil.which("7z")
    if not sevenzip:
        raise RuntimeError("7z or 7zz is required to package the maps")
    root = args.archive_root.resolve()
    output = args.output.resolve()
    with args.dimensions.open(encoding="utf-8", newline="") as catalog:
        names = [row["map_id"] for row in csv.DictReader(catalog)]
    if len(names) != len(set(names)):
        raise ValueError("Dimension catalog contains duplicate map names")

    relative_paths = [Path("data") / f"{name}.{suffix}" for name in names for suffix in ("rsw", "gnd", "gat")]
    relative_paths.extend(Path("data") / "texture" / name for name in TEXTURES)
    missing = [str(path) for path in relative_paths if not (root / path).is_file()]
    if missing:
        raise FileNotFoundError(f"Missing authored assets: {missing[:8]}")

    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="freokro-package-", dir=output.parent) as temp_name:
        temp = Path(temp_name)
        file_list = temp / "files.txt"
        package = temp / "freokro-maps.7z"
        file_list.write_text("\n".join(map(str, relative_paths)) + "\n", encoding="utf-8")
        print(f"Packaging {len(names)} maps and {len(TEXTURES)} textures", flush=True)
        run(sevenzip, ["a", "-t7z", "-mx=5", "-ms=off", "-scsUTF-8", str(package), f"@{file_list}"], root)

        listing = run(sevenzip, ["l", "-slt", str(package)], root)
        paths = [line[7:] for line in listing.splitlines() if line.startswith(("Path = data/", "Path = data\\"))]
        if "Solid = -" not in listing or len(paths) != len(relative_paths):
            raise ValueError("Package is solid or has the wrong number of entries")
        if set(path.replace("\\", "/") for path in paths) != set(path.as_posix() for path in relative_paths):
            raise ValueError("Package entries differ from the authored map catalog")
        if "Everything is Ok" not in run(sevenzip, ["t", str(package)], root):
            raise ValueError("7z integrity check failed")
        os.replace(package, output)
    print(f"Verified non-solid package: {output} ({output.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
