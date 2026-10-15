use std::cell::Cell;
use std::sync::Arc;

use korangar_components::skill_box;
use korangar_interface::MouseMode;
use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element, ElementBox, ErasedElement};
use korangar_interface::event::{ClickHandler, Event, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Icon, MouseButton, Resolver, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::HotbarSlot;
use rust_state::{ArrayLookupExt, Context, OptionExt, Path};

use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::interface::resource::SkillSource;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::hotbar::HOTBAR_SLOTS;
use crate::state::skills::{LearnableSkill, LearnedSkill, LearnedSkillPath};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, this_player};

const HUD_WIDTH: f32 = 600.0;
const FRAME_HEIGHT: f32 = 200.0;
const EXPANDED_HEIGHT: f32 = 168.0;
const COLLAPSED_HEIGHT: f32 = 118.0;
const EXPERIENCE_HEIGHT: f32 = 43.0;
const EXPERIENCE_SEGMENTS: usize = 10;
const SKILL_COLUMNS: [f32; 9] = [92.0, 139.0, 186.0, 233.0, 280.0, 329.0, 376.0, 423.0, 470.0];
const SKILL_SIZE: f32 = 38.0;

/// Display a band of the generated skin without changing or resampling the
/// original asset. Each HUD module can position its band independently.
fn frame_band(layout: &mut WindowLayout<'_, ClientState>, frame: &Arc<Texture>, destination: Area, source_left: f32, source_top: f32) {
    layout.with_clip(destination, |layout| {
        layout.add_texture(
            Area {
                left: destination.left - source_left,
                top: destination.top - source_top,
                width: HUD_WIDTH,
                height: FRAME_HEIGHT,
            },
            frame.clone(),
            Color::WHITE,
            true,
        );
    });
}

fn draw_text<'a>(layout: &mut WindowLayout<'a, ClientState>, area: Area, text: &'a str, size: f32, color: Color) {
    layout.add_text(
        area,
        text,
        FontSize(size),
        color,
        color,
        HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
        VerticalAlignment::Center { offset: 0.0 },
        OverflowBehavior::Shrink,
    );
}

#[derive(Default)]
struct HudDrag {
    window_id: Cell<u64>,
}

impl ClickHandler<ClientState> for HudDrag {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(Event::SetMouseMode {
            mouse_mode: MouseMode::MovingWindow {
                window_id: self.window_id.get(),
            },
        });
    }
}

struct SecondaryRow {
    expanded: Cell<bool>,
}

impl ClickHandler<ClientState> for SecondaryRow {
    fn handle_click(&self, _: &Context<ClientState>, _: &mut EventQueue<ClientState>) {
        self.expanded.set(!self.expanded.get());
    }
}

pub struct HotbarWindow<A, B> {
    hotbar_path: A,
    skills_path: B,
    frame: Arc<Texture>,
}

impl<A, B> HotbarWindow<A, B> {
    pub fn new(hotbar_path: A, skills_path: B, frame: Arc<Texture>) -> Self {
        Self {
            hotbar_path,
            skills_path,
            frame,
        }
    }
}

impl<A, B> CustomWindow<ClientState> for HotbarWindow<A, B>
where
    A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]> + Copy,
    B: Path<ClientState, Vec<LearnedSkill>> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Hotbar)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            title_gap: 0.0,
            closable: false,
            resizable: false,
            border: 0.0,
            gaps: 0.0,
            minimum_width: HUD_WIDTH,
            maximum_width: HUD_WIDTH,
            minimum_height: COLLAPSED_HEIGHT,
            maximum_height: EXPANDED_HEIGHT,
            background_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
            elements: ActionHudBody::new(self.hotbar_path, self.skills_path, self.frame),
        }
    }
}

struct ActionHudBody {
    frame: Arc<Texture>,
    slots: Vec<ElementBox<ClientState>>,
    drag: HudDrag,
    secondary_row: SecondaryRow,
    hp_text: String,
    sp_text: String,
    level_text: String,
    hp_ratio: f32,
    sp_ratio: f32,
}

struct ActionHudLayout {
    area: Area,
    expanded: bool,
}

