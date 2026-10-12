# FreokRO character menu art

The character selection and creation screens use original project artwork inspired by the supplied layout references. The watermarked UI sample and screenshots of other games are not included as runtime assets.

| Source | Runtime asset | Use |
| --- | --- | --- |
| `forest-scene-source.png` | `character-scene.png` | Static menu background |
| `female-sheet-source.png` | `female-preview.png` | Front-facing female character preview |
| `male-preview-source.png` | `male-preview.png` | Front-facing male character preview |

Run `tools/build_freokro_character_menu.py` with Pillow to rebuild the runtime PNGs under `korangar/archive/data/texture/freokro`. The female sheet was provided by the project owner. The forest and male character images were generated specifically for this FreokRO menu. The preview portraits currently indicate sex only; hair style and color controls affect the character creation packet but do not yet alter the portrait.
