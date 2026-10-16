mod event;
mod key;
mod mode;

use std::mem::variant_count;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ragnarok_packets::{ClientTick, HotbarSlot};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;

pub use self::event::InputEvent;
pub use self::key::Key;
pub use self::mode::{Grabbed, MouseInputMode, MouseModeExt};
use crate::graphics::{PickerTarget, ScreenPosition, ScreenSize};
use crate::state::hotbar::Hotbar;

const MOUSE_SCOLL_MULTIPLIER: f32 = 30.0;
const KEY_COUNT: usize = variant_count::<KeyCode>();
const DOUBLE_CLICK_TIME_MS: u32 = 250;

#[derive(Debug, Clone, Copy)]
struct PreviousMouseButton {
    button: MouseButton,
    tick: ClientTick,
}

#[cfg(test)]
mod control_tests {
    use super::*;

    #[cfg(feature = "debug")]
    fn frame() -> korangar_debug::profiling::ActiveMeasurement {
        let profiler = Box::leak(Box::new(std::sync::Mutex::new(korangar_debug::profiling::Profiler::default())));
        korangar_debug::profiling::Profiler::set_active(profiler);
        let frame = profiler.lock().unwrap().start_frame();
        frame
    }

    fn input() -> InputSystem {
        InputSystem::new(Arc::new(AtomicU64::new(0)))
    }

    #[test]
    fn physical_numpad_and_function_keys_activate_visible_rows() {
        #[cfg(feature = "debug")]
        let _frame = frame();
        for (key, expected) in [(KeyCode::Numpad1, 9), (KeyCode::Numpad9, 17), (KeyCode::F1, 0), (KeyCode::F9, 8)] {
            let mut input = input();
            let mut hotbar = Hotbar::default();
            let mut events = Vec::new();
            input.update_keyboard(key, ElementState::Pressed);
            input.update_delta(ClientTick(1));
            input.handle_skill_hotkeys(&mut events, &mut hotbar, false);
            assert!(matches!(events.as_slice(), [InputEvent::CastSkill { slot }] if slot.0 == expected));
            events.clear();
            input.update_keyboard(key, ElementState::Released);
            input.update_delta(ClientTick(2));
            input.handle_skill_hotkeys(&mut events, &mut hotbar, true);
            assert!(matches!(events.as_slice(), [InputEvent::StopSkill { slot }] if slot.0 == expected));
        }
    }

    #[test]
    fn typing_or_alt_shortcuts_do_not_trigger_numpad_skills() {
        #[cfg(feature = "debug")]
        let _frame = frame();
        let mut input = input();
        let mut hotbar = Hotbar::default();
        let mut events = Vec::new();
        input.update_keyboard(KeyCode::Numpad1, ElementState::Pressed);
        input.update_delta(ClientTick(1));
        input.handle_skill_hotkeys(&mut events, &mut hotbar, true);
        assert!(events.is_empty());
        input.update_keyboard(KeyCode::AltLeft, ElementState::Pressed);
        input.update_keyboard(KeyCode::Numpad2, ElementState::Pressed);
        input.update_delta(ClientTick(2));
        input.handle_skill_hotkeys(&mut events, &mut hotbar, false);
        assert!(events.is_empty());
    }

    #[test]
    fn wasd_supports_diagonals_opposites_and_request_throttling() {
        let mut input = input();
        let mut events = Vec::new();
        input.update_keyboard(KeyCode::KeyW, ElementState::Pressed);
        input.update_keyboard(KeyCode::KeyD, ElementState::Pressed);
        input.handle_movement(&mut events, ClientTick(10));
        assert!(matches!(events.as_slice(), [InputEvent::PlayerMoveRelative {
            horizontal: 1,
            vertical: 1
        }]));
        events.clear();
        input.handle_movement(&mut events, ClientTick(50));
        assert!(events.is_empty());
        input.handle_movement(&mut events, ClientTick(150));
        assert_eq!(events.len(), 1);
        events.clear();
        input.update_keyboard(KeyCode::KeyA, ElementState::Pressed);
        input.update_keyboard(KeyCode::KeyS, ElementState::Pressed);
        input.handle_movement(&mut events, ClientTick(300));
        assert!(events.is_empty());
        input.update_keyboard(KeyCode::KeyA, ElementState::Released);
        input.update_keyboard(KeyCode::AltLeft, ElementState::Pressed);
        input.handle_movement(&mut events, ClientTick(500));
        assert!(events.is_empty());
    }
}

