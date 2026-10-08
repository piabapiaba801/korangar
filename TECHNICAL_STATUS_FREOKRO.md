# FreokRO technical status — October 8, 2026

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

- The local BrowEdit3 installation was identified at D:\ragnarok\ferramentas do servidor\browedite 3. Its 2022 configuration still targeted unrelated SkyRO GRFs. A separate portable BrowEdit3 v3.660 was prepared locally under Desktop/freokro/tools/browedit3-660, with the source archive as its data directory and an empty GRF list. That editor installation is outside Git.
- Blender 5.2 is installed locally. BrowEdit3 is the primary editor for .rsw, .gnd, and .gat; Blender will be used for original 3D props after an import/export path is validated.
- [The authored Prontera design](map-authoring/prontera.json) now defines an original 400 × 420 GAT and 200 × 210 GND. [The map generator](tools/freokro_generate_maps.py) reads this design only and wrote korangar/archive/data/prontera.rsw, .gnd, and .gat. It does not read the legacy map files, GRFs, or the old map cache. The dimensions were chosen to contain current server script coordinates, whose observed maxima were x=299 and y=379.
- The generated map is a flat, fully walkable authoring baseline with the four supplied FreokRO ground textures and no static scenery. It is **not yet a finished Prontera**, is **not installed in the live game directory**, and the server still uses its existing map cache. No original map assets have been removed.
- The Korangar integration test in [authored_map.rs](korangar/tests/authored_map.rs) passed: the client format parser read all three files without trailing bytes and confirmed the dimensions, texture names, empty static scenery, and walkable GAT cells. BrowEdit3 inspection and an in-client/server navigation test still remain before promotion.

## Remaining work, in order

1. Inspect and refine this original Prontera in BrowEdit3, then validate file parsing, visual appearance, NPC locations, warps, and movement in a controlled client/server test.
2. Rebuild the server map cache from the new FreokRO GAT when the pilot is ready, so client and server use matching cell dimensions and navigation.
3. Repeat original authoring for adjacent and then remaining active maps. Keep technical map names and scripted NPC/warp coordinates during this stage as agreed with the user; do not copy legacy geometry or navigation cells.
4. Replace runtime dependencies map by map and remove legacy assets only after checking all consumers. Record each proven removal and the actual distribution size change.

The current live milestone remains the **working visual pilot**. The original structural Prontera files are an **unpromoted authoring draft**.
