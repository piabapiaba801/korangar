use std::fs;
use std::path::Path;

use ragnarok_bytes::{ByteReader, FromBytes};
use ragnarok_formats::map::{GatData, GroundData, MapData, TileFlags};
use ragnarok_formats::version::InternalVersion;

fn parse<T: FromBytes>(file_name: &str) -> T {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("archive/data");
    let bytes = fs::read(root.join(file_name)).unwrap();
    let mut reader: ByteReader<Option<InternalVersion>> = ByteReader::with_default_metadata(&bytes);
    let value = T::from_bytes(&mut reader).unwrap();
    assert!(reader.is_empty(), "trailing data in {file_name}");
    value
}

#[test]
fn freokro_authored_maps_are_readable() {
    for (name, width, height) in [
        ("06guild_01", 100, 100),
        ("1@20cn1", 300, 260),
        ("prontera", 312, 392),
        ("prt_fild05", 400, 400),
        ("prt_fild06", 380, 340),
        ("prt_fild08", 400, 400),
    ] {
        let world: MapData = parse(&format!("{name}.rsw"));
        let ground: GroundData = parse(&format!("{name}.gnd"));
        let gat: GatData = parse(&format!("{name}.gat"));
        assert_eq!(world.ground_file, format!("{name}.gnd"));
        assert_eq!(world.gat_file, format!("{name}.gat"));
        assert_eq!(world.water_settings.unwrap().water_level, Some(1_000_000.0));
        assert_eq!(world.resources.resources_amount, 0);
        assert!(world.resources.objects.is_empty());
        assert!(world.resources.light_sources.is_empty());
        assert!(world.resources.sound_sources.is_empty());
        assert!(world.resources.effect_sources.is_empty());
        assert_eq!((ground.width, ground.height), (width / 2, height / 2));
        assert_eq!((gat.map_width, gat.map_height), (width, height));
        assert_eq!(ground.texture_name_length, 80);
        assert_eq!(ground.textures.len(), 4);
        assert_eq!(ground.surfaces.len(), 8);
        assert!(ground.textures.iter().all(|texture| texture.starts_with("freokro_ground_")));
        assert_eq!(ground.light_map_count, 1);
        assert_eq!((ground.light_map_width, ground.light_map_height), (8, 8));
        assert_eq!(ground.light_map_cells_per_grid, 1);
        assert!(ground.surfaces.iter().all(|surface| surface.light_map_index == 0));
        assert!(ground.surfaces[4..].iter().all(|surface| {
            surface.color.red == 0 && surface.color.green == 0 && surface.color.blue == 0 && surface.color.alpha == 255
        }));
        assert_eq!(gat.tiles.len(), (width * height) as usize);
        assert!(gat.tiles.iter().all(|tile| {
            tile.flags == TileFlags::WALKABLE
                && tile.southwest_corner_height == 0.0
                && tile.southeast_corner_height == 0.0
                && tile.northwest_corner_height == 0.0
                && tile.northeast_corner_height == 0.0
        }));
        assert!(ground.ground_tiles.iter().all(|tile| {
            tile.southwest_corner_height == 0.0
                && tile.southeast_corner_height == 0.0
                && tile.northwest_corner_height == 0.0
                && tile.northeast_corner_height == 0.0
        }));
        let ground_width = ground.width as usize;
        let ground_height = ground.height as usize;
        for (index, tile) in ground.ground_tiles.iter().enumerate() {
            let x = index % ground_width;
            let y = index / ground_width;
            let border = x == 0 || y == 0 || x == ground_width - 1 || y == ground_height - 1;
            assert_eq!(tile.top_surface_index >= 4, border, "wrong border in {name} at {x}, {y}");
        }
    }
}
