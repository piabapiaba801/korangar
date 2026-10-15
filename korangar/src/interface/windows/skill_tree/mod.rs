mod slot;
mod state;
mod tabs;

use std::sync::Arc;

use hashbrown::HashMap;
use korangar_interface::element::StateElement;
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::{SkillId, SkillLevel};
use rust_state::{Context, ManuallyAssertExt, Path, PathExt, RustState, VecIndexExt};

use crate::input::InputEvent;
use crate::graphics::{Color, ShadowPadding, Texture};
use crate::interface::windows::WindowClass;
use crate::interface::windows::themed_panel::{PanelStyle, ThemedPanel};
use crate::interface::windows::skill_tree::state::AvailablePointsDisplay;
use crate::interface::windows::skill_tree::tabs::{DropSkillWrapper, DynamicTabs, SkillTreeTab, TabSelector};
use crate::state::localization::LocalizationPathExt;
use crate::state::skills::{LearnedSkill, SkillTabLayoutPathExt, SkillTreeLayout, SkillTreeLayoutPathExt};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

/// Internal state of the skill tree window.
#[derive(Default, RustState, StateElement)]
pub struct SkillTreeWindowState {
    currently_skilling: bool,
    /// List of pending skill points. Each new point gets appended to this list,
    /// meaning the same skill id can be in the list multiple times. The
    /// total count of each skill id defines how many points to raise the
    /// skill level by.
    pending_skill_points: Vec<SkillId>,
    selected_tab: usize,
    highlighted_skill: Option<SkillId>,
    /// Map of skills that are not selected at their highest learned level. Only
    /// contains skills that have this property.
    #[hidden_element]
    chosen_skill_level: HashMap<SkillId, SkillLevel>,
}

pub struct SkillTreeWindow<A, B, C, D> {
    window_state_path: A,
    layout_path: B,
    skills_path: C,
    available_skill_points_path: D,
    frame: Arc<Texture>,
}

impl<A, B, C, D> SkillTreeWindow<A, B, C, D> {
    pub fn new(window_state_path: A, layout_path: B, skills_path: C, available_skill_points_path: D, frame: Arc<Texture>) -> Self {
        Self {
            window_state_path,
            layout_path,
            skills_path,
            available_skill_points_path,
            frame,
        }
    }
}

impl<A, B, C, D> CustomWindow<ClientState> for SkillTreeWindow<A, B, C, D>
where
    A: Path<ClientState, SkillTreeWindowState>,
    B: Path<ClientState, SkillTreeLayout>,
    C: Path<ClientState, Vec<LearnedSkill>>,
    D: Path<ClientState, u32>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::SkillTree)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 710.0,
            maximum_width: 710.0,
            minimum_height: 800.0,
            maximum_height: 1100.0,
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
                PanelStyle { header: 100.0, inset: 42.0, footer: 27.0, minimum_height: 800.0,
                    title_left: 95.0, title_width: 520.0, close_right: 7.0, close_top: 45.0 },
                |_| "Skill Tree".to_owned(),
                ErasedElement::new((
                split! {
                    gaps: theme().window().gaps(),
                    children: TabSelector::new(self.window_state_path.selected_tab(), self.layout_path),
                },
                text! { text: "", height: 8.0 },
                DropSkillWrapper::new(
                    tabs! {
                        selected_tab: self.window_state_path.selected_tab(),
                        children: DynamicTabs::new(move |index|
                            SkillTreeTab::new(
                                self.layout_path.tabs().index(index).skills().manually_asserted(),
                                self.skills_path,
                                self.window_state_path,
                            )
                        )
                    }
                ),
                text! { text: "", height: 40.0 },
                split! {
                    children: (
                        text! {
                            text: AvailablePointsDisplay::new(
                                client_state().localization().available_skill_points_text(),
                                self.available_skill_points_path,
                                self.window_state_path.pending_skill_points(),
                            ),
                            height: 20.0,
                            horizontal_alignment: HorizontalAlignment::Left { offset: 12.0, border: 0.0 },
                        },
                        either! {
                            selector: self.window_state_path.currently_skilling(),
                            on_true: split! {
                                gaps: theme().window().gaps(),
                                children: (
                                    button! {
                                        text: client_state().localization().reset_skill_points_button_text(),
                                        background_color: Color::TRANSPARENT,
                                        disabled_background_color: Color::TRANSPARENT,
                                        hovered_background_color: Color::rgba_u8(104, 27, 38, 100),
                                        shadow_color: Color::TRANSPARENT,
                                        shadow_padding: ShadowPadding::uniform(0.0),
                                        event: move |state: &Context<ClientState>, _: &mut EventQueue<ClientState>| {
                                            state.update_value_with(self.window_state_path.pending_skill_points(), Vec::clear);
                                        },
                                    },
                                    button! {
                                        text: client_state().localization().cancel_skill_points_button_text(),
                                        background_color: Color::TRANSPARENT,
                                        disabled_background_color: Color::TRANSPARENT,
                                        hovered_background_color: Color::rgba_u8(104, 27, 38, 100),
                                        shadow_color: Color::TRANSPARENT,
                                        shadow_padding: ShadowPadding::uniform(0.0),
                                        event: move |state: &Context<ClientState>, _: &mut EventQueue<ClientState>| {
                                            state.update_value_with(self.window_state_path.pending_skill_points(), Vec::clear);
                                            state.update_value(self.window_state_path.currently_skilling(), false);
                                        },
                                    },
                                    button! {
                                        text: client_state().localization().apply_skill_points_button_text(),
                                        background_color: Color::TRANSPARENT,
                                        disabled_background_color: Color::TRANSPARENT,
                                        hovered_background_color: Color::rgba_u8(104, 27, 38, 100),
                                        shadow_color: Color::TRANSPARENT,
                                        shadow_padding: ShadowPadding::uniform(0.0),
                                        disabled: ComputedSelector::new_default(move |state: &ClientState| {
                                            self.window_state_path.pending_skill_points().follow_safe(state).is_empty()
                                        }),
                                        event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                            let pending_skill_points_path = self.window_state_path.pending_skill_points();
                                            let pending_skill_points = state.get(&pending_skill_points_path);
                                            let skill_ids = pending_skill_points.clone();

                                            state.update_value(self.window_state_path.currently_skilling(), false);
                                            state.update_value_with(pending_skill_points_path, Vec::clear);

                                            queue.queue(InputEvent::LevelUpSkills { skill_ids });
                                        },
                                    },
                                ),
                            },
                            on_false: button! {
                                text: client_state().localization().distribute_skill_points_button_text(),
                                background_color: Color::TRANSPARENT,
                                disabled_background_color: Color::TRANSPARENT,
                                hovered_background_color: Color::rgba_u8(104, 27, 38, 100),
                                shadow_color: Color::TRANSPARENT,
                                shadow_padding: ShadowPadding::uniform(0.0),
                                disabled: ComputedSelector::new_default(move |state: &ClientState| {
                                    *self.available_skill_points_path.follow_safe(state) == 0
                                }),
                                event: SetToTrue(self.window_state_path.currently_skilling()),
                            },
                        },
                    )
                }
                )),
            ),
        }
    }
}
