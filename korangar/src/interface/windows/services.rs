use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element, ElementBox};
use korangar_interface::event::{ClickHandler, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{MouseButton, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::Context;

use super::WindowClass;
use super::themed_panel::{PanelStyle, ThemedPanel};
use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::input::InputEvent;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::inventory::InventoryPathExt;
use crate::state::services::{ServiceAction, ServicesStatePathExt};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

pub struct MailboxWindow {
    frame: Arc<Texture>,
}
impl MailboxWindow {
    pub fn new(frame: Arc<Texture>) -> Self {
        Self { frame }
    }
}

impl CustomWindow<ClientState> for MailboxWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Mailbox)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "", class: Self::window_class(), theme: InterfaceThemeType::InGame,
            minimum_width: 590.0, maximum_width: 590.0, minimum_height: 630.0, maximum_height: 700.0,
            title_height: 0.0, title_gap: 0.0, border: 0.0, gaps: 0.0, closable: false, resizable: false,
            background_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
            elements: ThemedPanel::new(self.frame,
                PanelStyle { header: 78.0, inset: 40.0, footer: 38.0, minimum_height: 630.0,
                    title_left: 70.0, title_width: 450.0, close_right: 8.0, close_top: 30.0 },
                |_| "Mailbox (Alt+L)".to_owned(),
                ErasedElement::new((
                    split! { children: (
                        button! { text: "Refresh", event: InputEvent::GameService { action: ServiceAction::RefreshMail } },
                        button! { text: "Previous", event: InputEvent::GameService { action: ServiceAction::MailPage { next: false } } },
                        button! { text: "Next", event: InputEvent::GameService { action: ServiceAction::MailPage { next: true } } },
                    ) },
                    MailList::default(),
                    text! { text: client_state().services().mail_notice(), height: 32.0, overflow_behavior: OverflowBehavior::LineBreak },
                    scroll_view! { height: 146.0, children: text! { text: client_state().services().mail_body(), overflow_behavior: OverflowBehavior::LineBreak } },
                    split! { children: (
                        button! { text: "Collect Zeny", event: InputEvent::GameService { action: ServiceAction::ClaimZeny } },
                        button! { text: "Collect Items", event: InputEvent::GameService { action: ServiceAction::ClaimItems } },
                        button! { text: "Delete Empty Mail", event: InputEvent::GameService { action: ServiceAction::DeleteMail } },
                    ) },
                )),
            ),
        }
    }
}

struct ServiceClick(ServiceAction);
impl ClickHandler<ClientState> for ServiceClick {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(InputEvent::GameService { action: self.0.clone() });
    }
}
#[derive(Default)]
struct MailList {
    rows: Vec<(String, ServiceClick, bool)>,
}
impl Element<ClientState> for MailList {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        let path = client_state().services();
        let services = state.get(&path);
        self.rows = services
            .mail
            .iter()
            .skip(services.mail_page * 7)
            .take(7)
            .map(|mail| {
                (
                    format!("{}{} — {}", if mail.read { "" } else { "[New] " }, mail.sender, mail.title),
                    ServiceClick(ServiceAction::ReadMail {
                        inbox: mail.inbox,
                        id: mail.id,
                    }),
                    services.mail_selected_id == mail.id,
                )
            })
            .collect();
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(224.0),
        })
    }

    fn lay_out<'a>(
        &'a self,
        _: &'a Context<ClientState>,
        _: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        for (index, (text, handler, selected)) in self.rows.iter().enumerate() {
            let area = Area {
                top: info.area.top + index as f32 * 32.0,
                height: 30.0,
                ..info.area
            };
            let hovered = area.check().run(layout);
            if hovered {
                layout.register_click_handler(MouseButton::Left, handler);
            }
            if hovered || *selected {
                layout.add_rectangle(
                    area,
                    CornerDiameter::uniform(3.0),
                    Color::rgba_u8(106, 27, 39, 150),
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
            }
            layout.add_text(
                area,
                text,
                FontSize(12.0),
                Color::rgb_u8(237, 223, 204),
                Color::WHITE,
                HorizontalAlignment::Left { offset: 8.0, border: 0.0 },
                VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );
        }
    }
}

