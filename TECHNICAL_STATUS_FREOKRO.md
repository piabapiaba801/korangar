# FreokRO technical status — October 8, 2026

## Current repository state

- Active development and publication branch: `freokro` in the [FreokRO Korangar fork](https://github.com/piabapiaba801/korangar/tree/freokro). The existing `main`, `dev`, and `rebase` branches remain separate.
- The preserved client work before map migration is documented in [TECHNICAL_STATUS_DEV.md](TECHNICAL_STATUS_DEV.md). The server is in the [FreokRO rAthena repository](https://github.com/piabapiaba801/freokro-rathena-korangar); the Auction HUD is a separate private project.
- `5573be9b` introduced the FreokRO emblem and project identity in the README. `cc374ebd` introduced the first map inventory and Prontera terrain pilot.

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

## Remaining work, in order

1. Identify the source of the remaining flag/banner and hide only visual resources that can be removed without losing NPC, warp, or interaction behavior.
2. Extend the ground and static scenery override to all supported maps with safe bounds for maps that have no terrain vertices. Keep `.gat` and server logic unchanged. Record exceptions for maps that rely on bridges, multiple heights, or other essential geometry.
3. Test representative cities, fields, interiors, dungeons, water maps, and warps in the client. Check loading, movement, missing textures, NPCs, FPS, and crashes before declaring any group migrated.
4. Complete the cross-resource dependency audit. Remove old map-only files from the active distribution only after proving they have no remaining consumers, then measure actual bytes saved.

The current milestone is a **working visual pilot**, not a completed all-map migration. See [validation.md](reports/validation.md) for the test matrix and open checks.
