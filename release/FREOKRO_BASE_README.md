# 🌿 FreokRO Base

FreokRO Base is a simplified, independent starting point for a **Ragnarok-like Plus** game. Its client adapts [Korangar](https://github.com/vE5li/korangar) and is designed to connect to the separately maintained [FreokRO rAthena server](https://github.com/piabapiaba801/freokro-rathena-korangar). It does not include the private Auction HUD.

> **Release status:** local candidate. The FreokRO code and authored resources still need gameplay and asset-provenance validation before a public release. This repository does **not** claim to be free of every Gravity-owned dependency yet.

The owner visually checked the local October 13 test client through login, world selection, character selection/creation, and lateral movement. Broader gameplay and asset review remain on the release checklist.

## ✨ Current visual foundation

- A FreokRO startup loop leads into login, world selection, character selection, and creation with a common dark green and emerald menu palette.
- Occupied character slots show the same sprite resources as the in-world character. The turquoise Azuk is the temporary female base; the male uses one original static placeholder.
- The in-game **Classic** skin is the default for common HUD controls, menus, and windows.
- The client supplies all 1,265 project-generated RSW/GND/GAT map triplets, the original ground textures, and the catalog and generator that reproduce them. Technical map names and dimensions remain as compatibility references.

Appearance variants, rear/diagonal character views, NPCs, monster art, and broader gameplay validation remain incomplete. Hair and color choices are sent to the server but are not yet visually distinct in the project-owned placeholders.

## 🛠️ Build and run locally

Install [Rustup](https://rustup.rs/), [Slang](https://github.com/shader-slang/slang/releases) (`slangc` 2025.18.2 or newer), and [NASM](https://www.nasm.us/) on the build machine. This repository selects Rust nightly `2026-02-01`.

```text
cargo +nightly-2026-02-01 build --release --locked -p korangar --features unicode,debug
```

To regenerate the authored flat map set, install Python with Pillow and run:

```text
python tools/freokro_generate_active_maps.py --dimensions map-authoring/dimensions.csv --spec-dir map-authoring --output korangar/archive/data
```

See [map authoring](map-authoring/README.md) for packaging and a matching server map cache. A local MariaDB and the compatible FreokRO rAthena login, char, and map servers are required for gameplay. Configure your own local server connection and provide any required compatible game resources; no account, password, legacy GRF, or server configuration is bundled here.

## 📦 Repository boundary

This repository contains a curated client snapshot with a fresh Git history. It includes the project-generated map triplets and excludes local game archives, personal screenshots, the private Auction HUD, and the source fork's development history. Advanced personalized work remains in a separate private development repository.

Korangar code retains its upstream attribution and [MIT license](LICENSE.md). FreokRO additions are documented under `art/freokro`, `map-authoring`, and `tools`. Audit all assets and dependencies before redistributing a playable package.
