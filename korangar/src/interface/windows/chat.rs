use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use korangar_interface::MouseMode;
use korangar_interface::application::Size;
use korangar_interface::components::text_box::DefaultHandler;
use korangar_interface::element::id::FocusIdExt;
use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{Element, ElementBox, StateElement};
use korangar_interface::event::{ClickHandler, Event, EventQueue, ScrollHandler};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::tooltip::TooltipExt;
use korangar_interface::layout::{Icon, MouseButton, Resolver, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::prelude::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::window::{CustomWindow, Window};
use korangar_networking::MessageColor;
use rust_state::{Context, Path, RustState};

use super::WindowClass;
use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::input::InputEvent;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ChatChannel, ChatMessage, ClientState, ClientStatePathExt, client_state};

const MAXIMUM_CHAT_MESSAGE_LENGTH: usize = 80;
const CHAT_WIDTH: f32 = 440.0;
const CHAT_HEIGHT: f32 = 260.0;
const TAB_LABELS: [&str; 4] = ["Geral", "Batalha", "Grupo", "Privado"];

/// Used to focus the message field when the player presses Enter.
pub struct ChatTextBox;
struct ChatRecipientBox;
struct SendTooltip;
struct DragTooltip;

fn relative(area: Area, x: f32, y: f32, width: f32, height: f32) -> Area {
    Area {
        left: area.left + x * area.width / CHAT_WIDTH,
        top: area.top + y * area.height / CHAT_HEIGHT,
        width: width * area.width / CHAT_WIDTH,
        height: height * area.height / CHAT_HEIGHT,
    }
}

fn tab_accepts(tab: usize, channel: ChatChannel) -> bool {
    match tab {
        1 => channel == ChatChannel::Battle,
        2 => channel == ChatChannel::Party,
        3 => channel == ChatChannel::Private,
        _ => channel != ChatChannel::Battle,
    }
}

fn message_color(color: &MessageColor) -> Color {
    match *color {
        MessageColor::Rgb { red, green, blue } => Color::rgb_u8(red, green, blue),
        MessageColor::Broadcast => Color::rgb_u8(239, 217, 120),
        MessageColor::Server => Color::rgb_u8(221, 218, 203),
        MessageColor::Error => Color::rgb_u8(246, 117, 117),
        MessageColor::Information => Color::rgb_u8(234, 233, 226),
    }
}

fn text<'a>(layout: &mut WindowLayout<'a, ClientState>, area: Area, value: &'a str, size: f32, color: Color) {
    layout.add_text(
        area,
        value,
        FontSize(size),
        color,
        color,
        HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
        VerticalAlignment::Center { offset: 0.0 },
        OverflowBehavior::Shrink,
    );
}

#[derive(Default)]
struct ChatDrag {
    window_id: Cell<u64>,
}

impl ClickHandler<ClientState> for ChatDrag {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(Event::SetMouseMode {
            mouse_mode: MouseMode::MovingWindow {
                window_id: self.window_id.get(),
            },
        });
    }
}

struct SelectTab<P> {
    path: P,
    index: usize,
}

impl<P: Path<ClientState, usize>> ClickHandler<ClientState> for SelectTab<P> {
    fn handle_click(&self, state: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        state.update_value(self.path, self.index);
        queue.queue(Event::Unfocus);
    }
}

struct HistoryScroll {
    offset: f32,
    maximum: f32,
    following: bool,
    tab: usize,
}

impl Default for HistoryScroll {
    fn default() -> Self {
        Self {
            offset: 0.0,
            maximum: 0.0,
            following: true,
            tab: usize::MAX,
        }
    }
}

impl ScrollHandler<ClientState> for RefCell<HistoryScroll> {
    fn handle_scroll(&self, _: &Context<ClientState>, _: &mut EventQueue<ClientState>, delta: f32) -> bool {
        let mut scroll = self.borrow_mut();
        let previous = scroll.offset;
        scroll.offset = (scroll.offset - delta).clamp(0.0, scroll.maximum);
        scroll.following = scroll.offset >= scroll.maximum - 0.5;
        previous != scroll.offset
    }
}

