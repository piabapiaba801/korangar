"""Create a history-free local candidate for the separate FreokRO public base.

This intentionally does not publish or configure a remote. It copies tracked
source and project-generated maps, excluding review-only material.
"""

import argparse
import os
from pathlib import Path
import shutil
import subprocess


SOURCE = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = SOURCE.parents[1] / "freokro-base"
EXCLUDED_FILES = {
    "README.md",
    "TECHNICAL_STATUS_DEV.md",
    "TECHNICAL_STATUS_FREOKRO.md",
    "PUBLIC_BASE_RELEASE.md",
    "korangar/archive/data/sclientinfo.xml",
    "art/freokro/character-menu/female-sheet-source.png",
    "tools/export_freokro_base.py",
}
EXCLUDED_PREFIXES = (".github/", "docs/screenshots/", "reports/", "release/")


def included(relative: Path) -> bool:
    name = relative.as_posix()
    if name in EXCLUDED_FILES or name.startswith(EXCLUDED_PREFIXES):
        return False
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    output = args.output.resolve()
    if output == SOURCE or SOURCE in output.parents:
        raise ValueError("The public base must be outside the source checkout")
    if output.exists() and any(output.iterdir()):
        raise FileExistsError(f"Output is not empty: {output}")

    names = subprocess.check_output(["git", "ls-files", "-z"], cwd=SOURCE).split(b"\0")
    selected = [Path(os.fsdecode(name)) for name in names if name and included(Path(os.fsdecode(name)))]
    output.mkdir(parents=True, exist_ok=True)
    copied_bytes = 0
    for relative in selected:
        source = SOURCE / relative
        target = output / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        copied_bytes += source.stat().st_size

    shutil.copy2(SOURCE / "release" / "FREOKRO_BASE_README.md", output / "README.md")
    gitignore = output / ".gitignore"
    ignore_text = gitignore.read_text(encoding="utf-8")
    ignore_text = ignore_text.replace("korangar/client/\n", "korangar/client/*\n!korangar/client/branding/\n")
    gitignore.write_text(ignore_text, encoding="utf-8")
    print(f"Prepared {len(selected)} files ({copied_bytes / 1024 / 1024:.1f} MiB) in {output}")
    print("No Git history, remote, credentials, or user screenshots were copied.")


if __name__ == "__main__":
    main()