impl ActionHudBody {
    fn new<A, B>(hotbar_path: A, skills_path: B, frame: Arc<Texture>) -> Self
    where
        A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]> + Copy,
        B: Path<ClientState, Vec<LearnedSkill>> + Copy,
    {
        let slots = (0..18)
            .map(|index| {
                let learnable_skill_path = hotbar_path.array_index(index).unwrapped();
                let learned_skill_path = LearnedSkillPath::new(learnable_skill_path, skills_path);
                ErasedElement::new(skill_box! {
                    learnable_skill_path,
                    learned_skill_path,
                    source: SkillSource::Hotbar { slot: HotbarSlot(index as u16) },
                }) as ElementBox<ClientState>
            })
            .collect();
        Self {
            frame,
            slots,
            drag: HudDrag::default(),
            secondary_row: SecondaryRow { expanded: Cell::new(true) },
            hp_text: String::new(),
            sp_text: String::new(),
            level_text: String::new(),
            hp_ratio: 0.0,
            sp_ratio: 0.0,
        }
    }

    fn slot_area(area: Area, index: usize) -> Area {
        Area {
            left: area.left + SKILL_COLUMNS[index % 9],
            top: area.top + if index < 9 { 124.0 } else { 74.0 },
            width: SKILL_SIZE,
            height: SKILL_SIZE,
        }
    }

    fn visible_slot(index: usize, expanded: bool) -> bool {
        expanded || index >= 9
    }

    fn vitals<'a>(&'a self, layout: &mut WindowLayout<'a, ClientState>, area: Area) {
        for (left, ratio, color, text) in [
            (84.0, self.hp_ratio, Color::rgba_u8(220, 35, 44, 175), self.hp_text.as_str()),
            (346.0, self.sp_ratio, Color::rgba_u8(30, 125, 235, 175), self.sp_text.as_str()),
        ] {
            let bar = Area {
                left: area.left + left,
                top: area.top + 44.0,
                width: 169.0 * ratio,
                height: 17.0,
            };
            if ratio > 0.0 {
                layout.add_rectangle(
                    bar,
                    CornerDiameter::uniform(4.0),
                    color,
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
                layout.add_rectangle(
                    Area {
                        left: bar.left + 1.0,
                        top: bar.top + 1.0,
                        width: (bar.width - 2.0).max(0.0),
                        height: 3.0,
                    },
                    CornerDiameter::uniform(2.0),
                    Color::rgba_u8(230, 240, 255, 55),
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
            }
            draw_text(
                layout,
                Area {
                    left: area.left + left,
                    top: area.top + 43.0,
                    width: 169.0,
                    height: 20.0,
                },
                text,
                11.0,
                Color::WHITE,
            );
        }
        draw_text(
            layout,
            Area {
                left: area.left + 280.0,
                top: area.top + 34.0,
                width: 40.0,
                height: 31.0,
            },
            &self.level_text,
            16.0,
            Color::rgb_u8(255, 221, 163),
        );
    }
}

impl Element<ClientState> for ActionHudBody {
    type LayoutInfo = ActionHudLayout;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        mut store: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.drag.window_id.set(store.get_window_id());
        if let Some(player) = state.try_get(&this_player()) {
            let common = player.get_common();
            self.hp_text = format!("{} / {}", common.health_points, common.maximum_health_points);
            self.sp_text = format!("{} / {}", player.spell_points, player.maximum_spell_points);
            self.level_text = player.base_level.to_string();
            self.hp_ratio = if common.maximum_health_points == 0 {
                0.0
            } else {
                (common.health_points as f32 / common.maximum_health_points as f32).clamp(0.0, 1.0)
            };
            self.sp_ratio = if player.maximum_spell_points == 0 {
                0.0
            } else {
                (player.spell_points as f32 / player.maximum_spell_points as f32).clamp(0.0, 1.0)
            };
        }
        let expanded = self.secondary_row.expanded.get();
        with_single_resolver(resolvers, |resolver| {
            let area = resolver.with_height(if expanded { EXPANDED_HEIGHT } else { COLLAPSED_HEIGHT });
            for (index, slot) in self.slots.iter_mut().enumerate() {
                if Self::visible_slot(index, expanded) {
                    let mut slot_resolver = Resolver::new(Self::slot_area(area, index), 0.0, resolver.get_text_layouter());
                    slot.create_layout_info(state, store.child_store(index as u64), &mut slot_resolver);
                }
            }
            ActionHudLayout { area, expanded }
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
        let band_height = if info.expanded { 164.0 } else { 114.0 };
        frame_band(
            layout,
            &self.frame,
            Area {
                height: band_height,
                ..area
            },
            0.0,
            0.0,
        );
        // Reuse the original narrow footer under the visible row. The EXP
        // artwork is drawn exclusively by ExperienceWindow.
        frame_band(
            layout,
            &self.frame,
            Area {
                left: area.left + 82.0,
                top: area.top + band_height,
                width: 454.0,
                height: 4.0,
            },
            82.0,
            164.0,
        );

        let drag_area = Area { height: 66.0, ..area };
        if drag_area.check().run(layout) {
            layout.register_click_handler(MouseButton::Left, &self.drag);
        }

        // Layout batches textures after rectangles within each layer.
        // Live fills must be in the layer above the opaque generated skin.
        layout.with_layer(|layout| {
            self.vitals(layout, area);
            for (index, slot) in self.slots.iter().enumerate() {
                if Self::visible_slot(index, info.expanded) {
                    slot.lay_out(state, store.child_store(index as u64), &(), layout);
                }
            }

            let button = Area {
                left: area.left + 555.0,
                top: area.top + if info.expanded { 140.0 } else { 90.0 },
                width: 23.0,
                height: 21.0,
            };
            let hovered = button.check().run(layout);
            if hovered {
                layout.register_click_handler(MouseButton::Left, &self.secondary_row);
            }
            layout.add_rectangle(
                button,
                CornerDiameter::uniform(5.0),
                Color::rgb_u8(179, 184, 187),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_rectangle(
                Area {
                    left: button.left + 2.0,
                    top: button.top + 2.0,
                    width: button.width - 4.0,
                    height: button.height - 4.0,
                },
                CornerDiameter::uniform(4.0),
                if hovered {
                    Color::rgb_u8(67, 74, 78)
                } else {
                    Color::rgb_u8(23, 26, 31)
                },
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_rectangle(
                Area { left: button.left + 4.0, top: button.top + 3.0, width: button.width - 8.0, height: 2.0 },
                CornerDiameter::uniform(1.0),
                Color::rgb_u8(105, 114, 119),
                Color::TRANSPARENT,
                ShadowPadding::uniform(0.0),
            );
            layout.add_icon(
                Area {
                    left: button.left + 6.0,
                    top: button.top + 5.0,
                    width: 11.0,
                    height: 11.0,
                },
                Icon::ExpandArrow { expanded: info.expanded },
                if hovered {
                    Color::rgb_u8(248, 249, 247)
                } else {
                    Color::rgb_u8(207, 216, 218)
                },
            );
        });
    }
}

/// Separate draggable module with its own cached position.
pub struct ExperienceWindow {
    frame: Arc<Texture>,
}

impl ExperienceWindow {
    pub fn new(frame: Arc<Texture>) -> Self {
        Self { frame }
    }
}

impl CustomWindow<ClientState> for ExperienceWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Experience)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;
        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            title_gap: 0.0,
            closable: false,
            resizable: false,
            border: 0.0,
            gaps: 0.0,
            minimum_width: HUD_WIDTH,
            maximum_width: HUD_WIDTH,
            minimum_height: EXPERIENCE_HEIGHT,
            maximum_height: EXPERIENCE_HEIGHT,
            background_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
            elements: ExperienceBody { frame: self.frame, drag: HudDrag::default(), text: String::new(), ratio: 0.0 },
        }
    }
}

