"""Package FreokRO-owned character menu art for the game archive.

Requires Pillow only when rebuilding the runtime PNGs. The supplied female
sprite sheet is kept intact under art/freokro/character-menu.
"""

from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art" / "freokro" / "character-menu"
DESTINATION = ROOT / "korangar" / "archive" / "data" / "texture" / "freokro"


def save_preview(image: Image.Image, target: Path, size: tuple[int, int]) -> None:
    image = image.convert("RGBA")
    bounds = image.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError(f"No visible character pixels for {target}")
    image = image.crop(bounds)
    image.thumbnail(size, Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", size)
    canvas.alpha_composite(image, ((size[0] - image.width) // 2, size[1] - image.height))
    canvas.save(target, optimize=True)


def main() -> None:
    DESTINATION.mkdir(parents=True, exist_ok=True)
    forest = Image.open(SOURCE / "forest-scene-source.png").convert("RGB")
    forest.save(DESTINATION / "character-scene.png", optimize=True)

    sheet = Image.open(SOURCE / "female-sheet-source.png")
    if sheet.size != (512, 768):
        raise ValueError("Expected a 4 x 4 sheet of 128 x 192 female frames")
    female_frame = sheet.crop((0, 0, 128, 192)).resize((256, 384), Image.Resampling.NEAREST)
    save_preview(female_frame, DESTINATION / "female-preview.png", (256, 384))

    male = Image.open(SOURCE / "male-preview-source.png")
    save_preview(male, DESTINATION / "male-preview.png", (256, 384))
    print(f"Wrote character menu art to {DESTINATION}")


if __name__ == "__main__":
    main()
