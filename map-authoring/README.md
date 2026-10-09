# FreokRO map authoring

FreokRO has a reproducible flat-map baseline for the **1,265 maps enabled by the current rAthena configuration**. The [dimension catalog](dimensions.csv) preserves each map's GAT and GND size. Of these sizes, 1,053 came from extracted map headers and 212 from server map-cache headers. Where both sources existed, their dimensions matched. Only dimensions and technical map names were retained; legacy terrain, objects, heights, and navigation cells were not imported.

## Fixed replacement model

Every generated map has:

- An original RSW with no static objects, including trees, vegetation, buildings, lights, sounds, or effects, and no water plane.
- A flat GND using the four project-supplied FreokRO ground PNGs in [the texture directory](../korangar/archive/data/texture). The GND includes BrowEdit3-compatible 80-byte texture records and a neutral lightmap.
- A same-size GAT with zero height and every cell marked as walkable land. Visual roads do not restrict movement.
- A one-GND-tile black perimeter on every map. The perimeter is a visible floor tint; its GAT cells remain walkable up to the actual map edge.

The three fields adjoining Prontera have explicit road anchors at current warp coordinates: [prt_fild05](prt_fild05.json), [prt_fild06](prt_fild06.json), and [prt_fild08](prt_fild08.json). [Prontera](prontera.json) keeps its 312 × 392 GAT and 156 × 196 GND. These authored designs keep the existing technical map IDs and server NPC/warp coordinates for now. All other active maps use the same deterministic flat template with a centered visual crossroad.

## Rebuild

The committed catalog and generator can recreate the files without the old map art or server cache:

```text
python tools/freokro_generate_active_maps.py --dimensions map-authoring/dimensions.csv --spec-dir map-authoring --output korangar/archive/data
```

The generator verifies and skips complete maps with matching dimensions and flat-cell format. It rejects partial or mismatched outputs. Use `--refresh-ground` to upgrade an existing earlier flat GND to the black-perimeter design without rewriting RSW or GAT files. The dimension catalog was originally assembled with [the dimension audit tool](../tools/freokro_build_map_dimensions.py), which reads only map headers and the active map list.

Korangar requires a **non-solid** 7z archive for random file access. After generating the maps, package the exact 1,265 RSW/GND/GAT triplets and four PNG textures with:

```text
python tools/freokro_package_maps.py --dimensions map-authoring/dimensions.csv --archive-root korangar/archive --output /path/to/game/freokro-maps.7z
```

The packaging tool requires 7z or 7zz, checks every expected source file, verifies that the package is non-solid with the exact entry set, and runs the 7z integrity check. Place the package last in `client/game_archives.ron` so it overrides earlier game archives. Build a matching rAthena map cache from the same authored files before testing movement.

## Validation and current use

The Korangar format test parses representative RSW/GND/GAT files, including a map name containing `@`, and checks dimensions, flat heights, walkable flags, empty static scenery, textures, and lightmaps. A full isolated rAthena mapcache run accepted all 1,265 maps; inspection of the resulting cache confirmed **104,823,864 walkable land cells** and every catalog dimension. The test cache is outside the live server.

[BrowEdit3](https://github.com/Borf/BrowEdit3/releases) can refine the maps. Point its RO directory at this checkout's `korangar/archive` folder and keep its GRF list empty. A clean v3.660 installation was prepared locally after the first draft exposed an incorrect GND texture record length. Visual inspection in the clean editor remains pending.

These files are a structural baseline, not finished environments. On October 9, 2026, the package and matching server cache were installed in the local FreokRO game and server directories; the client executable was rebuilt without the older Prontera-only texture override. The user visually approved the first flat-map batch, but identified places where clicks did not lead to movement. Higher-priority legacy rAthena map caches caused that mismatch and have since been cleared. The perimeter and new in-game minimap have compiled but still need visual validation. Verify appearance, NPC positions, connected warps, and movement before removing legacy assets or distributing the package. The 51 extracted maps absent from the active server list are outside this batch.

The **active map baseline is complete at 1,265/1,265 (100%)** for new RSW/GND/GAT geometry, flat ground, empty static RSW resource lists, and walkable GAT cells. **Legacy archive removal is 0%**: the older multi-gigabyte game archive remains installed because it also supplies sprites, interface, audio, and other non-map resources. No percentage is established for all Gravity-derived content in the game.

The [legacy map removal queue](../reports/legacy_map_removal_queue.csv) inventories map files in the older local `data-freokro.7z`: 1,053 active map IDs now have authored replacements, while 51 old map IDs are outside the current server list. This is a review queue, not a deletion instruction. The old archive still contains resources needed outside map geometry, and 21 active legacy map IDs lack a GND entry in that archive. Keep it until gameplay and other asset consumers are checked.
