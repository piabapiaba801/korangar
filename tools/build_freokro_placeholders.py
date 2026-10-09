"""Build FreokRO cursor, skill, and female-character SPR/ACT placeholders.

Only FreokRO artwork is read. Pillow is required to rebuild; the game loads the
generated files without Python or the source images.
"""

from collections import deque
from pathlib import Path
import struct

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art" / "freokro" / "sprites"
SPRITES = ROOT / "korangar" / "archive" / "data" / "sprite"


def fit(source: Path, size: tuple[int, int], *, resample=Image.Resampling.LANCZOS) -> Image.Image:
    image = Image.open(source).convert("RGBA")
    bounds = image.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError(f"Empty artwork: {source}")
    image = image.crop(bounds)
    image.thumbnail((size[0] - 2, size[1] - 2), resample)
    canvas = Image.new("RGBA", size)
    canvas.alpha_composite(image, ((size[0] - image.width) // 2, (size[1] - image.height) // 2))
    return canvas


def remove_gif_background(image: Image.Image) -> Image.Image:
    """Remove the near-black, border-connected GIF background only."""
    image = image.convert("RGBA")
    pixels = image.load()
    width, height = image.size
    pending = deque()
    for x in range(width):
        pending.extend(((x, 0), (x, height - 1)))
    for y in range(height):
        pending.extend(((0, y), (width - 1, y)))
    outside = set()
    while pending:
        x, y = pending.popleft()
        if x < 0 or x >= width or y < 0 or y >= height or (x, y) in outside:
            continue
        red, green, blue, alpha = pixels[x, y]
        if alpha and max(red, green, blue) > 28:
            continue
        outside.add((x, y))
        pending.extend(((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)))
    for x, y in outside:
        pixels[x, y] = (0, 0, 0, 0)
    return image


def female_frames() -> list[Image.Image]:
    source = Image.open(ART / "female-placeholder-source.gif")
    frames = []
    for index in range(source.n_frames):
        source.seek(index)
        image = remove_gif_background(source.copy())
        # A fixed crop keeps the character's feet at one point during the loop.
        image = image.crop((32, 0, 160, 184)).resize((64, 92), Image.Resampling.NEAREST)
        canvas = Image.new("RGBA", (72, 96))
        canvas.alpha_composite(image, (4, 2))
        frames.append(canvas)
    frames[0].save(ART / "female-placeholder.png")
    return frames


def make_spr(frames: list[Image.Image]) -> bytes:
    width, height = frames[0].size
    if len(frames) > 65535 or any(frame.size != (width, height) for frame in frames):
        raise ValueError("SPR frames must have one size and fit a 16-bit count")
    output = bytearray(b"SP" + bytes((0, 2)))
    output.extend(struct.pack("<HH", 0, len(frames)))
    for frame in frames:
        output.extend(struct.pack("<HH", width, height))
        pixels = frame.load()
        for y in range(height - 1, -1, -1):
            for x in range(width):
                red, green, blue, alpha = pixels[x, y]
                output.extend((alpha, blue, green, red))
    output.extend(bytes(256 * 4))
    return bytes(output)


def make_act(actions: list[list[tuple[int, int, float]]], delay: float, offset=(0, 0)) -> bytes:
    """Each motion is (sprite index, alpha, zoom)."""
    output = bytearray(b"AC" + bytes((2, 2)))
    output.extend(struct.pack("<H", len(actions)))
    output.extend(bytes(10))
    for motions in actions:
        output.extend(struct.pack("<I", len(motions)))
        for sprite_index, alpha, zoom in motions:
            output.extend(bytes(32))
            output.extend(struct.pack("<I", 1))
            output.extend(struct.pack("<ii", *offset))
            output.extend(struct.pack("<i", sprite_index))
            output.extend(struct.pack("<I", 0))
            output.extend(struct.pack("<I", (alpha << 24) | 0xFFFFFF))
            output.extend(struct.pack("<f", zoom))
            output.extend(struct.pack("<i", 0))
            output.extend(struct.pack("<I", 1))
            output.extend(struct.pack("<i", -1))
    output.extend(struct.pack("<I", 0))
    output.extend(struct.pack(f"<{len(actions)}f", *([delay] * len(actions))))
    return bytes(output)


def save(subdir: str, stem: str, frames: list[Image.Image], actions, delay: float, offset=(0, 0)) -> None:
    target = SPRITES / subdir
    target.mkdir(parents=True, exist_ok=True)
    (target / f"{stem}.spr").write_bytes(make_spr(frames))
    (target / f"{stem}.act").write_bytes(make_act(actions, delay, offset))
    print(f"{stem}: {len(frames)} sprites, {len(actions)} actions")


def main() -> None:
    cursor_names = (
        "cursor-source.png",
        "cursor-click-concept.png",
        "cursor-attack-concept.png",
        "cursor-dialog-concept.png",
        "cursor-prohibited-source.png",
    )
    cursor_frames = [fit(ART / name, (48, 48)) for name in cursor_names]
    cursor_actions = []
    for action in range(14):
        sprite_index = {1: 3, 2: 1, 5: 2, 6: 2, 8: 4, 9: 1}.get(action, 0)
        for _direction in range(8):
            if action == 0:  # Breathing emerald glow while idle.
                motions = [(sprite_index, alpha, zoom) for alpha, zoom in ((215, 0.96), (235, 1.0), (255, 1.04), (235, 1.0))]
            elif action in (2, 9):
                motions = [(sprite_index, 255, zoom) for zoom in (1.0, 0.91, 1.0)]
            else:
                motions = [(sprite_index, 255, 1.0)]
            cursor_actions.append(motions)
    save("", "FREOKRO_CURSOR", cursor_frames, cursor_actions, 2.0, (24, 24))

    skill = fit(ART / "skill-placeholder-source.png", (32, 32))
    skill.save(ART / "skill-placeholder.png")
    save("아이템", "FREOKRO_SKILL_PLACEHOLDER", [skill], [[(0, 255, 1.0)] for _ in range(8)], 100.0)

    frames = female_frames()
    # Only the original front-facing animation is used; the other actions and
    # all camera directions reuse its first frontal frame until authored later.
    actions = [[(frame, 255, 1.0) for frame in range(len(frames))] if action < 8 else [(0, 255, 1.0)] for action in range(13 * 8)]
    save("npc", "FREOKRO_FEMALE", frames, actions, 2.0)


if __name__ == "__main__":
    main()
