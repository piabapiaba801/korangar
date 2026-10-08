# FreokRO map migration baseline

- Branch: `freokro` in the local Korangar fork checkout. Starting commit: `461a8c0b`.
- Scope of this checkpoint: inventory the available extracted map files and prepare a visual pilot for `prontera`. No server or production files were changed.
- Active client archive list in the local test installation: `data-freokro.7z`, then `archive/`, configured in `client/game_archives.ron`. This client does not use `data.ini` for that list.
- Map loader: `.rsw` supplies scene resources, `.gnd` supplies terrain and texture names, and `.gat` supplies cells, flags, and heights. The pilot keeps all three source map files and leaves `.gat` unchanged.
- Inventory source: the existing local `extracted/data` directory. The audit script reads it without modifying it. The files are not committed to Git.
- Inventory: 1,104 `.rsw` maps; 1,103 resolved `.gnd` references; 1,104 resolved `.gat` references; 3,160 distinct ground texture paths. `air_evt` has no resolved `.gnd` in this extraction.
- Prontera: `.gnd` 156 × 196; `.gat` 312 × 392; 12 original ground texture references; combined `.rsw`/`.gnd`/`.gat` size 6,199,398 bytes.
- Four user-provided PNGs are used as the base, light, dark, and transition art. Their combined size is 14,382,481 bytes. Image ownership beyond the user's supplied files has not been independently verified.

| Pilot texture | SHA-256 |
| --- | --- |
| `freokro_ground_base.png` | `03A281AEDE46AC2D5EEAE243CDB1505FA756D1196D51CC6BBB862373A3030D4D` |
| `freokro_ground_light.png` | `16F458C8591BDBF6D7DC75C7ABD77F47DAC4B97BBA0717A926B31B51B6212D84` |
| `freokro_ground_dark.png` | `C635CE35478B7FA22BF02704F9857E0423EF688A33CD82663E9CC679323BBA77` |
| `freokro_ground_transition.png` | `FAEEB920D7FF9842843B520ECBFB0FBF26B318BA641B2325230D0760494664F6` |

## Limits of this audit

`asset_dependencies.csv` indexes ground texture references found in `.gnd` files. It does not yet parse `.rsw` model references, model texture references, UI consumers, or archive priority overrides. Its `scope` and `ownership` values remain unknown. The map inventory does not claim runtime compatibility or load success. Therefore no original map asset is approved for deletion.

The pilot replaces Prontera's ground texture names and suppresses static `.rsw` objects in memory. It still reads original `.rsw`, `.gnd`, and `.gat` structural data and the game archive for all other resources. It is a visual migration experiment, not an independent replacement for the Gravity map file formats or art package.

No physical backup was made, following the user's earlier direction to remove physical-copy stages. Git history and the unchanged source archives provide rollback for this local checkpoint.
