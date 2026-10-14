use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::event::{ClickHandler, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{MouseButton, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::{AccountId, JobId};
use rust_state::Context;

use super::WindowClass;
use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::input::InputEvent;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state, this_player};
use crate::world::{JobIdentity, Library};

const MAX_PARTY_MEMBERS: usize = 12;

pub struct PartyWindow { library: Arc<Library> }

impl PartyWindow {
    pub fn new(library: Arc<Library>) -> Self { Self { library } }
}

impl CustomWindow<ClientState> for PartyWindow {
    fn window_class() -> Option<WindowClass> { Some(WindowClass::Party) }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: client_state().party_name(),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 270.0,
            maximum_width: 270.0,
            closable: false,
            elements: PartyBody::new(self.library),
        }
    }
}

struct SelectMember {
    index: usize,
    selected: Rc<Cell<Option<AccountId>>>,
}

impl ClickHandler<ClientState> for SelectMember {
    fn handle_click(&self, state: &Context<ClientState>, _: &mut EventQueue<ClientState>) {
        if let Some(member) = state.get(&client_state().party_members()).get(self.index) {
            self.selected.set(Some(member.account_id));
        }
    }
}

struct MemberAction {
    selected: Rc<Cell<Option<AccountId>>>,
    pass_leadership: bool,
}

impl ClickHandler<ClientState> for MemberAction {
    fn handle_click(&self, state: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        let Some(selected_id) = self.selected.get() else { return };
        let player_path = this_player();
        let Some(player) = state.try_get(&player_path) else { return };
        let self_id = AccountId(player.get_common().entity_id.0);
        if !state.get(&client_state().party_members()).iter().any(|member| member.account_id == self_id && member.leader) {
            return;
        }
        let members_path = client_state().party_members();
        let Some(member) = state.get(&members_path).iter().find(|member| member.account_id == selected_id) else { return };
        if selected_id == self_id { return; }
        if self.pass_leadership {
            queue.queue(InputEvent::ChangePartyLeader { account_id: selected_id });
        } else {
            queue.queue(InputEvent::RemovePartyMember { account_id: selected_id, name: member.name.clone() });
        }
        self.selected.set(None);
    }
}

struct PartyBody {
    library: Arc<Library>,
    selected: Rc<Cell<Option<AccountId>>>,
    row_handlers: Vec<SelectMember>,
    leader_handler: MemberAction,
    remove_handler: MemberAction,
    labels: Vec<String>,
    visible_indices: Vec<usize>,
}

impl PartyBody {
    fn new(library: Arc<Library>) -> Self {
        let selected = Rc::new(Cell::new(None));
        Self {
            library,
            row_handlers: (0..MAX_PARTY_MEMBERS).map(|index| SelectMember { index, selected: selected.clone() }).collect(),
            leader_handler: MemberAction { selected: selected.clone(), pass_leadership: true },
            remove_handler: MemberAction { selected: selected.clone(), pass_leadership: false },
            selected,
            labels: Vec::new(),
            visible_indices: Vec::new(),
        }
    }
}

