# FreokRO sprite artwork

These source PNGs were generated for FreokRO without using Gravity sprite files as visual inputs. `portal.png`, `guild-flag.png`, and `taming-sigil.png` are the reduced game-size previews.

Run `tools/build_freokro_sprites.py` with Pillow to create the matching static `.spr` and `.act` files under `korangar/archive/data/sprite`. The generated files are also checked into the project, so players do not need Python or Pillow.

| Legacy identifier | FreokRO asset | Purpose |
| --- | --- | --- |
| `WARPNPC`, `HIDDEN_WARP_NPC` | `FREOKRO_PORTAL` | Visible portal NPC |
| `GUILD_FLAG` | `FREOKRO_FLAG` | Guild banner |
| `SA_TAMINGMONSTER` | `FREOKRO_TAMING` | Taming effect |

The client resolves these identifiers in `korangar/src/loaders/mod.rs`. The generic `npc/missing.spr` and `npc/missing.act` remain the fallback for all other unavailable sprites. The FreokRO assets passed the game's SPR/ACT parser test, and the user reported that the sprite test passed in the client. Specific portal coverage across maps still needs a broader gameplay check.

## Cursor concepts (paused)

`cursor-source.png`, `cursor-click-concept.png`, `cursor-attack-concept.png`, and `cursor-dialog-concept.png` are artwork previews only. They have **not** been converted into a cursor sprite or connected to the client. Idle glow/motion and the prohibited state remain unimplemented. Cursor development was paused at the user's request.
