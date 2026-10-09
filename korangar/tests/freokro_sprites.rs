use std::fs;
use std::path::PathBuf;

use ragnarok_bytes::{ByteReader, FromBytes};
use ragnarok_formats::action::ActionsData;
use ragnarok_formats::sprite::SpriteData;
use ragnarok_formats::version::InternalVersion;

#[test]
fn freokro_sprites_decode_with_the_game_format_parser() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("archive/data/sprite");

    for (directory, stem, width, height, images, action_count) in [
        ("npc", "FREOKRO_PORTAL", 96, 128, 1, 8),
        ("npc", "FREOKRO_FLAG", 80, 120, 1, 8),
        ("아이템", "FREOKRO_TAMING", 64, 64, 1, 8),
        ("", "FREOKRO_CURSOR", 48, 48, 5, 112),
        ("아이템", "FREOKRO_SKILL_PLACEHOLDER", 32, 32, 1, 8),
        ("npc", "FREOKRO_FEMALE", 72, 96, 30, 104),
    ] {
        let sprite = fs::read(root.join(directory).join(format!("{stem}.spr"))).unwrap();
        let mut reader: ByteReader<Option<InternalVersion>> = ByteReader::with_default_metadata(&sprite);
        let sprite = SpriteData::from_bytes(&mut reader).unwrap();
        assert_eq!(sprite.palette_image_count, 0);
        assert_eq!(sprite.rgba_image_count, Some(images));
        assert!(sprite.rgba_image_data.iter().all(|image| image.width == width && image.height == height));
        assert!(sprite.palette.is_some());

        let actions = fs::read(root.join(directory).join(format!("{stem}.act"))).unwrap();
        let mut reader: ByteReader<Option<InternalVersion>> = ByteReader::with_default_metadata(&actions);
        let actions = ActionsData::from_bytes(&mut reader).unwrap();
        assert_eq!(actions.actions.len(), action_count);
        assert!(actions.actions.iter().all(|action| !action.motions.is_empty()));
        assert!(actions.actions.iter().all(|action| {
            action.motions.iter().all(|motion| motion.sprite_clips.len() == 1 && motion.sprite_clips[0].sprite_type == Some(1))
        }));
        assert_eq!(actions.delays.as_ref().unwrap().len(), action_count);
        if stem == "FREOKRO_FEMALE" {
            assert!(actions.actions[..8].iter().all(|action| action.motions.len() == 30));
            assert!(actions.actions[8..].iter().all(|action| action.motions.len() == 1));
        }
    }
}