// TODO: Rename
pub struct InputReport {
    pub mouse_click: Option<korangar_interface::layout::MouseButton>,
    pub mouse_position: ScreenPosition,
    pub mouse_delta: ScreenSize,
    pub mouse_button_released: bool,
    pub left_mouse_button_down: bool,
    pub scroll: Option<f32>,
    pub drag: Option<ScreenSize>,
    pub characters: Vec<char>,
    pub mouse_target: PickerTarget,
}

pub struct InputSystem {
    previous_mouse_position: ScreenPosition,
    new_mouse_position: ScreenPosition,
    mouse_delta: ScreenSize,
    previous_scroll_position: f32,
    new_scroll_position: f32,
    scroll_delta: f32,
    left_mouse_button: Key,
    right_mouse_button: Key,
    keys: [Key; KEY_COUNT],
    key_names: [Option<String>; KEY_COUNT],
    active_hotkey_slots: [Option<HotbarSlot>; KEY_COUNT],
    last_movement_tick: Option<u32>,
    input_buffer: Vec<char>,
    picker_value: Arc<AtomicU64>,
    previous_mouse_button: Option<PreviousMouseButton>,
}

impl InputSystem {
    pub fn new(picker_value: Arc<AtomicU64>) -> Self {
        let previous_mouse_position = ScreenPosition::default();
        let new_mouse_position = ScreenPosition::default();
        let mouse_delta = ScreenSize::default();

        let previous_scroll_position = 0.0;
        let new_scroll_position = 0.0;
        let scroll_delta = 0.0;

        let left_mouse_button = Key::default();
        let right_mouse_button = Key::default();
        let keys = [Key::default(); KEY_COUNT];

        let input_buffer = Vec::new();
        let previous_mouse_button = None;

        Self {
            previous_mouse_position,
            new_mouse_position,
            mouse_delta,
            previous_scroll_position,
            new_scroll_position,
            scroll_delta,
            left_mouse_button,
            right_mouse_button,
            keys,
            key_names: std::array::from_fn(|_| None),
            active_hotkey_slots: [None; KEY_COUNT],
            last_movement_tick: None,
            input_buffer,
            picker_value,
            previous_mouse_button,
        }
    }

    pub fn reset(&mut self) {
        self.left_mouse_button.reset();
        self.right_mouse_button.reset();
        self.keys.iter_mut().for_each(|key| key.reset());
    }

