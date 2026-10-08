# FreokRO Korangar — development and integration checkpoint — October 7–8, 2026

This file records the FreokRO development snapshot and its integration into the public [Korangar fork](https://github.com/piabapiaba801/korangar). The independent FreokRO repository originally had no Git base or `main`. A matching source tree was subsequently reconstructed on upstream tag `v0.1.1-20260220` as the fork's `dev` branch. The fork's `rebase` branch starts at upstream `main` commit `e14d3203ad63f35962f6492d8a6b348eb8f47d47` and applies the FreokRO changes there. The upstream and fork `main` branches were not changed. The separate FreokRO rAthena server and private Auction HUD are linked in the README.

## Upstream integration checkpoint

- **BASE:** upstream [`vE5li/korangar`](https://github.com/vE5li/korangar) `main` at `e14d3203ad63f35962f6492d8a6b348eb8f47d47` (18 commits after tag `v0.1.1-20260220`).
- **SOURCE:** FreokRO client snapshot reconstructed as commit `da7a188328ab33bbae1db5660d4e3316e62fe3d8`; README screenshots were added to fork `dev` afterward.
- **CONFLICTS RESOLVED:** retained upstream's new `lib.rs` application layout and Rust `State` API while porting FreokRO event handling, windows, and overlays. Kept the upstream font files and emotion handling, then adapted FreokRO speech-bubble glyph rendering to the new glyph instruction fields. `main.rs` remains upstream's thin wrapper.
- **COMPILE CHECK:** `cargo +nightly-2026-02-01 check --release --locked -p korangar --features unicode,debug` passed on October 7, 2026 with NASM and Slang 2025.18.2 on `PATH`.
- **RELEASE BUILD:** `cargo +nightly-2026-02-01 build --release --locked -p korangar --features unicode,debug` passed on October 8, 2026. The resulting `korangar.exe` is 51,777,536 bytes with SHA256 `452A1991C3AFC520F67ACC4E99763863CD9125D871BED0148B2586FE076AF050`. `cargo fmt --all -- --check` passed as well.
- **GAME CHECK:** the two README captures document the earlier local `dev` client. The integrated `rebase` binary has not been installed or tested in the live game. Item-text language and metadata remain visibly different between the client and auction HUD.

## Architecture and implemented work

- **IMPLEMENTED:** protocol adaptation for `PACKETVER 20220406` without packet obfuscation. The Ragexe/EXE `20250716` installation is separate. Episode 19 logic is primarily server-side; its presentation in this client requires game testing.
- **IMPLEMENTED:** item descriptions for inventory and shops in `korangar/src/interface/windows/item_details.rs`, including icon, identification state, description, facts, cards, and random options. The window uses the standard title-bar drag handler in `korangar-interface/src/window/mod.rs`.
- **IMPLEMENTED:** item-description text preserves Ragnarok `^RRGGBB` colors through the existing font renderer and shows underscore separators as horizontal rules. Equipped cards now show their item sprites beside their names when the assets load. These changes still need an in-game visual check.
- **IMPLEMENTED:** the in-game character menu has a **Skin** button backed by `korangar/src/interface/windows/skin.rs`. **Classic** is the first option and default. **Original** retains the previous in-game theme. Existing default-theme settings migrate to Classic; selection persists in `client/interface_settings.ron`.
- **IMPLEMENTED:** Classic applies the auction's dark green palette through `korangar/src/state/theme/interface.rs`. The item-description body received additional matching colors, section bars, and footer styling. Other controls using `InterfaceThemeType::InGame` receive the palette.
- **IMPLEMENTED:** the README explains build and runtime dependencies, cross-repository coupling, and official download links. Game assets, `data-freokro.7z`, saved login settings, and the deployed executable are outside this source tree.

## Build and functional status

- **BUILDS:** `cargo +nightly-2026-02-01 check --locked -p korangar --features unicode,debug` and `cargo +nightly-2026-02-01 build --release --locked -p korangar --features unicode,debug` passed on October 7, 2026. Local `slangc` and NASM were added to `PATH` for the build.
- **WORKING:** after the item-description color and card-icon updates, the same check and release build passed again on October 7, 2026. The new executable was copied to the separate local installation; source and installed SHA256 both equal `1776F9B1EF0DCD105B6F7E1AB08697758B16AD25C85E470387D9A1744FA0E241`. The local in-game theme setting was changed to `Classic`.
- **WORKING:** the user confirmed in a live game session that the item-description window can be dragged. October 7 screenshots show the Apple (ID 512) description open alongside the Auction HUD.
- **PARTIAL:** the shared theme covers controls and windows using `InterfaceThemeType::InGame`; any element with hard-coded colors may still need individual work for complete visual parity with the auction.
- **PARTIAL:** the screenshots show different item text sources: the installed Korangar asset describes Apple in Korean, while the HUD catalog describes it in English. The type/fact rows and window styling also differ. The palettes are closer, but this is not complete item-window or game-interface parity.
- **NOT VALIDATED:** visual inspection of Skin → Classic / Original selection, card icons, all item types, persistence after restart, and the rest of the interface in a real game session after this build. The tool's browser security policy blocked a local HTML preview.
- **NOT VALIDATED:** Episode 19 and all auction paths after the skin change. The user reported that server-side escrow passed in the client after its SQL migration, but this build has not independently repeated that test.
- **PENDING:** compare the actual game UI with the Auction HUD, adjust any components outside the shared theme, and test item descriptions and skin switching.

## October 8, 2026 — Character Overview shortcut on the integration branch

- **BASE CHECK:** fetched upstream `main` at `e14d3203ad63f35962f6492d8a6b348eb8f47d47`, unchanged from this branch's merge base. No base rebase or new conflict resolution was required.
- **DEV SOURCE:** fork `dev` commit `97323d0f89751af30902fc9e6c3cdf00c8b4fcf8`. The shortcut was adapted to the integrated `lib.rs` application layout instead of copying the old `main.rs` handler.
- **IMPLEMENTED:** Alt+C toggles Character Overview while a player is loaded. Both Alt keys work. Reopening constructs the window from the current player state, and Alt+C is reserved from custom skill bindings.
- **VALIDATED:** the matching `dev` keyboard test passed with `unicode` and `unicode,debug`; the user confirmed close and reopen in the live `dev` client. This integration branch passed `cargo +nightly-2026-02-01 fmt --check --package korangar` and `cargo +nightly-2026-02-01 build --release --locked -p korangar --features unicode,debug`.
- **INTEGRATION BUILD:** the `rebase` executable is 51,755,008 bytes with SHA256 `90834E8D7A8764B88827CCD51E6A60C24D372DB6254B8E08A5A2826D22C88810`. It remains separate from the installed `dev` executable and has not received an in-game check.
- **SEPARATE ISSUE:** relogging corrected a previously stale level display. The cause of missed live stat updates has not been established by this shortcut test.