struct ScrollJump {
    scroll: Rc<RefCell<HistoryScroll>>,
    fraction: Cell<f32>,
}

impl ClickHandler<ClientState> for ScrollJump {
    fn handle_click(&self, _: &Context<ClientState>, _: &mut EventQueue<ClientState>) {
        let mut scroll = self.scroll.borrow_mut();
        scroll.offset = scroll.maximum * self.fraction.get();
        scroll.following = scroll.offset >= scroll.maximum - 0.5;
    }
}

struct MessageLayout {
    index: usize,
    top: f32,
    height: f32,
}

struct HistoryLayout {
    area: Area,
    messages: Vec<MessageLayout>,
    total_height: f32,
    offset: f32,
}

struct ChatHistory<P, T> {
    messages_path: P,
    tab_path: T,
    scroll: Rc<RefCell<HistoryScroll>>,
    jump: ScrollJump,
}

impl<P, T> ChatHistory<P, T> {
    fn new(messages_path: P, tab_path: T) -> Self {
        let scroll = Rc::new(RefCell::new(HistoryScroll::default()));
        Self {
            messages_path,
            tab_path,
            scroll: scroll.clone(),
            jump: ScrollJump {
                scroll,
                fraction: Cell::new(0.0),
            },
        }
    }
}

impl<P, T> Element<ClientState> for ChatHistory<P, T>
where
    P: Path<ClientState, Vec<ChatMessage>>,
    T: Path<ClientState, usize>,
{
    type LayoutInfo = HistoryLayout;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| {
            let tab = *state.get(&self.tab_path);
            let height = resolver.push_available_area().height.unwrap_or(162.0);
            let mut total_height = 0.0;
            let mut messages = Vec::new();
            for (index, message) in state.get(&self.messages_path).iter().enumerate() {
                if !tab_accepts(tab, message.channel) {
                    continue;
                }
                let (size, _) = resolver.get_text_dimensions(
                    &message.text,
                    message_color(&message.color),
                    Color::rgb_u8(255, 191, 116),
                    FontSize(13.0),
                    HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
                    OverflowBehavior::LineBreak,
                );
                if !messages.is_empty() {
                    total_height += 3.0;
                }
                messages.push(MessageLayout {
                    index,
                    top: total_height,
                    height: size.height(),
                });
                total_height += size.height();
            }
            let area = resolver.with_height(height);
            let mut scroll = self.scroll.borrow_mut();
            if scroll.tab != tab {
                scroll.tab = tab;
                scroll.following = true;
            }
            scroll.maximum = (total_height - area.height).max(0.0);
            scroll.offset = if scroll.following {
                scroll.maximum
            } else {
                scroll.offset.min(scroll.maximum)
            };
            HistoryLayout {
                area,
                messages,
                total_height,
                offset: scroll.offset,
            }
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a Context<ClientState>,
        _: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        if info.area.check().dont_mark().run(layout) {
            layout.register_scroll_handler(self.scroll.as_ref());
        }
        layout.with_clip(info.area, |layout| {
            let messages = state.get(&self.messages_path);
            for message in &info.messages {
                let top = info.area.top + message.top - info.offset;
                if top + message.height <= info.area.top || top >= info.area.top + info.area.height {
                    continue;
                }
                let entry = &messages[message.index];
                layout.add_text(
                    Area {
                        top,
                        height: message.height,
                        ..info.area
                    },
                    &entry.text,
                    FontSize(13.0),
                    message_color(&entry.color),
                    Color::rgb_u8(255, 191, 116),
                    HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
                    VerticalAlignment::Top { offset: 0.0 },
                    OverflowBehavior::LineBreak,
                );
            }
        });
        let scale_x = info.area.width / 374.0;
        let track = Area {
            left: info.area.left + info.area.width + 8.0 * scale_x,
            top: info.area.top + 1.0,
            width: 6.0 * scale_x,
            height: info.area.height - 2.0,
        };
        let maximum = (info.total_height - info.area.height).max(0.0);
        let thumb_height = if maximum == 0.0 {
            track.height
        } else {
            (track.height * info.area.height / info.total_height).max(15.0).min(track.height)
        };
        let thumb_top = if maximum == 0.0 {
            track.top
        } else {
            track.top + (track.height - thumb_height) * info.offset / maximum
        };
        if track.check().run(layout) {
            self.jump.fraction.set(
                ((layout.get_mouse_position().top - track.top - thumb_height / 2.0) / (track.height - thumb_height).max(1.0))
                    .clamp(0.0, 1.0),
            );
            layout.register_click_handler(MouseButton::Left, &self.jump);
        }
        layout.add_rectangle(
            Area {
                top: thumb_top,
                height: thumb_height,
                ..track
            },
            CornerDiameter::uniform(4.0),
            Color::rgba_u8(119, 130, 156, 210),
            Color::TRANSPARENT,
            ShadowPadding::uniform(0.0),
        );
    }
}