struct PetRenameInput;
pub struct PetsWindow {
    frame: Arc<Texture>,
}
impl PetsWindow {
    pub fn new(frame: Arc<Texture>) -> Self {
        Self { frame }
    }
}
impl CustomWindow<ClientState> for PetsWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Pets)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "", class: Self::window_class(), theme: InterfaceThemeType::InGame,
            minimum_width: 430.0, maximum_width: 430.0, minimum_height: 380.0,
            title_height: 0.0, title_gap: 0.0, border: 0.0, gaps: 0.0, closable: false, resizable: false,
            background_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
            elements: ThemedPanel::new(self.frame,
                PanelStyle { header: 48.0, inset: 28.0, footer: 30.0, minimum_height: 280.0,
                    title_left: 50.0, title_width: 330.0, close_right: 8.0, close_top: 13.0 },
                |_| "Companion (Alt+J)".to_owned(),
                ErasedElement::new((
                    text! { text: client_state().services().pet_description(), height: 64.0, overflow_behavior: OverflowBehavior::LineBreak },
                    split! { children: (
                        button! { text: "Refresh", event: InputEvent::GameService { action: ServiceAction::PetMenu { action: 0 } } },
                        button! { text: "Feed", event: InputEvent::GameService { action: ServiceAction::PetMenu { action: 1 } } },
                        button! { text: "Perform", event: InputEvent::GameService { action: ServiceAction::PetMenu { action: 2 } } },
                    ) },
                    button! { text: "Return to Egg", event: InputEvent::GameService { action: ServiceAction::PetMenu { action: 3 } } },
                    button! { text: "Unequip Accessory", event: InputEvent::GameService { action: ServiceAction::PetMenu { action: 4 } } },
                    button! { text: "Toggle Auto-feed (VIP)", event: InputEvent::GameService { action: ServiceAction::TogglePetAutoFeed } },
                    text_box! { ghost_text: "New companion name", state: client_state().services().pet_new_name(),
                        input_handler: korangar_interface::components::text_box::DefaultHandler::<_, _, 24>::new(client_state().services().pet_new_name(), |_: &Context<ClientState>, queue: &mut EventQueue<ClientState>| { queue.queue(InputEvent::GameService { action: ServiceAction::RenamePet }); }), focus_id: PetRenameInput },
                    button! { text: "Rename Companion", event: InputEvent::GameService { action: ServiceAction::RenamePet } },
                    text! { text: "Double-click a pet accessory in Inventory to equip it.", height: 32.0, overflow_behavior: OverflowBehavior::LineBreak },
                    scroll_view! { height: 120.0, children: PetEggList::default() },
                )),
            ),
        }
    }
}

#[derive(Default)]
struct PetEggList {
    rows: Vec<(String, ServiceClick)>,
}
impl Element<ClientState> for PetEggList {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        let path = client_state().services();
        let inventory_path = client_state().inventory().items();
        let inventory = state.get(&inventory_path);
        self.rows = state
            .get(&path)
            .pet_eggs
            .iter()
            .map(|index| {
                let name = inventory
                    .iter()
                    .find(|item| item.index.0 + 2 == *index)
                    .map(|item| item.metadata.name.as_str())
                    .unwrap_or("Companion egg");
                (
                    format!("Hatch {}", name),
                    ServiceClick(ServiceAction::HatchPet { index: *index }),
                )
            })
            .collect();
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(self.rows.len() as f32 * 26.0),
        })
    }

    fn lay_out<'a>(
        &'a self,
        _: &'a Context<ClientState>,
        _: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        for (index, (text, handler)) in self.rows.iter().enumerate() {
            let area = Area {
                top: info.area.top + index as f32 * 26.0,
                height: 25.0,
                ..info.area
            };
            if area.check().run(layout) {
                layout.register_click_handler(MouseButton::Left, handler);
            }
            layout.add_text(
                area,
                text,
                FontSize(12.0),
                Color::rgb_u8(238, 215, 173),
                Color::WHITE,
                HorizontalAlignment::Left { offset: 8.0, border: 0.0 },
                VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );
        }
    }
}

