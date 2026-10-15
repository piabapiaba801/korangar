use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::Context;

use super::WindowClass;
use super::themed_panel::{PanelStyle, ThemedPanel};
use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

pub struct QuestTrackerWindow {
    frame: Arc<Texture>,
}

impl QuestTrackerWindow {
    pub fn new(frame: Arc<Texture>) -> Self { Self { frame } }
}

impl CustomWindow<ClientState> for QuestTrackerWindow {
    fn window_class() -> Option<WindowClass> { Some(WindowClass::QuestTracker) }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 284.0,
            maximum_width: 284.0,
            minimum_height: 114.0,
            maximum_height: 500.0,
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
                PanelStyle { header: 41.0, inset: 18.0, footer: 13.0, minimum_height: 114.0,
                    title_left: 44.0, title_width: 196.0, close_right: 6.0, close_top: 13.0 },
                |_| "Missões".to_owned(),
                ErasedElement::new(QuestBody::default()),
            ),
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
                layout.add_rectangle(row, CornerDiameter::uniform(2.0), Color::rgba_u8(84, 28, 37, 170), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
            }
            layout.add_text(row, line, FontSize(13.0),
                if *heading { Color::rgb_u8(239, 202, 113) } else { Color::rgb_u8(220, 219, 205) },
                Color::WHITE, HorizontalAlignment::Left { offset: 5.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
        }
    }
}
