# FreokRO sprite artwork

The artwork in this directory was supplied or generated for FreokRO without using Gravity sprite files as visual inputs. `portal.png`, `guild-flag.png`, `taming-sigil.png`, `skill-placeholder.png`, and `female-placeholder.png` are game-size previews.

Run `tools/build_freokro_sprites.py` and `tools/build_freokro_placeholders.py` with Pillow to create the matching `.spr` and `.act` files under `korangar/archive/data/sprite`. The generated files are also checked into the project, so players do not need Python or Pillow.

| Legacy identifier | FreokRO asset | Purpose |
| --- | --- | --- |
| `WARPNPC`, `HIDDEN_WARP_NPC` | `FREOKRO_PORTAL` | Visible portal NPC |
| `GUILD_FLAG` | `FREOKRO_FLAG` | Guild banner |
| `SA_TAMINGMONSTER` | `FREOKRO_TAMING` | Taming effect |
| All female player jobs and hair styles | `FREOKRO_FEMALE` | Turquoise frontal idle and east/west walk placeholders |
| All male player jobs and hair styles | `FREOKRO_MALE` | Temporary project-owned frontal character placeholder |
| All skill icons | `FREOKRO_SKILL_PLACEHOLDER` | Temporary common skill icon; skill IDs and behavior stay intact |
| Client mouse cursor | `FREOKRO_CURSOR` | Emerald animated idle, click, attack, dialog, and prohibited states |

The client resolves the first three legacy identifiers in `korangar/src/loaders/mod.rs`. Player routing is in `korangar/src/world/entity/mod.rs`, skill icon routing is in `korangar/src/loaders/async/mod.rs`, and cursor routing is in `korangar/src/interface/cursor/mod.rs`. The generic `npc/missing.spr` and `npc/missing.act` remain the fallback for other unavailable sprites. The user confirmed the earlier cursor, frontal female character, and common skill icon in the client on October 9, 2026. The new lateral and male sprites still need visual validation.

## Temporary animation scope

The cursor uses the project emerald artwork. The idle state pulses through ACT opacity and scale frames; click, attack, dialog, and prohibited states use separate sprites. The female sprite uses 30 front-facing frames from the supplied GIF for frontal idle and 30 right-facing walk frames from `female-side-walk-source.png`; the left walk mirrors that sheet. Rear and diagonal views still use the frontal placeholder. The builder removes near-black border-connected backgrounds before resizing. The male sprite uses the FreokRO male menu illustration as one temporary frame for every action and direction. Job, hair, equipment, and color variants still need authored art. Character selection uses these same in-game sprite resources for occupied slots.

The common skill icon is a temporary visual placeholder. It does not replace skill names, levels, descriptions, requirements, or casting behavior. Each skill can receive its own original icon later.