pub struct SocialServicesWindow<const KIND: u8> {
    frame: Arc<Texture>,
}
pub type SocialWindow = SocialServicesWindow<0>;
pub type GuildWindow = SocialServicesWindow<1>;
pub type TradeWindow = SocialServicesWindow<2>;
impl<const KIND: u8> SocialServicesWindow<KIND> {
    pub fn new(frame: Arc<Texture>) -> Self {
        Self { frame }
    }
}
impl<const KIND: u8> CustomWindow<ClientState> for SocialServicesWindow<KIND> {
    fn window_class() -> Option<WindowClass> {
        Some(match KIND {
            0 => WindowClass::Social,
            1 => WindowClass::Guild,
            _ => WindowClass::Trade,
        })
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::components::text_box::DefaultHandler;
        use korangar_interface::prelude::*;
        struct PlayerNameInput;
        struct GroupNameInput;
        struct GuildMessageInput;
        struct TradeAmountInput;
        struct TradeZenyInput;
        let body: ElementBox<ClientState> = match KIND {
            0 => ErasedElement::new((
                text! { text: "Friends, whispers, and party management", height: 24.0 },
                button! { text: "Friends (Alt+Z)", event: InputEvent::ToggleFriendListWindow },
                text_box! { ghost_text: "Player name", state: client_state().services().social_name(),
                input_handler: DefaultHandler::<_, _, 24>::new(client_state().services().social_name(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: PlayerNameInput },
                text_box! { ghost_text: "Party name", state: client_state().services().group_name(),
                input_handler: DefaultHandler::<_, _, 24>::new(client_state().services().group_name(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: GroupNameInput },
                split! { children: (
                    button! { text: "Create Party", event: InputEvent::GameService { action: ServiceAction::CreateParty } },
                    button! { text: "Invite Player", event: InputEvent::GameService { action: ServiceAction::InviteParty } },
                    button! { text: "Leave Party", event: InputEvent::GameService { action: ServiceAction::LeaveParty } },
                ) },
                text! { text: client_state().party_name(), height: 26.0 },
                text! { text: client_state().services().social_notice(), height: 74.0, overflow_behavior: OverflowBehavior::LineBreak },
                split! { children: (
                    button! { text: "Accept Party Invite", event: InputEvent::GameService { action: ServiceAction::ReplyParty { accept: true } } },
                    button! { text: "Decline", event: InputEvent::GameService { action: ServiceAction::ReplyParty { accept: false } } },
                ) },
            )),
            1 => ErasedElement::new((
                text! { text: client_state().services().guild_description(), height: 58.0, overflow_behavior: OverflowBehavior::LineBreak },
                text_box! { ghost_text: "Guild name (create) or player name (invite)", state: client_state().services().social_name(),
                input_handler: DefaultHandler::<_, _, 24>::new(client_state().services().social_name(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: PlayerNameInput },
                split! { children: (
                    button! { text: "Create Guild", event: InputEvent::GameService { action: ServiceAction::CreateGuild } },
                    button! { text: "Invite", event: InputEvent::GameService { action: ServiceAction::InviteGuild } },
                    button! { text: "Refresh", event: InputEvent::GameService { action: ServiceAction::RefreshGuild } },
                ) },
                scroll_view! { height: 170.0, children: text! { text: client_state().services().guild_roster(), overflow_behavior: OverflowBehavior::LineBreak } },
                text_box! { ghost_text: "Message your guild", state: client_state().services().guild_message(),
                input_handler: DefaultHandler::<_, _, 120>::new(client_state().services().guild_message(), |_: &Context<ClientState>, queue: &mut EventQueue<ClientState>| { queue.queue(InputEvent::GameService { action: ServiceAction::GuildChat }); }), focus_id: GuildMessageInput },
                button! { text: "Send Guild Message", event: InputEvent::GameService { action: ServiceAction::GuildChat } },
                text! { text: client_state().services().social_notice(), height: 50.0, overflow_behavior: OverflowBehavior::LineBreak },
                split! { children: (
                    button! { text: "Accept Guild Invite", event: InputEvent::GameService { action: ServiceAction::ReplyGuild { accept: true } } },
                    button! { text: "Decline", event: InputEvent::GameService { action: ServiceAction::ReplyGuild { accept: false } } },
                    button! { text: "Leave Guild", event: InputEvent::GameService { action: ServiceAction::LeaveGuild } },
                ) },
            )),
            _ => ErasedElement::new((
                text! { text: client_state().services().trade_balance(), height: 22.0 },
                text_box! { ghost_text: "Nearby player name", state: client_state().services().social_name(),
                input_handler: DefaultHandler::<_, _, 24>::new(client_state().services().social_name(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: PlayerNameInput },
                split! { children: (
                    button! { text: "Request Trade", event: InputEvent::GameService { action: ServiceAction::RequestTrade } },
                    button! { text: "Accept", event: InputEvent::GameService { action: ServiceAction::ReplyTrade { accept: true } } },
                    button! { text: "Decline", event: InputEvent::GameService { action: ServiceAction::ReplyTrade { accept: false } } },
                ) },
                text! { text: client_state().services().trade_notice(), height: 58.0, overflow_behavior: OverflowBehavior::LineBreak },
                scroll_view! { height: 96.0, children: TradeInventory::default() },
                split! { children: (
                    text_box! { ghost_text: "Quantity", state: client_state().services().trade_amount(), input_handler: DefaultHandler::<_, _, 6>::new(client_state().services().trade_amount(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: TradeAmountInput },
                    button! { text: "Add Item", event: InputEvent::GameService { action: ServiceAction::AddTradeItem } },
                ) },
                split! { children: (
                    text_box! { ghost_text: "Zeny", state: client_state().services().trade_zeny_input(), input_handler: DefaultHandler::<_, _, 10>::new(client_state().services().trade_zeny_input(), |_: &Context<ClientState>, _: &mut EventQueue<ClientState>| {}), focus_id: TradeZenyInput },
                    button! { text: "Offer Zeny", event: InputEvent::GameService { action: ServiceAction::AddTradeZeny } },
                ) },
                text! { text: "Your offer", height: 22.0 },
                scroll_view! { height: 94.0, children: text! { text: client_state().services().trade_own_offer(), overflow_behavior: OverflowBehavior::LineBreak } },
                text! { text: "Their offer", height: 22.0 },
                scroll_view! { height: 94.0, children: text! { text: client_state().services().trade_other_offer(), overflow_behavior: OverflowBehavior::LineBreak } },
                split! { children: (
                    button! { text: "Lock Offer", event: InputEvent::GameService { action: ServiceAction::LockTrade } },
                    button! { text: "Confirm Trade", event: InputEvent::GameService { action: ServiceAction::ConfirmTrade } },
                    button! { text: "Cancel", event: InputEvent::GameService { action: ServiceAction::CancelTrade } },
                ) },
            )),
        };
        window! {
            title: "", class: Self::window_class(), theme: InterfaceThemeType::InGame,
            minimum_width: 540.0, maximum_width: 540.0, minimum_height: 610.0, maximum_height: 760.0,
            title_height: 0.0, title_gap: 0.0, border: 0.0, gaps: 0.0, closable: false, resizable: false,
            background_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
            elements: ThemedPanel::new(self.frame,
                PanelStyle { header: 78.0, inset: 36.0, footer: 40.0, minimum_height: 610.0,
                    title_left: 65.0, title_width: 410.0, close_right: 8.0, close_top: 30.0 },
                |_| match KIND { 0 => "Social (Alt+P)", 1 => "Guild (Alt+G)", _ => "Trade (Alt+T)" }.to_owned(), body),
        }
    }
}

#[derive(Default)]
struct TradeInventory {
    rows: Vec<(String, ServiceClick)>,
}
impl Element<ClientState> for TradeInventory {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        let path = client_state().inventory().items();
        self.rows = state
            .get(&path)
            .iter()
            .filter(|item| match &item.details {
                korangar_networking::InventoryItemDetails::Equippable { equipped_position, .. }
                | korangar_networking::InventoryItemDetails::Regular { equipped_position, .. } => equipped_position.is_empty(),
            })
            .map(|item| {
                (
                    item.metadata.name.clone(),
                    ServiceClick(ServiceAction::SelectTradeItem { index: item.index.0 + 2 }),
                )
            })
            .collect();
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(self.rows.len() as f32 * 24.0),
        })
    }

    fn lay_out<'a>(
        &'a self,
        _: &'a Context<ClientState>,
        _: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        for (index, (name, handler)) in self.rows.iter().enumerate() {
            let area = Area {
                top: info.area.top + index as f32 * 24.0,
                height: 23.0,
                ..info.area
            };
            if area.check().run(layout) {
                layout.register_click_handler(MouseButton::Left, handler);
            }
            layout.add_text(
                area,
                name,
                FontSize(12.0),
                Color::rgb_u8(237, 219, 191),
                Color::WHITE,
                HorizontalAlignment::Left { offset: 6.0, border: 0.0 },
                VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );
        }
    }
}