    pub fn update_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        self.new_mouse_position = ScreenPosition {
            left: position.x as f32,
            top: position.y as f32,
        };
    }

    pub fn update_mouse_buttons(&mut self, button: MouseButton, state: ElementState) {
        let pressed = matches!(state, ElementState::Pressed);

        match button {
            MouseButton::Left => self.left_mouse_button.set_down(pressed),
            MouseButton::Right => self.right_mouse_button.set_down(pressed),
            _ignored => {}
        }
    }

    pub fn update_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_x, y) => self.new_scroll_position += y * MOUSE_SCOLL_MULTIPLIER,
            MouseScrollDelta::PixelDelta(position) => self.new_scroll_position += position.y as f32,
        }
    }

    pub fn update_keyboard(&mut self, key_code: KeyCode, state: ElementState) {
        let pressed = matches!(state, ElementState::Pressed);
        self.key_names[key_code as usize].get_or_insert_with(|| format!("{key_code:?}"));
        self.keys[key_code as usize].set_down(pressed);
    }

    pub fn buffer_character(&mut self, character: char) {
        self.input_buffer.push(character);
    }

    #[cfg_attr(feature = "debug", korangar_debug::profile("update input system"))]
    pub fn update_delta(&mut self, client_tick: ClientTick) -> InputReport {
        self.mouse_delta = self.new_mouse_position - self.previous_mouse_position;
        self.previous_mouse_position = self.new_mouse_position;

        self.scroll_delta = self.new_scroll_position - self.previous_scroll_position;
        self.previous_scroll_position = self.new_scroll_position;

        self.left_mouse_button.update();
        self.right_mouse_button.update();
        self.keys.iter_mut().for_each(|key| key.update());

        let mouse_button_released = self.left_mouse_button.released() || self.right_mouse_button.released();

        let last_pixel_value = self.picker_value.load(Ordering::Acquire);
        let mouse_target = PickerTarget::from(last_pixel_value);

        let mut mouse_click = None;

        if self.left_mouse_button.pressed() {
            if let Some(previous_mouse_button) = self.previous_mouse_button
                && previous_mouse_button.button == MouseButton::Left
                && client_tick.0.wrapping_sub(previous_mouse_button.tick.0) <= DOUBLE_CLICK_TIME_MS
            {
                self.previous_mouse_button = None;

                mouse_click = Some(korangar_interface::layout::MouseButton::DoubleLeft);
            } else {
                self.previous_mouse_button = Some(PreviousMouseButton {
                    button: MouseButton::Left,
                    tick: client_tick,
                });

                mouse_click = Some(korangar_interface::layout::MouseButton::Left);
            }
        } else if self.right_mouse_button.pressed() {
            if let Some(previous_mouse_button) = self.previous_mouse_button
                && previous_mouse_button.button == MouseButton::Right
                && client_tick.0.wrapping_sub(previous_mouse_button.tick.0) <= DOUBLE_CLICK_TIME_MS
            {
                self.previous_mouse_button = None;

                mouse_click = Some(korangar_interface::layout::MouseButton::DoubleRight);
            } else {
                self.previous_mouse_button = Some(PreviousMouseButton {
                    button: MouseButton::Right,
                    tick: client_tick,
                });

                mouse_click = Some(korangar_interface::layout::MouseButton::Right);
            }
        }

        InputReport {
            mouse_click,
            mouse_position: self.new_mouse_position,
            mouse_delta: self.mouse_delta,
            mouse_button_released,
            left_mouse_button_down: self.left_mouse_button.down(),
            scroll: (self.scroll_delta != 0.0).then_some(self.scroll_delta),
            drag: self.left_mouse_button.down().then_some(self.mouse_delta),
            characters: self.input_buffer.drain(..).collect(),
            mouse_target,
        }
    }

    fn get_key(&self, key_code: KeyCode) -> &Key {
        &self.keys[key_code as usize]
    }

    pub fn escape_pressed(&self) -> bool {
        self.get_key(KeyCode::Escape).pressed()
    }

    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        self.get_key(key_code).pressed()
    }

    pub fn alt_shortcut_active(&self) -> bool {
        self.get_key(KeyCode::AltLeft).down() || self.get_key(KeyCode::AltRight).down()
    }

    pub fn handle_movement(&mut self, events: &mut Vec<InputEvent>, tick: ClientTick) {
        let modified = [
            KeyCode::AltLeft,
            KeyCode::AltRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ]
        .iter()
        .any(|key| self.get_key(*key).down());
        let horizontal = i8::from(self.get_key(KeyCode::KeyD).down()) - i8::from(self.get_key(KeyCode::KeyA).down());
        let vertical = i8::from(self.get_key(KeyCode::KeyW).down()) - i8::from(self.get_key(KeyCode::KeyS).down());
        if modified || (horizontal == 0 && vertical == 0) {
            self.last_movement_tick = None;
            return;
        }
        if self.last_movement_tick.is_none_or(|last| tick.0.wrapping_sub(last) >= 140) {
            self.last_movement_tick = Some(tick.0);
            events.push(InputEvent::PlayerMoveRelative { horizontal, vertical });
        }
    }

    pub fn handle_skill_hotkeys(&mut self, events: &mut Vec<InputEvent>, hotbar: &mut Hotbar, text_input_has_focus: bool) {
        let shift_down = self.get_key(KeyCode::ShiftLeft).down() || self.get_key(KeyCode::ShiftRight).down();
        let control_down = self.get_key(KeyCode::ControlLeft).down() || self.get_key(KeyCode::ControlRight).down();
        let alt_down = self.get_key(KeyCode::AltLeft).down() || self.get_key(KeyCode::AltRight).down();
        let fixed_function_keys = [
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
            KeyCode::F7,
            KeyCode::F8,
            KeyCode::F9,
        ];
        let fixed_digit_keys = [
            KeyCode::Numpad1,
            KeyCode::Numpad2,
            KeyCode::Numpad3,
            KeyCode::Numpad4,
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
            KeyCode::Numpad9,
        ];

        for index in 0..KEY_COUNT {
            if self.keys[index].released() && !self.keys[index].pressed() {
                if let Some(slot) = self.active_hotkey_slots[index].take() {
                    events.push(InputEvent::StopSkill { slot });
                }
            }
            if !self.keys[index].pressed() {
                continue;
            }
            let Some(name) = self.key_names[index].as_deref() else { continue };
            // Movement keys and Alt shortcuts belong to the game/HUD, never to skill
            // bindings.
            if alt_down || (!control_down && !shift_down && matches!(name, "KeyW" | "KeyA" | "KeyS" | "KeyD")) {
                continue;
            }
            if name.starts_with("Shift") || name.starts_with("Control") || name.starts_with("Alt") || name.starts_with("Super") {
                continue;
            }
            let bare_fixed_key = !control_down
                && !shift_down
                && !alt_down
                && (fixed_function_keys.iter().any(|key| *key as usize == index)
                    || fixed_digit_keys.iter().any(|key| *key as usize == index));
            if let Some(slot) = hotbar.capture_slot() {
                if name == "Escape" {
                    hotbar.cancel_capture();
                } else if bare_fixed_key {
                    // Reserved keys always activate their rows, even when the binding editor
                    // was closed while it was still waiting for a custom key.
                    hotbar.cancel_capture();
                } else if name == "Delete" || name == "Backspace" {
                    hotbar.set_binding(slot, String::new());
                } else if !(alt_down && name == "F4")
                    && !(name == "Escape"
                        || (!control_down
                            && !shift_down
                            && !alt_down
                            && (fixed_function_keys.iter().any(|key| *key as usize == index)
                                || fixed_digit_keys.iter().any(|key| *key as usize == index))))
                    && !(alt_down && matches!(name, "KeyC" | "KeyE" | "KeyS" | "KeyA" | "KeyZ" | "KeyQ"))
                    && !(control_down
                        && matches!(
                            name,
                            "KeyS" | "KeyI" | "KeyG" | "KeyA" | "KeyH" | "KeyQ" | "KeyM" | "KeyC" | "KeyR" | "KeyP" | "KeyO" | "KeyN"
                        ))
                {
                    let binding = format!(
                        "{}{}{}{}",
                        if control_down { "Ctrl+" } else { "" },
                        if shift_down { "Shift+" } else { "" },
                        if alt_down { "Alt+" } else { "" },
                        name
                    );
                    hotbar.set_binding(slot, binding);
                }
                if !bare_fixed_key {
                    continue;
                }
            }
            // Numbers and custom letter bindings must not activate while the player types
            // in chat. The fixed function keys remain usable even if an old
            // text focus was left behind.
            let bare_function_key =
                !control_down && !shift_down && !alt_down && fixed_function_keys.iter().any(|key| *key as usize == index);
            if text_input_has_focus && !bare_function_key {
                continue;
            }
            // Alt+C belongs to the character overview, even if an older hotbar
            // configuration still contains that binding.
            if alt_down && !control_down && !shift_down && name == "KeyC" {
                continue;
            }
            let binding = format!(
                "{}{}{}{}",
                if control_down { "Ctrl+" } else { "" },
                if shift_down { "Shift+" } else { "" },
                if alt_down { "Alt+" } else { "" },
                name
            );
            let fixed = if !control_down && !shift_down && !alt_down {
                fixed_function_keys
                    .iter()
                    .position(|key| *key as usize == index)
                    .or_else(|| fixed_digit_keys.iter().position(|key| *key as usize == index).map(|slot| slot + 9))
            } else {
                None
            };
            let slot = fixed.or_else(|| (0..18).find(|slot| hotbar.binding(*slot) == binding));
            if let Some(slot) = slot {
                let slot = HotbarSlot(slot as u16);
                #[cfg(feature = "debug")]
                println!("[hotbar] tecla={} slot={}", name, slot.0);
                self.active_hotkey_slots[index] = Some(slot);
                events.push(InputEvent::CastSkill { slot });
                if self.keys[index].released() {
                    self.active_hotkey_slots[index] = None;
                    events.push(InputEvent::StopSkill { slot });
                }
            }
        }
    }

    #[cfg_attr(feature = "debug", korangar_debug::profile)]
    pub fn handle_keyboard_input(
        &mut self,
        events: &mut Vec<InputEvent>,
        #[cfg(feature = "debug")] process_mouse: bool,
        #[cfg(feature = "debug")] use_debug_camera: bool,
    ) {
        let alt_down = self.get_key(KeyCode::AltLeft).down() || self.get_key(KeyCode::AltRight).down();
        let control_down = self.get_key(KeyCode::ControlLeft).down();
        let shift_down = self.get_key(KeyCode::ShiftLeft).down() || self.get_key(KeyCode::ShiftRight).down();

        if alt_down && !control_down && !shift_down && self.get_key(KeyCode::KeyC).pressed() {
            events.push(InputEvent::ToggleCharacterOverviewWindow);
        }

        if alt_down && !control_down && !shift_down && self.get_key(KeyCode::KeyM).pressed() {
            events.push(InputEvent::ToggleMiniMapWindow);
        }
        if alt_down && !control_down && !shift_down && self.get_key(KeyCode::KeyL).pressed() {
            events.push(InputEvent::GameService {
                action: crate::state::services::ServiceAction::OpenMail,
            });
        }
        if alt_down && !control_down && !shift_down && self.get_key(KeyCode::KeyJ).pressed() {
            events.push(InputEvent::GameService {
                action: crate::state::services::ServiceAction::OpenPets,
            });
        }
        if alt_down && !control_down && !shift_down && self.get_key(KeyCode::KeyK).pressed() {
            events.push(InputEvent::ToggleHotkeySettingsWindow);
        }
        for (key, action) in [
            (KeyCode::KeyP, crate::state::services::ServiceAction::OpenSocial),
            (KeyCode::KeyG, crate::state::services::ServiceAction::OpenGuild),
            (KeyCode::KeyT, crate::state::services::ServiceAction::OpenTrade),
            (KeyCode::KeyB, crate::state::services::ServiceAction::OpenAuction),
        ] {
            if alt_down && !control_down && !shift_down && self.get_key(key).pressed() {
                events.push(InputEvent::GameService { action });
            }
        }

        if alt_down && self.get_key(KeyCode::KeyE).pressed() {
            events.push(InputEvent::ToggleInventoryWindow);
        }

        if alt_down && self.get_key(KeyCode::KeyS).pressed() {
            events.push(InputEvent::ToggleSkillTreeWindow);
        }

        if alt_down && self.get_key(KeyCode::KeyA).pressed() {
            events.push(InputEvent::ToggleStatsWindow);
        }

        if alt_down && self.get_key(KeyCode::KeyZ).pressed() {
            events.push(InputEvent::ToggleFriendListWindow);
        }

        if alt_down && self.get_key(KeyCode::KeyQ).pressed() {
            events.push(InputEvent::ToggleEquipmentWindow);
        }

        if control_down && self.get_key(KeyCode::KeyS).pressed() {
            events.push(InputEvent::ToggleGameSettingsWindow);
        }

        if control_down && self.get_key(KeyCode::KeyI).pressed() {
            events.push(InputEvent::ToggleInterfaceSettingsWindow);
        }

        if control_down && self.get_key(KeyCode::KeyG).pressed() {
            events.push(InputEvent::ToggleGraphicsSettingsWindow);
        }

        if control_down && self.get_key(KeyCode::KeyA).pressed() {
            events.push(InputEvent::ToggleAudioSettingsWindow);
        }

        if control_down && self.get_key(KeyCode::KeyH).pressed() {
            events.push(InputEvent::ToggleShowInterface);
        }

        if control_down && self.get_key(KeyCode::KeyQ).pressed() {
            events.push(InputEvent::CloseTopWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyM).pressed() {
            events.push(InputEvent::ToggleMapsWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyC).pressed() {
            events.push(InputEvent::ToggleClientStateInspectorWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyR).pressed() {
            events.push(InputEvent::ToggleRenderOptionsWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyP).pressed() {
            events.push(InputEvent::ToggleProfilerWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyO).pressed() {
            events.push(InputEvent::ToggleCommandsWindow);
        }

        #[cfg(feature = "debug")]
        if control_down && self.get_key(KeyCode::KeyN).pressed() {
            events.push(InputEvent::TogglePacketInspectorWindow);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::ShiftLeft).pressed() && use_debug_camera {
            events.push(InputEvent::CameraAccelerate);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::ShiftLeft).released() && use_debug_camera {
            events.push(InputEvent::CameraDecelerate);
        }

        // TODO: This should be moved.
        #[cfg(feature = "debug")]
        if self.right_mouse_button.down() && !self.right_mouse_button.pressed() && process_mouse && use_debug_camera {
            let offset = -cgmath::Vector2::new(self.mouse_delta.width, self.mouse_delta.height);
            events.push(InputEvent::CameraLookAround { offset });
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::KeyW).down() && use_debug_camera {
            events.push(InputEvent::CameraMoveForward);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::KeyS).down() && use_debug_camera {
            events.push(InputEvent::CameraMoveBackward);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::KeyA).down() && use_debug_camera {
            events.push(InputEvent::CameraMoveLeft);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::KeyD).down() && use_debug_camera {
            events.push(InputEvent::CameraMoveRight);
        }

        #[cfg(feature = "debug")]
        if self.get_key(KeyCode::Space).down() && use_debug_camera {
            events.push(InputEvent::CameraMoveUp);
        }

        self.input_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_c_toggles_character_overview_once_per_press() {
        #[cfg(feature = "debug")]
        let _frame = {
            use std::sync::{Mutex, OnceLock};

            use korangar_debug::profiling::Profiler;

            static PROFILER: OnceLock<Mutex<Profiler>> = OnceLock::new();
            let profiler = PROFILER.get_or_init(|| Mutex::new(Profiler::default()));
            Profiler::set_active(profiler);
            profiler.lock().unwrap().start_frame()
        };

        for alt_key in [KeyCode::AltLeft, KeyCode::AltRight] {
            let mut input = InputSystem::new(Arc::new(AtomicU64::new(0)));
            input.update_keyboard(alt_key, ElementState::Pressed);
            input.update_keyboard(KeyCode::KeyC, ElementState::Pressed);
            input.update_delta(ClientTick(1));

            let mut events = Vec::new();
            input.handle_keyboard_input(
                &mut events,
                #[cfg(feature = "debug")]
                false,
                #[cfg(feature = "debug")]
                false,
            );
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, InputEvent::ToggleCharacterOverviewWindow))
            );

            events.clear();
            input.update_delta(ClientTick(2));
            input.handle_keyboard_input(
                &mut events,
                #[cfg(feature = "debug")]
                false,
                #[cfg(feature = "debug")]
                false,
            );
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, InputEvent::ToggleCharacterOverviewWindow))
            );
        }
    }
}
