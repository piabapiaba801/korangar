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

pub struct QuestTrackerWindow;

impl CustomWindow<ClientState> for QuestTrackerWindow {
    fn window_class() -> Option<WindowClass> { Some(WindowClass::QuestTracker) }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "Missões",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 284.0,
            maximum_width: 284.0,
            closable: true,
            elements: QuestBody::default(),
        }
    }
}

#[derive(Default)]
struct QuestBody { lines: Vec<(String, bool)> }

impl Element<ClientState> for QuestBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, state: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        self.lines.clear();
        for quest in state.get(&client_state().quests()).iter().take(6) {
            self.lines.push((format!("Missão #{}", quest.quest_id), true));
            if quest.objective_details.is_empty() { self.lines.push(("Em andamento".to_owned(), false)); }
            for detail in quest.objective_details.iter().take(3) {
                let name = if detail.mob_name.is_empty() { "Objetivo" } else { &detail.mob_name };
                let progress = if detail.total_count > 0 { format!(" {}/{}", detail.kill_count, detail.total_count) } else { String::new() };
                self.lines.push((format!("  {}{}", name, progress), false));
            }
        }
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo { area: resolver.with_height(self.lines.len() as f32 * 22.0 + 8.0) })
    }

    fn lay_out<'a>(&'a self, _: &'a Context<ClientState>, _: ElementStore<'a>, info: &'a Self::LayoutInfo, layout: &mut WindowLayout<'a, ClientState>) {
        let area = info.area;
        for (index, (line, heading)) in self.lines.iter().enumerate() {
            let row = Area { left: area.left + 4.0, top: area.top + 4.0 + index as f32 * 22.0, width: area.width - 8.0, height: 20.0 };
            if *heading {
                layout.add_rectangle(row, CornerDiameter::uniform(2.0), Color::rgba_u8(47, 39, 45, 220), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
            }
            layout.add_text(row, line, FontSize(13.0),
                if *heading { Color::rgb_u8(239, 202, 113) } else { Color::rgb_u8(220, 219, 205) },
                Color::WHITE, HorizontalAlignment::Left { offset: 5.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
        }
    }
}
