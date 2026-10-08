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
fn freokro_prontera_files_are_readable() {
    let world: MapData = parse("prontera.rsw");
    let ground: GroundData = parse("prontera.gnd");
    let gat: GatData = parse("prontera.gat");
    assert_eq!(world.ground_file, "prontera.gnd");
    assert_eq!(world.gat_file, "prontera.gat");
    assert!(world.resources.objects.is_empty());
    assert_eq!((ground.width, ground.height), (200, 210));
    assert_eq!((gat.map_width, gat.map_height), (400, 420));
    assert_eq!(ground.textures.len(), 4);
    assert!(ground.textures.iter().all(|name| name.starts_with("freokro_ground_")));
    assert_eq!(gat.tiles.len(), 400 * 420);
    assert!(gat.tiles.iter().all(|tile| tile.flags == TileFlags::WALKABLE));
}