#[derive(Default, RustState, StateElement)]
pub struct ChatWindowState {
    current_text: String,
    active_tab: usize,
    recipient: String,
}

pub struct ChatWindow<A, B> {
    state_path: A,
    messages_path: B,
    frame: Arc<Texture>,
}

impl<A, B> ChatWindow<A, B> {
    pub fn new(state_path: A, messages_path: B, frame: Arc<Texture>) -> Self {
        Self {
            state_path,
            messages_path,
            frame,
        }
    }
}

impl<A, B> CustomWindow<ClientState> for ChatWindow<A, B>
where
    A: Path<ClientState, ChatWindowState>,
    B: Path<ClientState, Vec<ChatMessage>>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Chat)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        let current_text = self.state_path.current_text();
        let active_tab = self.state_path.active_tab();
        let recipient = self.state_path.recipient();
        let send_action = move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
            let value = state.get(&current_text).trim().to_owned();
            if value.is_empty() {
                return;
            }
            let tab = *state.get(&active_tab);
            match tab {
                1 => return,
                2 => {
                    if state.get(&client_state().party_members()).is_empty() {
                        state.update_value_with(client_state().chat_messages(), |messages| {
                            messages.push(ChatMessage::in_channel(
                                "Você precisa estar em um grupo para usar este canal.".to_owned(),
                                MessageColor::Error,
                                ChatChannel::Party,
                            ))
                        });
                        return;
                    }
                    queue.queue(InputEvent::SendPartyMessage { text: value });
                }
                3 => {
                    let target = state.get(&recipient).trim().to_owned();
                    if target.is_empty() {
                        queue.queue(Event::FocusElement {
                            focus_id: ChatRecipientBox.focus_id(),
                        });
                        return;
                    }
                    queue.queue(InputEvent::SendPrivateMessage {
                        recipient: target,
                        text: value,
                    });
                }
                _ => queue.queue(InputEvent::SendMessage { text: value }),
            }
            state.update_value_with(current_text, String::clear);
            queue.queue(Event::Unfocus);
        };
        let input = ErasedElement::new(text_box! {
            ghost_text: "Mensagem... (Enter para enviar)",
            state: current_text,
            input_handler: DefaultHandler::<_, _, MAXIMUM_CHAT_MESSAGE_LENGTH>::new(current_text, send_action),
            background_color: Color::TRANSPARENT,
            hovered_background_color: Color::rgba_u8(19, 22, 32, 80),
            focused_background_color: Color::rgba_u8(19, 22, 32, 100),
            shadow_color: Color::TRANSPARENT,
            shadow_padding: ShadowPadding::uniform(0.0),
            foreground_color: Color::rgb_u8(240, 235, 225),
            focused_foreground_color: Color::WHITE,
            ghost_foreground_color: Color::rgb_u8(153, 152, 153),
            font_size: FontSize(13.0),
            height: 22.0,
            horizontal_alignment: HorizontalAlignment::Left { offset: 9.0, border: 0.0 },
            vertical_alignment: VerticalAlignment::Center { offset: 0.0 },
            overflow_behavior: OverflowBehavior::Shrink,
            focus_id: ChatTextBox,
        });
        let recipient_input = ErasedElement::new(text_box! {
            ghost_text: "Jogador",
            state: recipient,
            input_handler: DefaultHandler::<_, _, 23>::new(recipient, |_: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                queue.queue(Event::FocusElement { focus_id: ChatTextBox.focus_id() });
            }),
            background_color: Color::TRANSPARENT,
            hovered_background_color: Color::rgba_u8(19, 22, 32, 80),
            focused_background_color: Color::rgba_u8(19, 22, 32, 100),
            shadow_color: Color::TRANSPARENT,
            shadow_padding: ShadowPadding::uniform(0.0),
            foreground_color: Color::rgb_u8(215, 169, 237),
            focused_foreground_color: Color::rgb_u8(236, 204, 255),
            ghost_foreground_color: Color::rgb_u8(173, 144, 187),
            font_size: FontSize(12.0),
            height: 22.0,
            horizontal_alignment: HorizontalAlignment::Left { offset: 9.0, border: 0.0 },
            vertical_alignment: VerticalAlignment::Center { offset: 0.0 },
            overflow_behavior: OverflowBehavior::Shrink,
            focus_id: ChatRecipientBox,
        });
        let tab_actions = (0..4)
            .map(|index| Box::new(SelectTab { path: active_tab, index }) as Box<dyn ClickHandler<ClientState>>)
            .collect();
        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            title_gap: 0.0,
            border: 0.0,
            gaps: 0.0,
            closable: false,
            resizable: true,
            minimum_width: CHAT_WIDTH,
            maximum_width: 800.0,
            minimum_height: CHAT_HEIGHT,
            maximum_height: 600.0,
            background_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
            elements: ChatBody {
                active_tab, frame: self.frame, drag: ChatDrag::default(), tabs: tab_actions,
                send: Box::new(send_action), history: ErasedElement::new(ChatHistory::new(self.messages_path, active_tab)),
                input, recipient: recipient_input,
            },
        }
    }
}