impl Element<ClientState> for PartyBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, state: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        let self_id = state.try_get(&this_player()).map(|player| AccountId(player.get_common().entity_id.0));
        let members_path = client_state().party_members();
        let members = state.get(&members_path);
        self.visible_indices = members.iter().enumerate()
            .filter(|(_, member)| Some(member.account_id) != self_id)
            .take(MAX_PARTY_MEMBERS - 1).map(|(index, _)| index).collect();
        self.labels = self.visible_indices.iter().map(|&index| {
            let member = &members[index];
            let status = if member.online { format!("Nv {}", member.base_level) } else { "Offline".to_owned() };
            let raw_job = self.library.get::<JobIdentity>(JobId(member.job_id as u16)).to_string();
            let job = if raw_job == "1_f_maria" { format!("Classe {}", member.job_id) } else { raw_job.replace('_', " ") };
            format!("{}  ·  {}  ·  {}", member.name, job, status)
        }).collect();
        let actions = self.selected.get().is_some() && self.visible_indices.iter().any(|&index| {
            members[index].account_id == self.selected.get().unwrap()
        });
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo {
            area: resolver.with_height(self.visible_indices.len() as f32 * 42.0 + if actions { 58.0 } else { 0.0 } + 4.0),
        })
    }

    fn lay_out<'a>(&'a self, state: &'a Context<ClientState>, _: ElementStore<'a>, info: &'a Self::LayoutInfo, layout: &mut WindowLayout<'a, ClientState>) {
        let area = info.area;
        let members_path = client_state().party_members();
        let members = state.get(&members_path);
        for (row_index, (&member_index, label)) in self.visible_indices.iter().zip(self.labels.iter()).enumerate() {
            let member = &members[member_index];
            let row = Area { left: area.left + 4.0, top: area.top + row_index as f32 * 42.0, width: area.width - 8.0, height: 38.0 };
            let hovered = row.check().run(layout);
            let selected = self.selected.get() == Some(member.account_id);
            layout.add_rectangle(row, CornerDiameter::uniform(3.0),
                if selected { Color::rgba_u8(61, 45, 71, 235) } else if hovered { Color::rgba_u8(48, 42, 57, 225) } else { Color::rgba_u8(21, 21, 31, 215) },
                Color::rgb_u8(103, 87, 93), ShadowPadding::uniform(0.0));
            layout.add_text(Area { left: row.left + 6.0, top: row.top + 2.0, width: row.width - 12.0, height: 18.0 }, label,
                FontSize(13.0), Color::rgb_u8(238, 226, 208), Color::WHITE,
                HorizontalAlignment::Left { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
            let bar = Area { left: row.left + 6.0, top: row.top + 24.0, width: row.width - 12.0, height: 8.0 };
            layout.add_rectangle(bar, CornerDiameter::uniform(2.0), Color::rgb_u8(40, 34, 43), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
            let ratio = if member.maximum_health <= 0 { 0.0 } else { (member.health as f32 / member.maximum_health as f32).clamp(0.0, 1.0) };
            if ratio > 0.0 {
                layout.add_rectangle(Area { width: bar.width * ratio, ..bar }, CornerDiameter::uniform(2.0), Color::rgb_u8(194, 63, 72), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
            }
            if hovered { layout.register_click_handler(MouseButton::Left, &self.row_handlers[member_index]); }
        }

        if let Some(selected_id) = self.selected.get() {
            if self.visible_indices.iter().any(|&index| members[index].account_id == selected_id) {
                let top = area.top + self.visible_indices.len() as f32 * 42.0;
                let self_id = state.try_get(&this_player()).map(|player| AccountId(player.get_common().entity_id.0));
                let leader = members.iter().any(|member| Some(member.account_id) == self_id && member.leader);
                let labels = ["Passar liderança", "Remover do grupo"];
                let handlers: [&dyn ClickHandler<ClientState>; 2] = [&self.leader_handler, &self.remove_handler];
                for index in 0..2 {
                    let button = Area { left: area.left + 4.0, top: top + index as f32 * 27.0, width: area.width - 8.0, height: 24.0 };
                    let hovered = button.check().run(layout);
                    layout.add_rectangle(button, CornerDiameter::uniform(3.0),
                        if leader && hovered { Color::rgb_u8(76, 57, 72) } else { Color::rgb_u8(34, 34, 43) },
                        Color::rgb_u8(119, 99, 93), ShadowPadding::uniform(0.0));
                    layout.add_text(button, labels[index], FontSize(13.0),
                        if leader { Color::WHITE } else { Color::rgb_u8(118, 115, 120) }, Color::WHITE,
                        HorizontalAlignment::Center { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
                    if leader && hovered { layout.register_click_handler(MouseButton::Left, handlers[index]); }
                }
            }
        }
    }
}
