use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::event::{ClickHandler, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Icon, MouseButton, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::Context;

use crate::input::InputEvent;
use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::interface::windows::WindowClass;
use super::themed_panel::{PanelStyle, ThemedPanel};
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state, this_player};
use crate::world::{EntityType, MiniMapData};

pub struct MiniMapWindow {
    map_name: String,
    data: MiniMapData,
    frame: Arc<Texture>,
}

impl MiniMapWindow {
    pub fn new(map_name: String, data: MiniMapData, frame: Arc<Texture>) -> Self {
        Self { map_name, data, frame }
    }
}

impl CustomWindow<ClientState> for MiniMapWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::MiniMap)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 224.0,
            maximum_width: 224.0,
            minimum_height: 300.0,
            maximum_height: 300.0,
            title_height: 0.0,
            title_gap: 0.0,
            border: 0.0,
            gaps: 0.0,
            closable: false,
            resizable: false,
            background_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
            elements: ThemedPanel::new(
                self.frame,
                PanelStyle { header: 48.0, inset: 16.0, footer: 6.0, minimum_height: 300.0,
                    title_left: 45.0, title_width: 134.0, close_right: 6.0, close_top: 17.0 },
                |_| "Minimap".to_owned(),
                ErasedElement::new((MiniMapBody::new(self.map_name, self.data, false), WorldMapButton)),
            ),
        }
    }
}

struct WorldMapButton;

impl ClickHandler<ClientState> for WorldMapButton {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(InputEvent::ToggleWorldMapWindow);
    }
}

impl Element<ClientState> for WorldMapButton {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, _: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo { area: resolver.with_height(26.0) })
    }

    fn lay_out<'a>(&'a self, _: &'a Context<ClientState>, _: ElementStore<'a>, info: &'a Self::LayoutInfo, layout: &mut WindowLayout<'a, ClientState>) {
        let hovered = info.area.check().run(layout);
        if hovered {
            layout.register_click_handler(MouseButton::Left, self);
        }
        layout.add_text(
            Area { width: info.area.width - 35.0, ..info.area },
            "Mapa mundi",
            FontSize(12.0),
            if hovered { Color::rgb_u8(255, 236, 215) } else { Color::rgb_u8(220, 207, 194) },
            Color::rgb_u8(255, 191, 116),
            HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
        layout.add_icon(
            Area { left: info.area.left + info.area.width - 25.0, top: info.area.top + 7.0, width: 11.0, height: 11.0 },
            Icon::ArrowRight,
            if hovered { Color::rgb_u8(255, 227, 205) } else { Color::rgb_u8(224, 200, 181) },
        );
    }
}

pub struct WorldMapWindow {
    map_name: String,
    data: MiniMapData,
}

impl WorldMapWindow {
    pub fn new(map_name: String, data: MiniMapData) -> Self { Self { map_name, data } }
}

impl CustomWindow<ClientState> for WorldMapWindow {
    fn window_class() -> Option<WindowClass> { Some(WindowClass::WorldMap) }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "Mapa mundi",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 560.0,
            maximum_width: 560.0,
            closable: true,
            elements: MiniMapBody::new(self.map_name, self.data, true),
        }
    }
}

struct MiniMapBody {
    map_name: String,
    data: MiniMapData,
    position_text: String,
    expanded: bool,
}

impl MiniMapBody {
    fn new(map_name: String, data: MiniMapData, expanded: bool) -> Self {
        Self {
            map_name,
            data,
            position_text: String::new(),
            expanded,
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
            area: resolver.with_height(if self.expanded { 510.0 } else { 220.0 }),
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
            height: if self.expanded { 482.0 } else { 170.0 },
        };
        layout.add_rectangle(
            frame,
            CornerDiameter::uniform(0.0),
            Color::rgb_u8(13, 19, 16),
            Color::TRANSPARENT,
            ShadowPadding::uniform(0.0),
        );

        let longest = self.data.map_width.max(self.data.map_height).max(1) as f32;
        let map_size = if self.expanded { 476.0 } else { (frame.width - 6.0).min(166.0) };
        let map_width = map_size * self.data.map_width as f32 / longest;
        let map_height = map_size * self.data.map_height as f32 / longest;
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

        // Static script warps cover the full map. Visible server warp entities
        // also cover portals created or moved while the map is running.
        let mut portals = self.data.portals.clone();
        for entity in state
            .get(&client_state().entities())
            .iter()
            .filter(|entity| entity.get_entity_type() == EntityType::Warp)
        {
            let position = entity.get_tile_position();
            if !portals.contains(&position) {
                portals.push(position);
            }
        }
        for position in portals {
            let x = (position.x as f32 + 0.5) / self.data.map_width.max(1) as f32;
            let y = 1.0 - (position.y as f32 + 0.5) / self.data.map_height.max(1) as f32;
            if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
                continue;
            }
            let marker = Area {
                left: (map_area.left + x * map_area.width - 3.0).clamp(map_area.left, map_area.left + map_area.width - 6.0),
                top: (map_area.top + y * map_area.height - 3.0).clamp(map_area.top, map_area.top + map_area.height - 6.0),
                width: 6.0,
                height: 6.0,
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
                    left: marker.left + 1.0,
                    top: marker.top + 1.0,
                    width: 4.0,
                    height: 4.0,
                },
                CornerDiameter::uniform(1.0),
                Color::rgb_u8(55, 222, 155),
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
                top: frame.top + frame.height + 12.0,
                width: area.width - 16.0,
                height: 16.0,
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