struct ChatBody<P> {
    active_tab: P,
    frame: Arc<Texture>,
    drag: ChatDrag,
    tabs: Vec<Box<dyn ClickHandler<ClientState>>>,
    send: Box<dyn ClickHandler<ClientState>>,
    history: ElementBox<ClientState>,
    input: ElementBox<ClientState>,
    recipient: ElementBox<ClientState>,
}

struct ChatBodyLayout {
    area: Area,
    tab: usize,
}

impl<P: Path<ClientState, usize>> Element<ClientState> for ChatBody<P> {
    type LayoutInfo = ChatBodyLayout;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        mut store: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.drag.window_id.set(store.get_window_id());
        with_single_resolver(resolvers, |resolver| {
            let height = resolver.push_available_area().height.unwrap_or(CHAT_HEIGHT).max(CHAT_HEIGHT);
            let area = resolver.with_height(height);
            let tab = *state.get(&self.active_tab);
            let mut history_resolver = Resolver::new(relative(area, 40.0, 50.0, 362.0, 162.0), 0.0, resolver.get_text_layouter());
            self.history.create_layout_info(state, store.child_store(0), &mut history_resolver);
            if tab != 1 {
                let input_area = if tab == 3 {
                    relative(area, 153.0, 220.0, 233.0, 25.0)
                } else {
                    relative(area, 54.0, 220.0, 332.0, 25.0)
                };
                let mut input_resolver = Resolver::new(input_area, 0.0, resolver.get_text_layouter());
                self.input.create_layout_info(state, store.child_store(1), &mut input_resolver);
                if tab == 3 {
                    let mut recipient_resolver = Resolver::new(relative(area, 54.0, 220.0, 94.0, 25.0), 0.0, resolver.get_text_layouter());
                    self.recipient
                        .create_layout_info(state, store.child_store(2), &mut recipient_resolver);
                }
            }
            ChatBodyLayout { area, tab }
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a Context<ClientState>,
        store: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let area = info.area;
        // Fit the generated frame's visible bounds; keep its original alpha.
        layout.add_texture(
            Area {
                left: area.left - 29.0 * area.width / 1578.0,
                top: area.top - 96.0 * area.height / 789.0,
                width: 1633.0 * area.width / 1578.0,
                height: 963.0 * area.height / 789.0,
            },
            self.frame.clone(),
            Color::WHITE,
            true,
        );

        let drag_areas = [
            relative(area, 0.0, 0.0, 440.0, 12.0),
            relative(area, 0.0, 12.0, 32.0, 32.0),
            relative(area, 410.0, 12.0, 30.0, 32.0),
        ];
        if drag_areas.iter().any(|area| area.check().run(layout)) {
            layout.register_click_handler(MouseButton::Left, &self.drag);
            layout.add_tooltip("Arraste a moldura para mover o chat", DragTooltip.tooltip_id());
        }
        layout.with_layer(|layout| {
            for (index, label) in TAB_LABELS.iter().enumerate() {
                let tab = relative(area, 36.0 + index as f32 * 92.0, 14.0, 86.0, 25.0);
                let hovered = tab.check().run(layout);
                if hovered {
                    layout.register_click_handler(MouseButton::Left, self.tabs[index].as_ref());
                }
                if index == info.tab || hovered {
                    layout.add_rectangle(
                        relative(area, 40.0 + index as f32 * 92.0, 18.0, 78.0, 17.0),
                        CornerDiameter::uniform(5.0),
                        if index == info.tab {
                            Color::rgba_u8(136, 20, 35, 125)
                        } else {
                            Color::rgba_u8(78, 68, 82, 85)
                        },
                        Color::TRANSPARENT,
                        ShadowPadding::uniform(0.0),
                    );
                }
                text(
                    layout,
                    tab,
                    label,
                    13.0,
                    if index == info.tab {
                        Color::rgb_u8(255, 238, 218)
                    } else {
                        Color::rgb_u8(196, 194, 193)
                    },
                );
            }
            self.history.lay_out(state, store.child_store(0), &(), layout);
            if info.tab != 1 {
                self.input.lay_out(state, store.child_store(1), &(), layout);
                if info.tab == 3 {
                    self.recipient.lay_out(state, store.child_store(2), &(), layout);
                    layout.add_rectangle(
                        relative(area, 149.0, 224.0, 1.0, 16.0),
                        CornerDiameter::uniform(0.0),
                        Color::rgba_u8(183, 148, 188, 130),
                        Color::TRANSPARENT,
                        ShadowPadding::uniform(0.0),
                    );
                }
            } else {
                text(
                    layout,
                    relative(area, 54.0, 220.0, 332.0, 25.0),
                    "Registro de combate",
                    12.0,
                    Color::rgb_u8(155, 150, 143),
                );
            }
            // The left circle is an ornamental counterpart to the send control.
            layout.add_rectangle(
                relative(area, 27.0, 229.0, 10.0, 10.0),
                CornerDiameter::uniform(10.0),
                Color::rgba_u8(91, 12, 25, 220),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_rectangle(
                relative(area, 30.0, 232.0, 4.0, 4.0),
                CornerDiameter::uniform(4.0),
                Color::rgb_u8(244, 67, 79),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            let send_area = relative(area, 391.0, 221.0, 24.0, 25.0);
            let hovered = send_area.check().run(layout);
            if hovered && info.tab != 1 {
                layout.register_click_handler(MouseButton::Left, self.send.as_ref());
                layout.add_tooltip("Enviar mensagem (Enter)", SendTooltip.tooltip_id());
            }
            layout.add_rectangle(
                relative(area, 395.0, 224.0, 17.0, 17.0),
                CornerDiameter::uniform(17.0),
                Color::rgb_u8(182, 190, 192),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_rectangle(
                relative(area, 397.0, 226.0, 13.0, 13.0),
                CornerDiameter::uniform(13.0),
                if info.tab == 1 { Color::rgb_u8(37, 39, 43) }
                else if hovered { Color::rgb_u8(79, 88, 91) }
                else { Color::rgb_u8(24, 27, 32) },
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_icon(
                relative(area, 399.0, 228.0, 9.0, 9.0),
                Icon::ArrowRight,
                if info.tab == 1 {
                    Color::rgb_u8(121, 130, 132)
                } else if hovered {
                    Color::rgb_u8(251, 251, 247)
                } else {
                    Color::rgb_u8(216, 224, 225)
                },
            );
        });
    }
}