struct ExperienceBody {
    frame: Arc<Texture>,
    drag: HudDrag,
    text: String,
    ratio: f32,
}

impl Element<ClientState> for ExperienceBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        store: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.drag.window_id.set(store.get_window_id());
        if let Some(player) = state.try_get(&this_player()) {
            self.ratio = if player.next_base_experience == 0 {
                0.0
            } else {
                (player.base_experience as f64 / player.next_base_experience as f64).clamp(0.0, 1.0) as f32
            };
            self.text = format!("Base EXP {:.0}%", self.ratio * 100.0);
        }
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(EXPERIENCE_HEIGHT),
        })
    }

    fn lay_out<'a>(
        &'a self,
        _: &'a Context<ClientState>,
        _: ElementStore<'a>,
        info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let area = info.area;
        frame_band(
            layout,
            &self.frame,
            Area {
                top: area.top + 12.0,
                height: 31.0,
                ..area
            },
            0.0,
            169.0,
        );
        if area.check().run(layout) {
            layout.register_click_handler(MouseButton::Left, &self.drag);
        }

        layout.with_layer(|layout| {
            draw_text(
                layout,
                Area {
                    left: area.left + 225.0,
                    top: area.top,
                    width: 150.0,
                    height: 13.0,
                },
                &self.text,
                11.0,
                Color::rgb_u8(238, 225, 196),
            );
            let exp_left = area.left + 62.0;
            let gap = 3.0;
            let segment_width = (476.0 - gap * (EXPERIENCE_SEGMENTS - 1) as f32) / EXPERIENCE_SEGMENTS as f32;
            for index in 0..EXPERIENCE_SEGMENTS {
                let left = exp_left + index as f32 * (segment_width + gap);
                let segment = Area {
                    left,
                    top: area.top + 18.0,
                    width: segment_width,
                    height: 10.0,
                };
                layout.add_rectangle(
                    segment,
                    CornerDiameter::uniform(1.0),
                    Color::rgba_u8(12, 14, 16, 220),
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
                let fill = (self.ratio * EXPERIENCE_SEGMENTS as f32 - index as f32).clamp(0.0, 1.0);
                if fill > 0.0 {
                    layout.add_rectangle(
                        Area {
                            left: left + 1.0,
                            top: segment.top + 1.0,
                            width: (segment_width - 2.0) * fill,
                            height: 8.0,
                        },
                        CornerDiameter::uniform(1.0),
                        Color::rgba_u8(41, 194, 93, 220),
                        Color::TRANSPARENT,
                        ShadowPadding::uniform(0.0),
                    );
                }
            }
        });
    }
}
