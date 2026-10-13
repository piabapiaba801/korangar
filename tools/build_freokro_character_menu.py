"""Package FreokRO-owned character menu art for the game archive.

Requires Pillow only when rebuilding the runtime PNGs. The supplied female
sprite sheet is kept intact under art/freokro/character-menu.
"""

from pathlib import Path

from PIL import Image
from build_freokro_placeholders import remove_gif_background


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

    # The menu and the world derive from the same turquoise source animation.
    female_source = Image.open(ROOT / "art" / "freokro" / "sprites" / "female-placeholder-source.gif")
    female_source.seek(0)
    female_frame = remove_gif_background(female_source.copy())
    save_preview(female_frame.resize((576, 576), Image.Resampling.NEAREST), DESTINATION / "female-preview.png", (256, 384))

    male = Image.open(SOURCE / "male-preview-source.png")
    save_preview(male, DESTINATION / "male-preview.png", (256, 384))
    print(f"Wrote character menu art to {DESTINATION}")


if __name__ == "__main__":
    main()
