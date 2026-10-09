use ragnarok_formats::map::GroundData;

const MAX_SAMPLES: usize = 48;

#[derive(Clone, Copy)]
pub struct MiniMapRun {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub palette: u8,
}

#[derive(Clone)]
pub struct MiniMapData {
    pub map_width: u16,
    pub map_height: u16,
    pub columns: u8,
    pub rows: u8,
    pub runs: Vec<MiniMapRun>,
}

impl MiniMapData {
    pub fn from_ground(ground: &GroundData, map_width: u16, map_height: u16) -> Self {
        let ground_width = ground.width.max(1) as usize;
        let ground_height = ground.height.max(1) as usize;
        let longest = ground_width.max(ground_height);
        let columns = (ground_width * MAX_SAMPLES).div_ceil(longest).clamp(1, MAX_SAMPLES);
        let rows = (ground_height * MAX_SAMPLES).div_ceil(longest).clamp(1, MAX_SAMPLES);
        let mut runs = Vec::new();

        for y in 0..rows {
            let mut x = 0;
            while x < columns {
                let palette = Self::sample(ground, ground_width, ground_height, columns, rows, x, y);
                let start = x;
                x += 1;
                while x < columns && Self::sample(ground, ground_width, ground_height, columns, rows, x, y) == palette {
                    x += 1;
                }
                runs.push(MiniMapRun {
                    x: start as u8,
                    y: y as u8,
                    width: (x - start) as u8,
                    palette,
                });
            }
        }

        Self {
            map_width,
            map_height,
            columns: columns as u8,
            rows: rows as u8,
            runs,
        }
    }

    fn sample(ground: &GroundData, ground_width: usize, ground_height: usize, columns: usize, rows: usize, x: usize, y: usize) -> u8 {
        let tile_x = if columns == 1 { 0 } else { x * (ground_width - 1) / (columns - 1) };
        // GAT coordinates increase northward, while UI rows increase downward.
        let tile_y = if rows == 1 {
            0
        } else {
            (rows - 1 - y) * (ground_height - 1) / (rows - 1)
        };
        let Some(tile) = ground.ground_tiles.get(tile_y * ground_width + tile_x) else {
            return 0;
        };
        let Ok(surface_index) = usize::try_from(tile.top_surface_index) else {
            return 0;
        };
        let Some(surface) = ground.surfaces.get(surface_index) else {
            return 0;
        };
        let color = surface.color;
        if color.alpha > 0 && color.red < 32 && color.green < 32 && color.blue < 32 {
            return 4;
        }
        u8::try_from(surface.texture_index).unwrap_or(0).min(3)
    }
}
