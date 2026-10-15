use std::collections::HashMap;

#[cfg(feature = "debug")]
use korangar_debug::logging::{Colorize, print_debug};
use korangar_interface::window::{Anchor, AnchorPoint};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use super::WindowClass;
use crate::graphics::{ScreenPosition, ScreenSize};
use crate::state::ClientState;

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub anchor: Anchor<ClientState>,
    pub size: ScreenSize,
}

impl WindowState {
    pub fn new(anchor: Anchor<ClientState>, size: ScreenSize) -> Self {
        Self { anchor, size }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct WindowCache {
    entries: HashMap<WindowClass, WindowState>,
}

impl WindowCache {
    // Since `WindowClass` has some variants with debug features enabled, we use a
    // differen file to store the window cache. This avoids failing to load and
    // thereby wiping the previous window cache when switching between debug and
    // non-debug builds.
    #[cfg(not(feature = "debug"))]
    const FILE_NAME: &'static str = "client/window_cache.ron";
    #[cfg(feature = "debug")]
    const FILE_NAME: &'static str = "client/window_cache_debug.ron";

    fn load() -> Option<Self> {
        #[cfg(feature = "debug")]
        print_debug!("loading window cache from {}", Self::FILE_NAME.magenta());

        std::fs::read_to_string(Self::FILE_NAME)
            .ok()
            .and_then(|data| ron::from_str(&data).ok())
            .map(|entries| Self { entries })
    }

    fn save(&self) {
        #[cfg(feature = "debug")]
        print_debug!("saving window cache to {}", Self::FILE_NAME.magenta());

        let data = ron::ser::to_string_pretty(&self.entries, PrettyConfig::new()).unwrap();
        std::fs::write(Self::FILE_NAME, data).expect("unable to write file");
    }
}

impl korangar_interface::application::WindowCache<ClientState> for WindowCache {
    fn create() -> Self {
        let mut cache = Self::load().unwrap_or_else(|| {
            #[cfg(feature = "debug")]
            print_debug!(
                "failed to load window cache from {}. creating empty cache",
                Self::FILE_NAME.magenta()
            );

            Default::default()
        });
        cache.entries.entry(WindowClass::MiniMap).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::TopRight, ScreenPosition { left: -236.0, top: 32.0 }),
                ScreenSize { width: 224.0, height: 240.0 },
            )
        });
        cache.entries.entry(WindowClass::Chat).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::BottomLeft, ScreenPosition { left: 14.0, top: -275.0 }),
                ScreenSize { width: 440.0, height: 260.0 },
            )
        });
        let default_hud_state = || {
            WindowState::new(
                Anchor::at(AnchorPoint::BottomCenter, ScreenPosition { left: -185.0, top: -235.0 }),
                ScreenSize { width: 600.0, height: 168.0 },
            )
        };
        match cache.entries.get_mut(&WindowClass::Hotbar) {
            Some(state) if state.size.width == 580.0 && state.size.height == 122.0 => {
                *state = default_hud_state();
            }
            Some(state) if state.size.width == 600.0 && state.size.height == 220.0 => {
                state.anchor = state.anchor.translated(ScreenPosition { left: 0.0, top: 20.0 });
                state.size.height = 168.0;
            }
            None => {
                cache.entries.insert(WindowClass::Hotbar, default_hud_state());
            }
            _ => {}
        }
        let experience_anchor = cache.entries[&WindowClass::Hotbar].anchor.translated(ScreenPosition { left: 0.0, top: 138.0 });
        let experience_state = || WindowState::new(experience_anchor, ScreenSize { width: 600.0, height: 43.0 });
        match cache.entries.get_mut(&WindowClass::Experience) {
            Some(state)
                if matches!(
                    state.anchor.anchor_point(),
                    AnchorPoint::BottomLeft | AnchorPoint::BottomCenter | AnchorPoint::BottomRight
                ) && state.anchor.offset().top >= -12.0 =>
            {
                // The visible EXP band begins 12 px below this window's top.
                // Recover a saved position whose entire artwork is below the screen.
                *state = experience_state();
            }
            None => {
                cache.entries.insert(WindowClass::Experience, experience_state());
            }
            _ => {}
        }
        cache.entries.entry(WindowClass::CharacterOverview).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::TopLeft, ScreenPosition { left: 16.0, top: 16.0 }),
                ScreenSize { width: 270.0, height: 160.0 },
            )
        });
        cache.entries.entry(WindowClass::Target).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::TopCenter, ScreenPosition { left: -160.0, top: 8.0 }),
                ScreenSize { width: 320.0, height: 82.0 },
            )
        });
        cache.entries.entry(WindowClass::Party).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::TopLeft, ScreenPosition { left: 16.0, top: 190.0 }),
                ScreenSize { width: 270.0, height: 550.0 },
            )
        });
        cache.entries.entry(WindowClass::QuestTracker).or_insert_with(|| {
            WindowState::new(
                Anchor::at(AnchorPoint::TopRight, ScreenPosition { left: -296.0, top: 296.0 }),
                ScreenSize { width: 284.0, height: 216.0 },
            )
        });
        cache
    }

    fn get_window_state(&self, class: WindowClass) -> Option<(Anchor<ClientState>, ScreenSize)> {
        self.entries.get(&class).map(|entry| (entry.anchor, entry.size))
    }

    fn register_window(&mut self, class: WindowClass, anchor: Anchor<ClientState>, size: ScreenSize) {
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.anchor = anchor;
            entry.size = size;
        } else {
            let entry = WindowState::new(anchor, size);
            self.entries.insert(class, entry);
        }
    }

    fn update_anchor(&mut self, class: WindowClass, anchor: Anchor<ClientState>) {
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.anchor = anchor;
        }
    }

    fn update_size(&mut self, class: WindowClass, size: ScreenSize) {
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.size = size;
        }
    }
}

impl Drop for WindowCache {
    fn drop(&mut self) {
        self.save();
    }
}
