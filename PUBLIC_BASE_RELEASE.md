# FreokRO public base: release boundary

**Status:** local Git candidate. No GitHub repository or remote publication has been created for this version.

The public base is a simplified starting point for a Ragnarok-like Plus game. It provides the FreokRO client source, project-authored map generator and textures, starter character sprites, common interface skin, and documentation for connecting to the separately maintained [FreokRO rAthena server](https://github.com/piabapiaba801/freokro-rathena-korangar). It is not the private advanced edition.

## Two development lines

| Public base | Local advanced edition |
| --- | --- |
| Independent local `freokro-base` repository with a new, curated Git history | Separate private `freokro-priv` repository and local checkout |
| Reproducible baseline, simple character art, common menus and HUD | Future customized art, deeper appearance variants, and experimental systems |
| All 1,265 project-generated map triplets, plus their generator and catalog | Local testing resources and further map customization |
| No private Auction HUD source, credentials, game archives, or user screenshots | Local integrations and test installations |

Changes that belong in both editions should be developed in the base first and imported into the advanced line deliberately. Private customization does not flow into the public repository automatically.

## Visual scope of the first candidate

1. The turquoise Azuk is the default female character. Her supplied front and lateral artwork drives the initial idle and east/west walk states. Other directions and appearance variants are placeholders.
2. Occupied character slots render the currently selected character's in-game sprite resources. The server supplies job, sex, hair, and equipment identifiers, but the current authored art does not yet distinguish every combination.
3. Intro, login, world selection, character selection/creation, and the in-game Classic skin share the same dark green, emerald, and cream visual language.
4. A project-owned static male sprite is temporary until animated original male artwork is available.

## Publication gate

- The client compiled from the curated repository on October 13, 2026, and the `freokro_sprites` parser test passed. The project owner checked login, world selection, character selection/creation, and lateral movement in the physical test client and confirmed that the tested views are correct. Representative map regeneration and a wider gameplay pass remain to be recorded.
- Verify both sexes, switching occupied slots, entering the world, the creation controls, and the other common HUD windows at the target resolution before a remote release.
- Review every distributed asset and dependency for provenance. The current FreokRO client is **not yet confirmed to be free of all Gravity-owned game assets**. Do not describe the candidate as such until the audit is complete.
- Check the public snapshot for private configuration, passwords, personal data, binaries, bulk archives, and screenshots containing third-party game art.
- Include upstream Korangar attribution and the applicable license. Document the server dependency and the remaining placeholder limitations.

The source fork's `main`, `dev`, and `rebase` branches are outside this release line. The public base already has its own local root commit; its GitHub repository will be created after the release gate passes.
