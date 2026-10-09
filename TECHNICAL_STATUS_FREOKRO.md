# FreokRO technical status — October 9, 2026

## Current repository state

- Active development and publication branch: `freokro` in the [FreokRO Korangar fork](https://github.com/piabapiaba801/korangar/tree/freokro). The existing `main`, `dev`, and `rebase` branches remain separate.
- The preserved client work before map migration is documented in [TECHNICAL_STATUS_DEV.md](TECHNICAL_STATUS_DEV.md). The server is in the [FreokRO rAthena repository](https://github.com/piabapiaba801/freokro-rathena-korangar); the Auction HUD is a separate private project.
- `5573be9b` introduced the FreokRO emblem and project identity in the README. `cc374ebd` introduced the first map inventory and Prontera terrain pilot.
- `3b5e7e6` replaced the menu map with the FreokRO video. `0cb8ddc` fixed the black screen by skipping directional shadow passes on frames without a map. Together they form the current startup checkpoint on `freokro`.

## Startup identity and map loading

- The client starts with no map loaded. Login and character selection render their own 2D background and interactive windows; Geffen is no longer decoded solely for the menu. The server-provided map still loads when the player enters the world. Geffen's game assets were retained so the map remains available during play.
- The supplied eight-second H.264 MP4 was converted to a 1280 × 720, 20 fps AV1/IVF file. The runtime reads `client/branding/intro.ivf` and loops it behind the menus. There is no separate logo layer because the FreokRO artwork is already in the video. Missing or invalid video falls back to a dark background.
- The client compiled successfully with `--release --locked --features unicode,debug`, and the rebuilt executable was installed in the local game directory. The first visual launch exposed a black screen and repeated `wgpu` validation errors: the map-free menu still recorded directional shadow passes without shadow partitions. The renderer now skips those passes when there is no map. A 40-second diagnostic launch logged zero validation errors, and the user confirmed that the startup screen passed visually. Menu interaction, character selection, and transition into the world remain to be checked with this build.
- The desktop shortcut `FreokRO (cliente independente)` points to the rebuilt executable in `client/game`. The six temporary startup diagnostic logs in `reports/` are local, untracked, and excluded from publication.

## Map migration: implemented scope

- The current code changes **Prontera only**. At load time, it replaces `.gnd` ground texture references with four project-supplied PNGs, removes static `.rsw` scenery objects from the rendered map, and disables the legacy water plane. The original `.rsw`, `.gnd`, `.gat`, GRF, and 7z files remain unchanged.
- The `.gat` cell data, flags, and heights are read as before. Removing visual geometry has nevertheless exposed navigation and visual alignment problems in the live pilot; preservation of `.gat` alone does not prove that navigation feels correct.
- The debug build and `cargo check` passed with `nightly-2026-02-01`, `unicode,debug`, local NASM, and Slang. The user opened the pilot executable in the game installation and supplied a [Prontera screenshot](docs/screenshots/freokro-prontera-map-pilot.jpg) confirming that the new ground renders with NPCs and the player visible. The README presents it as the current map state.
- The screenshot also shows a remaining flag/banner. Its source has **not** been identified. It may be a server entity or another visual source; removing all NPCs to hide it would risk gameplay.

## Asset audit and deletion status

- The first pass indexed **1,104 `.rsw` maps** and **3,160 distinct ground texture paths** from the extracted local game data. All 1,104 maps have a resolved `.gat`; `air_evt` is the only map without a resolved `.gnd` in that extraction.
- [Map inventory](reports/map_inventory.csv), [ground texture dependencies](reports/asset_dependencies.csv), [baseline](reports/baseline.md), and [migration manifest](reports/migration_manifest.json) are committed. Model references, shared UI consumers, archive overrides, and ownership still require auditing.
- **No original assets have been deleted.** The 12 original ground textures referenced by Prontera are in [blocked removals](reports/blocked_removals.csv); [delete candidates](reports/delete_candidates.csv) is empty. No reduction in distribution size has been verified.
- The project-supplied ground PNGs total **14,382,481 bytes**. They still require repeated-tile seam testing and possible optimization.

## Original map authoring checkpoint

- The first BrowEdit3 opening attempt hung at about 21 GB of memory because the generated GND had 40-byte texture records where BrowEdit3 v3.660 expects an 80-byte filename/display-name pair. The generator was corrected and now includes a neutral lightmap. The latest official BrowEdit3 release is still v3.660; a fresh portable installation was downloaded, checked against the official SHA-256 digest, configured without GRFs, and targeted by the `BrowEdit3 FreokRO` desktop shortcut. Visual inspection with that clean installation remains pending.
- The first isolated server cache test marked every cell as water because of the draft RSW water level. The generator now writes rAthena's no-water sentinel. An isolated cache check confirmed walkable land afterward; the live server cache was not changed.
- The user then requested a fixed replacement model that preserves each map's original size while removing terrain relief, static scenery (including trees and vegetation), and non-walkable areas. [The dimension catalog](map-authoring/dimensions.csv) records all **1,265 active maps**: 1,053 dimensions from extracted GND/GAT headers and 212 from the server cache header. Where both sources existed, they agreed. No old terrain, model, height, or cell data was copied. The 51 extracted maps absent from the active server list are outside this batch.
- [The generators](map-authoring/README.md) created RSW/GND/GAT files for all 1,265 active maps in `korangar/archive/data`, totaling **2,831,562,458 bytes** before Git compression. Each RSW has zero static resources; each GND is flat and uses the four FreokRO textures; each GAT preserves its catalog dimensions, has zero height, and is entirely walkable. The four named designs for Prontera and adjoining fields retain active warp coordinates and original dimensions. Prontera is now 312 × 392 GAT and 156 × 196 GND.
- The Korangar format test passes on representative maps, including an `@` name, and checks dimensions, empty static resources, textures, lightmaps, flat heights, and walkable flags. The batch verifier accepted all 1,265 generated maps. An isolated rAthena mapcache run cached all 1,265; inspection of its decompressed data found **104,823,864 walkable land cells**, all matching the catalog dimensions. This test cache remains outside the live server.
- These files are a structural baseline, not finished art or gameplay navigation. They have **not** been installed in the live client or server. The earlier visual Prontera pilot still runs from the old structural map files at runtime. No legacy map asset has been deleted, and no distribution-size reduction has been verified.

## Remaining work, in order

1. Open the corrected Prontera and representative field maps in the clean BrowEdit3 installation and check appearance, editor stability, and texture seams.
2. Install a controlled pilot on the client and rebuild a matching **test** server map cache; check NPC locations, every connected warp, and movement with the unchanged map dimensions.
3. Promote the validated map batch to the live client/server, then replace any remaining Gravity map dependencies. Check archive precedence and every asset consumer before removing legacy files.
4. Develop distinct FreokRO scenery, road layout, obstacles, and height variation where desired. The current all-walkable template intentionally removes those gameplay constraints.

The current live milestone remains the **working Prontera visual pilot**. The 1,265 original flat maps are staged in source control for validation.
