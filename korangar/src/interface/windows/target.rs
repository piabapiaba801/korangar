use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::Context;

use super::WindowClass;
use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::world::EntityType;

pub struct TargetWindow;

impl CustomWindow<ClientState> for TargetWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Target)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Alvo",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 320.0,
            maximum_width: 320.0,
            closable: false,
            elements: TargetBody::default(),
        }
    }
}

#[derive(Default)]
struct TargetBody {
    name: String,
    health: String,
    ratio: f32,
}

impl Element<ClientState> for TargetBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, state: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        self.name.clear();
        self.health.clear();
        self.ratio = 0.0;
        if let Some(id) = *state.get(&client_state().selected_target()) {
            if let Some(entity) = state.get(&client_state().entities()).iter().find(|entity| entity.get_entity_id() == id && entity.get_entity_type() == EntityType::Monster) {
                self.name = entity.get_details().map(|details| details.split('#').next().unwrap_or(details).to_owned()).unwrap_or_else(|| "Monstro".to_owned());
                let (current, maximum) = entity.health_values();
                self.health = format!("{current} / {maximum}");
                self.ratio = if maximum == 0 { 0.0 } else { (current as f32 / maximum as f32).clamp(0.0, 1.0) };
            }
        }
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo { area: resolver.with_height(48.0) })
    }

    fn lay_out<'a>(&'a self, _: &'a Context<ClientState>, _: ElementStore<'a>, info: &'a Self::LayoutInfo, layout: &mut WindowLayout<'a, ClientState>) {
        let area = info.area;
        layout.add_text(
            Area { left: area.left, top: area.top, width: area.width, height: 18.0 },
            &self.name,
            FontSize(14.0),
            Color::rgb_u8(240, 228, 208),
            Color::rgb_u8(240, 228, 208),
            HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
        let bar = Area { left: area.left + 4.0, top: area.top + 22.0, width: area.width - 8.0, height: 20.0 };
        layout.add_rectangle(bar, CornerDiameter::uniform(3.0), Color::rgb_u8(34, 22, 29), Color::rgb_u8(112, 82, 74), ShadowPadding::uniform(0.0));
        if self.ratio > 0.0 {
            layout.add_rectangle(
                Area { left: bar.left + 2.0, top: bar.top + 2.0, width: (bar.width - 4.0) * self.ratio, height: bar.height - 4.0 },
                CornerDiameter::uniform(2.0),
                Color::rgb_u8(191, 43, 55),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
        }
        layout.add_text(
            bar,
            &self.health,
            FontSize(12.0),
            Color::WHITE,
            Color::WHITE,
            HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
    }
}
