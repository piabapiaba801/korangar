use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::Context;

use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, this_player};
use crate::world::MiniMapData;

pub struct MiniMapWindow {
    map_name: String,
    data: MiniMapData,
}

impl MiniMapWindow {
    pub fn new(map_name: String, data: MiniMapData) -> Self {
        Self { map_name, data }
    }
}

impl CustomWindow<ClientState> for MiniMapWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::MiniMap)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Minimap",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 224.0,
            maximum_width: 224.0,
            closable: true,
            elements: MiniMapBody::new(self.map_name, self.data),
        }
    }
}

struct MiniMapBody {
    map_name: String,
    data: MiniMapData,
    position_text: String,
}

impl MiniMapBody {
    fn new(map_name: String, data: MiniMapData) -> Self {
        Self {
            map_name,
            data,
            position_text: String::new(),
        }
    }

    fn palette(index: u8) -> Color {
        match index {
            1 => Color::rgb_u8(200, 185, 147),
            2 => Color::rgb_u8(103, 94, 77),
            3 => Color::rgb_u8(139, 128, 103),
            4 => Color::BLACK,
            _ => Color::rgb_u8(158, 143, 111),
        }
    }
}

impl Element<ClientState> for MiniMapBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.position_text = match state.try_get(&this_player()) {
            Some(player) => {
                let position = player.get_common().tile_position;
                format!("{}  |  {}, {}", self.map_name, position.x, position.y)
            }
            None => self.map_name.clone(),
        };
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo {
            area: resolver.with_height(216.0),
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a Context<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let area = layout_info.area;
        let frame = Area {
            left: area.left + 8.0,
            top: area.top + 8.0,
            width: area.width - 16.0,
            height: 188.0,
        };
        layout.add_rectangle(
            frame,
            CornerDiameter::uniform(0.0),
            Color::rgb_u8(13, 19, 16),
            Color::TRANSPARENT,
            ShadowPadding::uniform(0.0),
        );

        let longest = self.data.map_width.max(self.data.map_height).max(1) as f32;
        let map_width = 182.0 * self.data.map_width as f32 / longest;
        let map_height = 182.0 * self.data.map_height as f32 / longest;
        let map_area = Area {
            left: frame.left + (frame.width - map_width) / 2.0,
            top: frame.top + (frame.height - map_height) / 2.0,
            width: map_width,
            height: map_height,
        };
        let cell_width = map_area.width / self.data.columns.max(1) as f32;
        let cell_height = map_area.height / self.data.rows.max(1) as f32;
        for run in &self.data.runs {
            layout.add_rectangle(
                Area {
                    left: map_area.left + run.x as f32 * cell_width,
                    top: map_area.top + run.y as f32 * cell_height,
                    width: run.width as f32 * cell_width + 0.2,
                    height: cell_height + 0.2,
                },
                CornerDiameter::uniform(0.0),
                Self::palette(run.palette),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
        }

        if let Some(player) = state.try_get(&this_player()) {
            let position = player.get_common().tile_position;
            let x = (position.x as f32 + 0.5) / self.data.map_width.max(1) as f32;
            let y = 1.0 - (position.y as f32 + 0.5) / self.data.map_height.max(1) as f32;
            if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
                let marker = Area {
                    left: map_area.left + x * map_area.width - 4.0,
                    top: map_area.top + y * map_area.height - 4.0,
                    width: 8.0,
                    height: 8.0,
                };
                layout.add_rectangle(
                    marker,
                    CornerDiameter::uniform(1.0),
                    Color::BLACK,
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
                layout.add_rectangle(
                    Area {
                        left: marker.left + 2.0,
                        top: marker.top + 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    CornerDiameter::uniform(1.0),
                    Color::rgb_u8(255, 206, 70),
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
            }
        }

        layout.add_text(
            Area {
                left: area.left + 8.0,
                top: frame.top + frame.height + 2.0,
                width: area.width - 16.0,
                height: 18.0,
            },
            &self.position_text,
            FontSize(12.0),
            Color::WHITE,
            Color::WHITE,
            HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
    }
}
