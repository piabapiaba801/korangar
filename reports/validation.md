# FreokRO Prontera pilot validation

| Check | Status | Evidence |
| --- | --- | --- |
| Map inventory generation | Passed | `tools/freokro_map_audit.ps1` indexed 1,104 maps and 3,160 distinct ground texture references. |
| Rust source check | Passed | `cargo +nightly-2026-02-01 check --locked -p korangar --features unicode,debug --quiet` with local NASM and Slang in `PATH`. |
| Pilot executable build | Passed | `cargo +nightly-2026-02-01 build --locked -p korangar --features unicode,debug --quiet` with local NASM and Slang in `PATH`. |
| Prontera load and visual check | Passed for loading | The user opened the pilot and supplied a [screenshot](../docs/screenshots/freokro-prontera-map-pilot.jpg) showing the new ground, player, and NPCs. |
| 8 × 8 seamlessness | Pending | The supplied artwork has not yet been approved as a seamless repeated texture. |
| Walking, blocked cells, warp and NPC checks | Partial | The user observed navigation problems in the live pilot. `.gat` is unchanged, but visual geometry was removed. Warps and blocked cells need focused testing. |
| Residual flag/banner | Open | A flag remains in the screenshot after static `.rsw` objects were cleared. Its source and gameplay role have not been identified. |
| Clean archive isolation | Pending | The pilot still reads original structural map files. |
| Deletion and size reduction | Not attempted | Reference audit is incomplete; zero files removed and zero verified distribution savings. |

## Rollback

The pilot is committed as `cc374ebd` on `freokro` and was published to the fork. Reverting that commit in a later commit would restore the previous map loader and remove the new PNGs and reports. The original game archives were not modified.
