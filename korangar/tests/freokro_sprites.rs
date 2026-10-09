use std::fs;
use std::path::PathBuf;

use ragnarok_bytes::{ByteReader, FromBytes};
use ragnarok_formats::action::ActionsData;
use ragnarok_formats::sprite::SpriteData;
use ragnarok_formats::version::InternalVersion;

#[test]
fn freokro_sprites_decode_with_the_game_format_parser() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("archive/data/sprite");

    for (directory, stem, width, height) in [
        ("npc", "FREOKRO_PORTAL", 96, 128),
        ("npc", "FREOKRO_FLAG", 80, 120),
        ("아이템", "FREOKRO_TAMING", 64, 64),
    ] {
        let sprite = fs::read(root.join(directory).join(format!("{stem}.spr"))).unwrap();
        let mut reader: ByteReader<Option<InternalVersion>> = ByteReader::with_default_metadata(&sprite);
        let sprite = SpriteData::from_bytes(&mut reader).unwrap();
        assert_eq!(sprite.palette_image_count, 0);
        assert_eq!(sprite.rgba_image_count, Some(1));
        assert_eq!(sprite.rgba_image_data[0].width, width);
        assert_eq!(sprite.rgba_image_data[0].height, height);
        assert!(sprite.palette.is_some());

        let actions = fs::read(root.join(directory).join(format!("{stem}.act"))).unwrap();
        let mut reader: ByteReader<Option<InternalVersion>> = ByteReader::with_default_metadata(&actions);
        let actions = ActionsData::from_bytes(&mut reader).unwrap();
        assert_eq!(actions.actions.len(), 8);
        assert!(actions.actions.iter().all(|action| action.motions.len() == 1));
        assert!(actions.actions.iter().all(|action| {
            action.motions[0].sprite_clips.len() == 1 && action.motions[0].sprite_clips[0].sprite_type == Some(1)
        }));
        assert_eq!(actions.delays.as_ref().unwrap().len(), 8);
    }
}
