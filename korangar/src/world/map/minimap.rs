use ragnarok_formats::map::GroundData;
use ragnarok_packets::TilePosition;

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
    pub portals: Vec<TilePosition>,
}

impl MiniMapData {
    pub fn from_ground(ground: &GroundData, map_name: &str, map_width: u16, map_height: u16) -> Self {
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
            portals: Self::portal_positions(map_name, map_width, map_height),
        }
    }

    fn portal_positions(map_name: &str, map_width: u16, map_height: u16) -> Vec<TilePosition> {
        let map_name = map_name
            .strip_suffix(".gat")
            .or_else(|| map_name.strip_suffix(".rsw"))
            .unwrap_or(map_name);
        include_str!("freokro_portals.csv")
            .lines()
            .skip(3)
            .filter_map(|line| {
                let mut fields = line.split(',');
                let name = fields.next()?;
                if !name.eq_ignore_ascii_case(map_name) {
                    return None;
                }
                let x = fields.next()?.parse::<u16>().ok()?;
                let y = fields.next()?.parse::<u16>().ok()?;
                (x < map_width && y < map_height).then_some(TilePosition { x, y })
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::MiniMapData;
    use ragnarok_packets::TilePosition;

    #[test]
    fn configured_prontera_and_adjacent_field_portals_use_server_tiles() {
        let prontera = MiniMapData::portal_positions("prontera.gat", 312, 392);
        assert!(prontera.contains(&TilePosition { x: 156, y: 22 }));
        assert!(prontera.contains(&TilePosition { x: 289, y: 203 }));
        assert!(!prontera.contains(&TilePosition { x: 170, y: 378 }));

        let field = MiniMapData::portal_positions("prt_fild08", 400, 400);
        assert!(field.contains(&TilePosition { x: 170, y: 378 }));
        assert!(!MiniMapData::portal_positions("prontera", 100, 100).contains(&TilePosition { x: 289, y: 203 }));
    }
}
