use std::sync::Arc;

use korangar_interface::components::text_box::DefaultHandler;
use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::event::{ClickHandler, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{MouseButton, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::Sex;
use rust_state::{Context, Path, PathExt};

use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

const MINIMUM_NAME_LENGTH: usize = 4;
const MAXIMUM_NAME_LENGTH: usize = 24;

enum AppearanceAction {
    Female,
    Male,
    StylePrevious,
    StyleNext,
    ColorPrevious,
    ColorNext,
}

impl ClickHandler<ClientState> for AppearanceAction {
    fn handle_click(&self, state: &Context<ClientState>, _: &mut EventQueue<ClientState>) {
        match self {
            Self::Female => state.update_value(client_state().create_character_sex(), Sex::Female),
            Self::Male => state.update_value(client_state().create_character_sex(), Sex::Male),
            Self::StylePrevious => state.update_value_with(client_state().create_character_hair_style(), |value| {
                *value = value.saturating_sub(1)
            }),
            Self::StyleNext => state.update_value_with(client_state().create_character_hair_style(), |value| {
                *value = (*value + 1).min(42)
            }),
            Self::ColorPrevious => state.update_value_with(client_state().create_character_hair_color(), |value| {
                *value = value.saturating_sub(1)
            }),
            Self::ColorNext => state.update_value_with(client_state().create_character_hair_color(), |value| {
                *value = (*value + 1).min(8)
            }),
        }
    }
}

struct AppearancePanel {
    female: Arc<Texture>,
    male: Arc<Texture>,
    labels: [String; 2],
    actions: [AppearanceAction; 6],
}

impl AppearancePanel {
    fn new(female: Arc<Texture>, male: Arc<Texture>) -> Self {
        Self {
            female,
            male,
            labels: [String::new(), String::new()],
            actions: [
                AppearanceAction::Female,
                AppearanceAction::Male,
                AppearanceAction::StylePrevious,
                AppearanceAction::StyleNext,
                AppearanceAction::ColorPrevious,
                AppearanceAction::ColorNext,
            ],
        }
    }
}

impl Element<ClientState> for AppearancePanel {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.labels[0] = format!("Hair style  ·  {}", state.get(&client_state().create_character_hair_style()));
        self.labels[1] = format!("Hair color  ·  {}", state.get(&client_state().create_character_hair_color()));
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(418.0),
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a Context<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let area = layout_info.area;
        let preview = Area {
            left: area.left + 7.0,
            top: area.top + 7.0,
            width: area.width * 0.48 - 12.0,
            height: area.height - 14.0,
        };
        let options = Area {
            left: preview.left + preview.width + 16.0,
            top: area.top + 7.0,
            width: area.width - preview.width - 30.0,
            height: area.height - 14.0,
        };
        let border = Color::rgb_u8(74, 115, 93);
        layout.add_rectangle(
            preview,
            CornerDiameter::uniform(10.0),
            Color::rgba_u8(8, 22, 20, 210),
            border,
            ShadowPadding::uniform(0.0),
        );
        layout.add_rectangle(
            options,
            CornerDiameter::uniform(10.0),
            Color::rgba_u8(8, 22, 20, 235),
            border,
            ShadowPadding::uniform(0.0),
        );
        let female = *state.get(&client_state().create_character_sex()) == Sex::Female;
        let portrait = if female { &self.female } else { &self.male };
        layout.add_texture(
            Area {
                left: preview.left + (preview.width - 256.0) / 2.0,
                top: preview.top + 15.0,
                width: 256.0,
                height: 384.0,
            },
            portrait.clone(),
            Color::WHITE,
            false,
        );
        text(layout, preview, "YOUR ADVENTURER", 20.0, 8.0, Color::rgb_u8(234, 232, 196));
        text(layout, options, "APPEARANCE", 20.0, 15.0, Color::rgb_u8(234, 232, 196));
        text(
            layout,
            options,
            "Starting class: Novice",
            16.0,
            58.0,
            Color::rgb_u8(205, 222, 204),
        );
        text(layout, options, "Sex", 16.0, 106.0, Color::rgb_u8(205, 222, 204));

        let female_area = Area {
            left: options.left + 18.0,
            top: options.top + 135.0,
            width: (options.width - 48.0) / 2.0,
            height: 34.0,
        };
        let male_area = Area {
            left: female_area.left + female_area.width + 12.0,
            top: female_area.top,
            width: female_area.width,
            height: 34.0,
        };
        self.option(layout, female_area, "Female", female, &self.actions[0]);
        self.option(layout, male_area, "Male", !female, &self.actions[1]);
        for index in 0..2 {
            let top = options.top + 207.0 + index as f32 * 78.0;
            let label_area = Area {
                left: options.left + 18.0,
                top,
                width: options.width - 36.0,
                height: 25.0,
            };
            text(layout, label_area, &self.labels[index], 16.0, 0.0, Color::rgb_u8(205, 222, 204));
            let left = Area {
                left: options.left + 18.0,
                top: top + 31.0,
                width: 58.0,
                height: 31.0,
            };
            let right = Area {
                left: options.left + options.width - 76.0,
                top: top + 31.0,
                width: 58.0,
                height: 31.0,
            };
            self.option(layout, left, "‹", false, &self.actions[2 + index * 2]);
            self.option(layout, right, "›", false, &self.actions[3 + index * 2]);
        }
        text(
            layout,
            options,
            "Appearance preview uses current FreokRO art.",
            12.0,
            372.0,
            Color::rgb_u8(158, 185, 166),
        );
    }
}

impl AppearancePanel {
    fn option<'a>(
        &'a self,
        layout: &mut WindowLayout<'a, ClientState>,
        area: Area,
        label: &'a str,
        active: bool,
        action: &'a AppearanceAction,
    ) {
        let hovered = area.check().run(layout);
        let color = if active {
            Color::rgba_u8(35, 100, 69, 245)
        } else if hovered {
            Color::rgba_u8(45, 74, 59, 240)
        } else {
            Color::rgba_u8(24, 47, 39, 230)
        };
        layout.add_rectangle(
            area,
            CornerDiameter::uniform(5.0),
            color,
            Color::rgb_u8(76, 125, 96),
            ShadowPadding::uniform(0.0),
        );
        layout.add_text(
            area,
            label,
            FontSize(16.0),
            Color::rgb_u8(235, 236, 212),
            Color::rgb_u8(186, 230, 182),
            HorizontalAlignment::Center { offset: 0.0, border: 4.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
        if hovered {
            layout.register_click_handler(MouseButton::Left, action);
        }
    }
}

fn text<'a>(layout: &mut WindowLayout<'a, ClientState>, area: Area, label: &'a str, size: f32, top: f32, color: Color) {
    layout.add_text(
        area,
        label,
        FontSize(size),
        color,
        Color::rgb_u8(186, 230, 182),
        HorizontalAlignment::Center { offset: 0.0, border: 8.0 },
        VerticalAlignment::Top { offset: top },
        OverflowBehavior::Shrink,
    );
}

pub struct CharacterCreationWindow<A> {
    character_name_path: A,
    slot: usize,
    female: Arc<Texture>,
    male: Arc<Texture>,
}

impl<A> CharacterCreationWindow<A> {
    pub fn new(character_name_path: A, slot: usize, female: Arc<Texture>, male: Arc<Texture>) -> Self {
        Self {
            character_name_path,
            slot,
            female,
            male,
        }
    }
}

impl<A> CustomWindow<ClientState> for CharacterCreationWindow<A>
where
    A: Path<ClientState, String>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::CharacterCreation)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        struct CharacterName;
        let disabled = ComputedSelector::new_default(move |state: &ClientState| {
            self.character_name_path.follow_safe(state).len() < MINIMUM_NAME_LENGTH
        });
        let create_action = move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
            let name = state.get(&self.character_name_path).clone();
            queue.queue(InputEvent::CreateCharacter {
                slot: self.slot,
                name,
                sex: *state.get(&client_state().create_character_sex()),
                hair_style: *state.get(&client_state().create_character_hair_style()),
                hair_color: *state.get(&client_state().create_character_hair_color()),
            });
        };
        window! {
            title: "FreokRO · Create Character",
            class: Self::window_class(),
            theme: InterfaceThemeType::Menu,
            background_color: Color::rgba_u8(5, 18, 16, 100),
            minimum_width: 940.0,
            maximum_width: 940.0,
            closable: true,
            elements: (
                AppearancePanel::new(self.female, self.male),
                text_box! {
                    ghost_text: client_state().localization().character_name_text(),
                    state: self.character_name_path,
                    input_handler: DefaultHandler::<_, _, MAXIMUM_NAME_LENGTH>::new(self.character_name_path, create_action),
                    focus_id: CharacterName,
                    overflow_behavior: OverflowBehavior::Shrink,
                },
                button! {
                    text: client_state().localization().create_character_button_text(),
                    disabled,
                    disabled_tooltip: client_state().localization().create_character_button_tooltip(),
                    event: create_action,
                },
            ),
        }
    }
}
