# FreokRO sprite artwork

The artwork in this directory was supplied or generated for FreokRO without using Gravity sprite files as visual inputs. `portal.png`, `guild-flag.png`, `taming-sigil.png`, `skill-placeholder.png`, and `female-placeholder.png` are game-size previews.

Run `tools/build_freokro_sprites.py` and `tools/build_freokro_placeholders.py` with Pillow to create the matching `.spr` and `.act` files under `korangar/archive/data/sprite`. The generated files are also checked into the project, so players do not need Python or Pillow.

| Legacy identifier | FreokRO asset | Purpose |
| --- | --- | --- |
| `WARPNPC`, `HIDDEN_WARP_NPC` | `FREOKRO_PORTAL` | Visible portal NPC |
| `GUILD_FLAG` | `FREOKRO_FLAG` | Guild banner |
| `SA_TAMINGMONSTER` | `FREOKRO_TAMING` | Taming effect |
| All female player jobs and hair styles | `FREOKRO_FEMALE` | Temporary frontal character animation |
| All skill icons | `FREOKRO_SKILL_PLACEHOLDER` | Temporary common skill icon; skill IDs and behavior stay intact |
| Client mouse cursor | `FREOKRO_CURSOR` | Emerald animated idle, click, attack, dialog, and prohibited states |

The client resolves the first three legacy identifiers in `korangar/src/loaders/mod.rs`. Female player routing is in `korangar/src/world/entity/mod.rs`, skill icon routing is in `korangar/src/loaders/async/mod.rs`, and cursor routing is in `korangar/src/interface/cursor/mod.rs`. The generic `npc/missing.spr` and `npc/missing.act` remain the fallback for other unavailable sprites. All six FreokRO sprite sets passed the game's SPR/ACT parser test. The user confirmed the cursor, female character, and common skill icon in the client on October 9, 2026.

## Temporary animation scope

The cursor uses the project emerald artwork. The idle state pulses through ACT opacity and scale frames; click, attack, dialog, and prohibited states use separate sprites. The female sprite uses the 30 front-facing frames from the supplied GIF when idle. Every camera direction repeats that frontal artwork, and all non-idle actions repeat its first frontal frame. No side, rear, or walk cycle has been authored yet. The source GIF had an opaque, near-black background; the builder removes only border-connected dark pixels before resizing the frames.

The common skill icon is a temporary visual placeholder. It does not replace skill names, levels, descriptions, requirements, or casting behavior. Each skill can receive its own original icon later.
