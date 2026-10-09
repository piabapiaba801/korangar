# FreokRO map authoring

FreokRO maps are authored from project-owned design data and textures. The current pilot is Prontera; its source specification is [prontera.json](prontera.json). The generated structural files are in [the client archive](../korangar/archive/data).

## Tools

- [BrowEdit3](https://github.com/Borf/BrowEdit3) edits RSW world settings and objects, GND ground, and GAT walkability. Use a separate portable installation. Set its RO directory to the checkout's korangar/archive folder and leave its GRF list empty. Do not point this authoring workspace at the legacy game archive.
- Blender 5.2 can create original 3D props. The conversion path from Blender output to the client's map model format still needs validation; do not treat a Blender file as a drop-in map.
- The four FreokRO ground PNGs live in korangar/archive/data/texture.

## Current pilot

The Prontera spec sets a 400 x 420 GAT and a 200 x 210 GND with a new crossroad and plaza pattern. All cells are initially walkable so scripted positions can be checked. This is a draft for BrowEdit3 sculpting and navigation design, not a finished map. The GND uses 80-byte texture records (filename plus display name) and a neutral 8 x 8 lightmap so BrowEdit3 can read and render it safely.

The generator reads only the authored JSON:

    python tools/freokro_generate_maps.py --spec map-authoring/prontera.json --output korangar/archive/data

It refuses to overwrite existing generated files. Further visual edits should be saved with BrowEdit3; generate a new draft only in a fresh output directory.

The Korangar format test parses the files. An isolated rAthena mapcache run also produced a one-map `prontera` cache with the expected 400 x 420 cells. This test cache was not installed on the live server.

Do not install these files in the live game until BrowEdit3 visual inspection, NPC/warp navigation, and an integrated client/server test have passed. Keep the existing game assets available during that transition. Technical map IDs and scripted NPC/warp coordinates remain in place for now.
