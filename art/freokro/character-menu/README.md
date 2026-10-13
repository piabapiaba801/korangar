# FreokRO character menu art

The character selection and creation screens use original project artwork inspired by the supplied layout references. The watermarked UI sample and screenshots of other games are not included as runtime assets.

| Source | Runtime asset | Use |
| --- | --- | --- |
| `forest-scene-source.png` | `character-scene.png` | Static menu background |
| `../sprites/female-placeholder-source.gif` | `female-preview.png` | Turquoise female preview matching the world sprite |
| `male-preview-source.png` | `male-preview.png` | Front-facing male character preview |

Run `tools/build_freokro_character_menu.py` with Pillow to rebuild the runtime PNGs under `korangar/archive/data/texture/freokro`. The former pink `female-sheet-source.png` remains unused in the development checkout and is excluded from the public-base snapshot. The turquoise GIF and side-walk sheet were supplied by the project owner; the forest and male character images were generated specifically for this FreokRO menu. Occupied character slots draw their current in-game sprite resources. The creation preview remains illustrative: hair style and color controls affect the packet but do not yet alter the artwork.
