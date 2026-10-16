use std::cell::UnsafeCell;

use korangar_interface::components::text::Text;
use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Context, PathExt, Selector};

use crate::interface::windows::WindowClass;
use crate::state::hotbar::HotbarPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

pub struct HotkeySettingsWindow;

struct HotkeyTextSelector {
    index: usize,
    text: UnsafeCell<String>,
}

impl HotkeyTextSelector {
    fn new(index: usize) -> Self {
        Self {
            index,
            text: UnsafeCell::default(),
        }
    }
}

impl Selector<ClientState, String> for HotkeyTextSelector {
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let hotbar = client_state().hotbar().follow_safe(state);
        let label = if hotbar.capture_slot() == Some(self.index) {
            "Press a key..."
        } else if hotbar.binding(self.index).is_empty() {
            "Default"
        } else {
            hotbar.binding(self.index)
        };
        unsafe {
            *self.text.get() = label.to_string();
            Some(self.text.as_ref_unchecked())
        }
    }
}

impl CustomWindow<ClientState> for HotkeySettingsWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::HotkeySettings)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Skill Shortcuts",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: (
                text! { text: "Top: Numpad 1-9 | Bottom: F1-F9. WASD moves; Alt opens HUD windows. Esc cancels; Delete restores the default." },
                std::array::from_fn::<_, 18, _>(|index| {
                    let binding_text: Text<String, _, _, _, _, _, _, _, _> = text! { text: HotkeyTextSelector::new(index) };
                    split! {
                        gaps: theme().window().gaps(),
                        children: (
                            text! { text: format!("Row {} / {}", if index < 9 { 2 } else { 1 }, index % 9 + 1) },
                            binding_text,
                            button! {
                                text: "Bind",
                                tooltip: "Press a key. Esc cancels; Delete restores the default. WASD and Alt are reserved.",
                                event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                    state.update_value_with(client_state().hotbar().capture_slot(), move |slot| *slot = Some(index));
                                    queue.queue(Event::Unfocus);
                                },
                            },
                        ),
                    }
                }),
            ),
        }
    }
}
