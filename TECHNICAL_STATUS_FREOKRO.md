# FreokRO technical status — October 9, 2026

## Current repository state

- Active development and publication branch: `freokro` in the [FreokRO Korangar fork](https://github.com/piabapiaba801/korangar/tree/freokro). The existing `main`, `dev`, and `rebase` branches remain separate.
- The preserved client work before map migration is documented in [TECHNICAL_STATUS_DEV.md](TECHNICAL_STATUS_DEV.md). The server is in the [FreokRO rAthena repository](https://github.com/piabapiaba801/freokro-rathena-korangar); the Auction HUD is a separate private project.
- `5573be9b` introduced the FreokRO emblem and project identity in the README. `cc374ebd` introduced the first map inventory and Prontera terrain pilot.
- `3b5e7e6` replaced the menu map with the FreokRO video. `0cb8ddc` fixed the black screen by skipping directional shadow passes on frames without a map. Together they form the current startup checkpoint on `freokro`.

## Startup identity and map loading

- The client starts with no map loaded. Login and character selection render their own 2D background and interactive windows; Geffen is no longer decoded solely for the menu. The server-provided map still loads when the player enters the world. Geffen remains in the authored active-map set.
- The supplied eight-second H.264 MP4 was converted to a 1280 × 720, 20 fps AV1/IVF file. The runtime reads `client/branding/intro.ivf` and loops it behind the menus. There is no separate logo layer because the FreokRO artwork is already in the video. Missing or invalid video falls back to a dark background.
- The client compiled successfully with `--release --locked --features unicode,debug`, and the rebuilt executable was installed in the local game directory. The first visual launch exposed a black screen and repeated `wgpu` validation errors: the map-free menu still recorded directional shadow passes without shadow partitions. The renderer now skips those passes when there is no map. A 40-second diagnostic launch logged zero validation errors, and the user confirmed that the startup screen passed visually. Menu interaction, character selection, and transition into the world remain to be checked with this build.
- The desktop shortcut `FreokRO (cliente independente)` points to the rebuilt executable in `client/game`. The six temporary startup diagnostic logs in `reports/` are local, untracked, and excluded from publication.

## Map migration: implemented scope

- The original Prontera-only runtime texture override has now been removed. The client reads the newly authored map files directly from the local `freokro-maps.7z` package, which takes precedence over the older archives. The `.rsw`, `.gnd`, and `.gat` drafts cover all 1,265 active maps.
- The [current Prontera screenshot](docs/screenshots/freokro-prontera-current-2026-10-09.jpg) shows the authored ground, the FreokRO portal sprite, temporary character/NPC placeholders, and the basic minimap. The user visually approved the first authored flat-map batch in-game on October 9, then reported clickable places where the character could not walk. The cause was eight higher-priority legacy maps in the rAthena renewal and pre-renewal override caches. Both override files have been cleared, leaving the new main cache as the map source. Movement after a Map-server restart awaits retesting.
- A black, one-GND-tile perimeter now marks the real edge of every active map without blocking its GAT cells. The client builds a map overview from the GND's ground palette and shows the player position; it opens when entering a map and can be toggled with **Alt+M** or the menu. The current screenshot confirms the minimap renders in Prontera; the black perimeter still needs an in-game edge check.
- An earlier pilot capture showed a flag/banner whose source was not identified. It is not visible in the current screenshot; no NPCs were removed merely to hide it.

## Asset audit and deletion status

- The first pass indexed **1,104 `.rsw` maps** and **3,160 distinct ground texture paths** from the extracted local game data. All 1,104 maps have a resolved `.gat`; `air_evt` is the only map without a resolved `.gnd` in that extraction.
- [Map inventory](reports/map_inventory.csv), [ground texture dependencies](reports/asset_dependencies.csv), [baseline](reports/baseline.md), and [migration manifest](reports/migration_manifest.json) are committed. Model references, shared UI consumers, archive overrides, and ownership still require auditing.
- The user physically removed the old `client/game/data-freokro.7z`, the duplicate `client/game/extracted` tree, and `client/game/archive/bgm`. The local archive list now uses only `archive/` and `freokro-maps.7z`. No corresponding deletion was performed by Codex or committed to Git; the deleted items were physical distribution files outside this source repository.
- [The legacy map removal queue](reports/legacy_map_removal_queue.csv) records 1,104 map IDs formerly indexed in `data-freokro.7z`: 1,053 active IDs with authored replacements and 51 outside the active server list. The queue remains historical inventory, not a list of files still installed.
- The project-supplied ground PNGs total **14,382,481 bytes**. They still require repeated-tile seam testing and possible optimization.

## Original map authoring checkpoint

- The first BrowEdit3 opening attempt hung at about 21 GB of memory because the generated GND had 40-byte texture records where BrowEdit3 v3.660 expects an 80-byte filename/display-name pair. The generator was corrected and now includes a neutral lightmap. The latest official BrowEdit3 release is still v3.660; a fresh portable installation was downloaded, checked against the official SHA-256 digest, configured without GRFs, and targeted by the `BrowEdit3 FreokRO` desktop shortcut. Visual inspection with that clean installation remains pending.
- The first isolated server cache test marked every cell as water because of the draft RSW water level. The generator now writes rAthena's no-water sentinel. The validated main cache was installed locally; the two higher-priority caches that masked eight of its maps were subsequently cleared.
- The user then requested a fixed replacement model that preserves each map's original size while removing terrain relief, static scenery (including trees and vegetation), and non-walkable areas. [The dimension catalog](map-authoring/dimensions.csv) records all **1,265 active maps**: 1,053 dimensions from extracted GND/GAT headers and 212 from the server cache header. Where both sources existed, they agreed. No old terrain, model, height, or cell data was copied. The 51 extracted maps absent from the active server list are outside this batch.
- [The generators](map-authoring/README.md) created RSW/GND/GAT files for all 1,265 active maps in `korangar/archive/data`. Each RSW has zero static resources; each GND is flat, uses four FreokRO textures, and has a black perimeter; each GAT preserves its catalog dimensions, has zero height, and is entirely walkable. The four named designs for Prontera and adjoining fields retain active warp coordinates and original dimensions. Prontera remains 312 × 392 GAT and 156 × 196 GND.
- The full batch verifier accepted all **1,265/1,265** active maps after the perimeter update. An isolated rAthena mapcache run cached all 1,265; inspection of its decompressed data found **104,823,864 walkable land cells**, all matching the catalog dimensions. The matching cache is installed in the local server as `db/map_cache.dat` (155,503 bytes); renewal and pre-renewal overrides now contain zero maps. The representative Korangar parser test passed with border checks after this revision. The `unicode,debug` release build passed and its executable is installed locally.
- These files are a structural baseline, not finished art or gameplay navigation. The local game uses a verified **non-solid** 7z package with 3,799 entries and 17,722,761 compressed bytes, listed last in `client/game_archives.ron` for highest priority. The new client executable was built and installed alongside it.
- Targeted rollback files for the previous executable, archive settings, and three server caches are under `C:\Users\Administrator\Desktop\freokro\rollback\map-switch-20261009`. The invalid first solid 7z draft was removed. The user's removal of the old archive and extraction reduced the physical game copy, but no complete distribution-size audit has been run.

## FreokRO sprites and cursor concepts

- New original art and generated static `.spr`/`.act` files cover the portal (`FREOKRO_PORTAL`), guild banner (`FREOKRO_FLAG`), and taming effect (`FREOKRO_TAMING`). The client redirects `WARPNPC`, `HIDDEN_WARP_NPC`, `GUILD_FLAG`, and `SA_TAMINGMONSTER` to those new names. The parser test and `unicode,debug` release build passed; the user reported that the sprite test passed in the client.
- At the latest physical-folder check, six old NPC sprite files (`GUILD_FLAG`, `HIDDEN_WARP_NPC`, and `WARPNPC` `.act`/`.spr` pairs) remained alongside the new assets and the `missing` fallback pair. The two `SA_TAMINGMONSTER` files were no longer present; Codex did not delete or overwrite them. Authorship of the old pairs is unconfirmed. Automatic review blocked Codex's attempted deletion of `GUILD_FLAG.act`. The new test executable resolves the legacy identifiers to the FreokRO files.
- The emerald cursor master and click, attack, and dialog concept PNGs are saved under [sprite artwork](art/freokro/sprites). They are previews only. No cursor animation, prohibited state, or runtime integration was published; development was paused at the user's request.
- Minimap portal markers are implemented and were confirmed in-game by the user on October 9, 2026. [The exporter](tools/freokro_export_portals.py) follows the configured Renewal NPC script imports and records **3,825 distinct in-bounds static warp tiles across 623 active maps** in [the portal catalog](korangar/src/world/map/freokro_portals.csv). The client draws emerald markers at those coordinates and at live warp entities sent by the server, avoiding duplicate markers at the same tile. The current screenshot predates these markers. Regenerate the catalog after changing server scripts; spot checks of additional maps and warp travel remain useful.

## Coverage and Gravity-derived assets

| Measure | Current result |
| --- | --- |
| Active technical map IDs with authored RSW/GND/GAT | **1,265 / 1,265 (100%)** |
| Active authored maps with flat ground, a black border, and all-walkable GAT cells | **1,265 / 1,265 (100%)** |
| Static RSW scenery entries in those authored maps | **0**; trees, vegetation, buildings, lights, sounds, and effects were omitted from the new RSW files |
| Legacy map/audio package in the physical game copy | `data-freokro.7z`, the duplicate extracted tree, and `archive/bgm` were removed by the user; completeness of other loose assets is not verified |
| All Gravity-derived content across sprites, UI, audio, effects, and other game systems | **Not measured**; no defensible overall removal percentage exists |

The 51 extracted map IDs outside the active server list were not regenerated in this batch. NPCs and other entities may still be provided by server scripts and other game resources. The new file formats and map names alone do not eliminate those dependencies.

## Remaining work, in order

1. Restart Map and validate movement in `prt_fild08`, Prontera, and a representative interior after clearing the old cache overrides. Check the new black perimeter and map overview in-game.
2. Open corrected Prontera and representative field maps in the clean BrowEdit3 installation; check appearance, editor stability, and texture seams.
3. Check NPC locations, connected warps, and movement on more maps before treating the 1,265-map batch as playable.
4. Identify remaining Gravity asset consumers before deleting legacy files. Develop distinct FreokRO scenery and gameplay obstacles as the next art and navigation pass.
5. Spot-check portal markers and warp travel on additional maps. Resume the animated emerald cursor only when the user requests it.

The first authored flat-map view and the portal markers were approved by the user. The current screenshot shows the basic minimap and FreokRO portal sprite; the cache fix, black perimeter, movement, and connected warp travel still await focused gameplay checks. The animated cursor remains paused at the user's request.
