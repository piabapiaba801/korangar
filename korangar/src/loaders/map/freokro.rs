use ragnarok_formats::color::ColorBGRA;
use ragnarok_formats::map::{GatData, GroundData, GroundTile, MapData, Surface};
use ragnarok_formats::version::Version;

const TEXTURES: [&str; 4] = [
    "freokro_ground_base.png",
    "freokro_ground_light.png",
    "freokro_ground_dark.png",
    "freokro_ground_transition.png",
];

/// Replace map art at load time while retaining the original GAT cells and
/// heights used for movement and server coordinates.
pub(super) fn apply(resource_file: &str, map_data: &mut MapData, ground_data: &mut GroundData, gat_data: &GatData) {
    // Some maps have a model-only floor or no GND file at all. Once their
    // scenery models are removed, a floor derived from GAT is required.
    if ground_data.textures.is_empty()
        || ground_data.surfaces.is_empty()
        || ground_data.ground_tiles.iter().all(|tile| tile.top_surface_index < 0)
    {
        *ground_data = ground_from_gat(gat_data);
    }

    ground_data.textures = TEXTURES.iter().map(|name| (*name).to_string()).collect();
    ground_data.texture_count = TEXTURES.len() as i32;

    // Keep Prontera's already approved distribution. Other maps use the same
    // weights with a stable map-specific offset so neighboring areas vary.
    let map_seed = if resource_file == "prontera" {
        0
    } else {
        resource_file
            .bytes()
            .fold(2_166_136_261usize, |hash, byte| hash.wrapping_mul(16_777_619) ^ byte as usize)
    };
    for (index, surface) in ground_data.surfaces.iter_mut().enumerate() {
        let selection = index
            .wrapping_add(map_seed)
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345)
            % 100;
        surface.texture_index = match selection {
            0..=79 => 0,
            80..=89 => 1,
            90..=97 => 2,
            _ => 3,
        };
    }

    // RSW objects are static scenery. NPCs, players, and monsters arrive from
    // the server; light and sound sources remain available to the map.
    map_data.resources.objects.clear();
    map_data.water_settings = None;
}

/// Build a visible ground for maps whose floor existed only as RSW models.
/// A single GND tile covers two GAT cells in each direction.
pub(super) fn ground_from_gat(gat_data: &GatData) -> GroundData {
    let gat_width = gat_data.map_width.max(0) as usize;
    let gat_height = gat_data.map_height.max(0) as usize;
    let width = gat_width.div_ceil(2);
    let height = gat_height.div_ceil(2);
    let sample = |x: usize, y: usize, corner: fn(&ragnarok_formats::map::Tile) -> f32| {
        gat_data
            .tiles
            .get(y.saturating_mul(gat_width).saturating_add(x))
            .map(corner)
            .unwrap_or(0.0)
    };

    let mut surfaces = Vec::with_capacity(width * height);
    let mut ground_tiles = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let surface_index = surfaces.len() as i32;
            surfaces.push(Surface {
                u: [0.0, 1.0, 0.0, 1.0],
                v: [1.0, 1.0, 0.0, 0.0],
                texture_index: 0,
                light_map_index: -1,
                color: ColorBGRA {
                    blue: 255,
                    green: 255,
                    red: 255,
                    alpha: 255,
                },
            });
            ground_tiles.push(GroundTile {
                southwest_corner_height: sample(2 * x, 2 * y, |tile| tile.southwest_corner_height),
                southeast_corner_height: sample(2 * x + 1, 2 * y, |tile| tile.southeast_corner_height),
                northwest_corner_height: sample(2 * x, 2 * y + 1, |tile| tile.northwest_corner_height),
                northeast_corner_height: sample(2 * x + 1, 2 * y + 1, |tile| tile.northeast_corner_height),
                top_surface_index: surface_index,
                north_surface_index: if y + 1 < height { surface_index } else { -1 },
                east_surface_index: if x + 1 < width { surface_index } else { -1 },
            });
        }
    }

    GroundData {
        signature: Default::default(),
        version: Version::new(1, 7),
        width: width as i32,
        height: height as i32,
        zoom: 1.0,
        texture_count: TEXTURES.len() as i32,
        texture_name_length: 40,
        textures: TEXTURES.iter().map(|name| (*name).to_string()).collect(),
        light_map_count: 0,
        light_map_width: 0,
        light_map_height: 0,
        light_map_cells_per_grid: 0,
        _skip: Some(Vec::new()),
        _skip2: None,
        surface_count: surfaces.len() as i32,
        surfaces,
        ground_tiles,
    }
}
