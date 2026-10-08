# FreokRO Prontera pilot validation

| Check | Status | Evidence |
| --- | --- | --- |
| Map inventory generation | Passed | `tools/freokro_map_audit.ps1` indexed 1,104 maps and 3,160 distinct ground texture references. |
| Rust source check | Passed | `cargo +nightly-2026-02-01 check --locked -p korangar --features unicode,debug --quiet` with local NASM and Slang in `PATH`. |
| Pilot executable build | Passed | `cargo +nightly-2026-02-01 build --locked -p korangar --features unicode,debug --quiet` with local NASM and Slang in `PATH`. |
| Prontera load and visual check | Pending | Requires launching the newly built client with the four PNGs in the active `archive/` directory. |
| 8 × 8 seamlessness | Pending | The supplied artwork has not yet been approved as a seamless repeated texture. |
| Walking, blocked cells, warp and NPC checks | Pending | `.gat` is unchanged; this still needs a live client test. |
| Clean archive isolation | Pending | The pilot still reads original structural map files. |
| Deletion and size reduction | Not attempted | Reference audit is incomplete; zero files removed and zero verified distribution savings. |

## Rollback

The pilot is limited to a local Git commit on `freokro`. Reverting that commit restores the old map loader and removes the new PNGs and reports. No remote branch or original game archive was modified.
