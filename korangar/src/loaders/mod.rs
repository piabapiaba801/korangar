mod action;
mod animation;
mod archive;
mod color;
mod rectangle;

mod r#async;
mod effect;
pub mod error;
mod font;
mod gamefile;
mod map;
mod model;
mod server;
mod smoothing;
mod sprite;
mod texture;
mod video;

pub use self::action::*;
pub use self::animation::*;
pub use self::r#async::*;
pub use self::effect::EffectLoader;
pub use self::font::{FontLoader, FontSize, GlyphInstruction, OverflowBehavior, Scaling};
pub use self::gamefile::*;
pub use self::map::{GAT_TILE_SIZE, MapLoader};
pub use self::model::*;
pub use self::server::{ClientInfo, ClientInfoPathExt, PacketVersion, ServiceId, load_client_info};
pub use self::smoothing::{smooth_ground_normals, smooth_model_normals};
pub use self::sprite::*;
pub use self::texture::{ImageType, TextureLoader, TextureSetBuilder, TextureSetTexture};
pub use self::video::VideoLoader;

pub const FALLBACK_BMP_FILE: &str = "missing.bmp";
pub const FALLBACK_JPEG_FILE: &str = "missing.jpg";
pub const FALLBACK_PNG_FILE: &str = "missing.png";
pub const FALLBACK_TGA_FILE: &str = "missing.tga";
pub const FALLBACK_MODEL_FILE: &str = "missing.rsm";
pub const FALLBACK_SPRITE_FILE: &str = "npc\\missing.spr";
pub const FALLBACK_ACTIONS_FILE: &str = "npc\\missing.act";

/// Redirect legacy sprite identifiers to FreokRO-owned artwork.
///
/// Keeping the identifiers allows existing server NPC definitions and skill
/// metadata to work without shipping the corresponding legacy sprite files.
pub fn freokro_sprite_path(path: &str) -> &str {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    match file.to_ascii_uppercase().as_str() {
        "WARPNPC.SPR" | "HIDDEN_WARP_NPC.SPR" => "npc\\FREOKRO_PORTAL.spr",
        "WARPNPC.ACT" | "HIDDEN_WARP_NPC.ACT" => "npc\\FREOKRO_PORTAL.act",
        "GUILD_FLAG.SPR" => "npc\\FREOKRO_FLAG.spr",
        "GUILD_FLAG.ACT" => "npc\\FREOKRO_FLAG.act",
        "SA_TAMINGMONSTER.SPR" => "아이템\\FREOKRO_TAMING.spr",
        "SA_TAMINGMONSTER.ACT" => "아이템\\FREOKRO_TAMING.act",
        _ => path,
    }
}
