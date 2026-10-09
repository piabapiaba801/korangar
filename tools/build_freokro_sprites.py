"""Build FreokRO-owned static SPR/ACT sprites from transparent PNG artwork.

Requires Pillow at build time. The game only needs the generated SPR/ACT files.
"""

from pathlib import Path
import struct

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art" / "freokro" / "sprites"
SPRITES = ROOT / "korangar" / "archive" / "data" / "sprite"

ASSETS = (
    ("portal-source.png", "portal.png", "npc", "FREOKRO_PORTAL", (96, 128)),
    ("guild-flag-source.png", "guild-flag.png", "npc", "FREOKRO_FLAG", (80, 120)),
    ("taming-sigil-source.png", "taming-sigil.png", "아이템", "FREOKRO_TAMING", (64, 64)),
)


def prepare_image(source: Path, size: tuple[int, int]) -> Image.Image:
    image = Image.open(source).convert("RGBA")
    alpha = image.getchannel("A").point(lambda value: 255 if value > 4 else 0)
    bounds = alpha.getbbox()
    if bounds is None:
        raise ValueError(f"Image is fully transparent: {source}")
    image = image.crop(bounds)
    image.thumbnail((size[0] - 4, size[1] - 4), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", size)
    position = ((size[0] - image.width) // 2, (size[1] - image.height) // 2)
    canvas.alpha_composite(image, position)
    return canvas


def make_spr(image: Image.Image) -> bytes:
    width, height = image.size
    if not (0 < width <= 65535 and 0 < height <= 65535):
        raise ValueError("Sprite dimensions exceed the SPR format")
    # SPR 2.0: no indexed images, one ABGR image, empty palette.
    output = bytearray(b"SP" + bytes((0, 2)))
    output.extend(struct.pack("<HHHH", 0, 1, width, height))
    pixels = image.load()
    for y in range(height - 1, -1, -1):
        for x in range(width):
            red, green, blue, alpha = pixels[x, y]
            output.extend((alpha, blue, green, red))
    output.extend(bytes(256 * 4))
    return bytes(output)


def make_act() -> bytes:
    # ACT 2.2: eight static directions, one frame each, one RGBA sprite clip.
    output = bytearray(b"AC" + bytes((2, 2)))
    output.extend(struct.pack("<H", 8))
    output.extend(bytes(10))
    for _direction in range(8):
        output.extend(struct.pack("<I", 1))  # one motion
        output.extend(bytes(32))  # two unused range rectangles
        output.extend(struct.pack("<I", 1))  # one clip
        output.extend(struct.pack("<ii", 0, 0))  # centered
        output.extend(struct.pack("<i", 0))  # first sprite
        output.extend(struct.pack("<I", 0))  # no mirror
        output.extend(struct.pack("<I", 0xFFFFFFFF))  # opaque white tint
        output.extend(struct.pack("<f", 1.0))  # original size
        output.extend(struct.pack("<i", 0))  # no rotation
        output.extend(struct.pack("<I", 1))  # RGBA sprite image
        output.extend(struct.pack("<i", -1))  # no event
    output.extend(struct.pack("<I", 0))  # no sound/action events
    output.extend(struct.pack("<8f", *([100.0] * 8)))
    return bytes(output)


def main() -> None:
    for source_name, preview_name, subdir, stem, size in ASSETS:
        image = prepare_image(ART / source_name, size)
        image.save(ART / preview_name)
        target = SPRITES / subdir
        target.mkdir(parents=True, exist_ok=True)
        (target / f"{stem}.spr").write_bytes(make_spr(image))
        (target / f"{stem}.act").write_bytes(make_act())
        print(f"{stem}: {image.width}x{image.height}")


if __name__ == "__main__":
    main()
