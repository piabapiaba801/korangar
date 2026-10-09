"""Export static warp locations from the configured FreokRO rAthena scripts.

Only map IDs and source tile coordinates are exported. Run this again after
changing the server's NPC script configuration or warp definitions.
"""

import argparse
import csv
import re
from pathlib import Path


DIRECTIVE = re.compile(r"^(import|npc):\s*(\S+)", re.IGNORECASE)
WARP = re.compile(
    r"^([A-Za-z0-9_@-]+),(\d+),(\d+)(?:,\d+)?\s+warp\s+\S+\s+\d+,\d+,\S+,\d+,\d+",
    re.IGNORECASE,
)


def active_scripts(server_root: Path) -> list[Path]:
    pending = [server_root / "npc/re/scripts_main.conf"]
    visited = set()
    scripts = []

    while pending:
        config = pending.pop()
        config = config.resolve()
        if config in visited:
            continue
        if not config.is_relative_to(server_root) or not config.is_file():
            raise ValueError(f"Configured script file is missing or outside server root: {config}")
        visited.add(config)
        for raw_line in config.read_text(encoding="utf-8-sig", errors="replace").splitlines():
            line = raw_line.split("//", 1)[0].strip()
            match = DIRECTIVE.match(line)
            if match is None:
                continue
            target = (server_root / match.group(2)).resolve()
            if not target.is_relative_to(server_root) or not target.is_file():
                raise ValueError(f"Configured script file is missing or outside server root: {target}")
            if match.group(1).lower() == "import":
                pending.append(target)
            else:
                scripts.append(target)

    return sorted(set(scripts))


def map_dimensions(catalog: Path) -> dict[str, tuple[int, int]]:
    with catalog.open(newline="", encoding="utf-8") as source:
        return {
            row["map_id"].lower(): (int(row["gat_width"]), int(row["gat_height"]))
            for row in csv.DictReader(source)
        }


def export(server_root: Path, catalog: Path, output: Path) -> tuple[int, int]:
    dimensions = map_dimensions(catalog)
    locations = set()
    scripts = active_scripts(server_root)
    for script in scripts:
        for raw_line in script.read_text(encoding="utf-8-sig", errors="replace").splitlines():
            line = raw_line.split("//", 1)[0].strip()
            match = WARP.match(line)
            if match is None:
                continue
            map_name, x_text, y_text = match.groups()
            map_name = map_name.lower()
            size = dimensions.get(map_name)
            if size is None:
                continue
            x, y = int(x_text), int(y_text)
            if 0 <= x < size[0] and 0 <= y < size[1]:
                locations.add((map_name, x, y))

    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", newline="", encoding="utf-8") as target:
        target.write("# Static warp source tiles from the configured FreokRO Renewal scripts.\n")
        target.write("# Regenerate with tools/freokro_export_portals.py after server script changes.\n")
        writer = csv.writer(target, lineterminator="\n")
        writer.writerow(("map_id", "x", "y"))
        writer.writerows(sorted(locations))
    return len(scripts), len(locations)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("server_root", type=Path, help="Path to the matching FreokRO rAthena checkout")
    parser.add_argument(
        "--catalog",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "map-authoring/dimensions.csv",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "korangar/src/world/map/freokro_portals.csv",
    )
    args = parser.parse_args()
    server_root = args.server_root.resolve()
    scripts, locations = export(server_root, args.catalog, args.output)
    print(f"Exported {locations} distinct portal tiles from {scripts} configured scripts to {args.output}")


if __name__ == "__main__":
    main()
