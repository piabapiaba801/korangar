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
use ragnarok_packets::Sex;
use rust_state::{Context, Path};

use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::ClientState;
use crate::state::character_slots::CharacterSlots;
use crate::state::theme::InterfaceThemeType;

const MAX_VISIBLE_SLOTS: usize = 15;

struct SelectSlot<M> {
    index: usize,
    selected: Rc<Cell<usize>>,
    pending_delete: Rc<Cell<Option<usize>>>,
    switch_request: M,
}

impl<M> ClickHandler<ClientState> for SelectSlot<M>
where
    M: Path<ClientState, Option<usize>>,
{
    fn handle_click(&self, state: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        if let Some(origin_slot) = *state.get(&self.switch_request) {
            if origin_slot == self.index {
                state.update_value(self.switch_request, None);
            } else {
                queue.queue(InputEvent::SwitchCharacterSlot {
                    origin_slot,
                    destination_slot: self.index,
                });
            }
        } else {
            self.selected.set(self.index);
            self.pending_delete.set(None);
        }
    }
}

struct CharacterStage<C, M> {
    character_slots: C,
    switch_request: M,
    selected: Rc<Cell<usize>>,
    pending_delete: Rc<Cell<Option<usize>>>,
    female: Arc<Texture>,
    male: Arc<Texture>,
    slot_handlers: Vec<SelectSlot<M>>,
    slot_labels: Vec<String>,
    selected_details: String,
}

impl<C, M> CharacterStage<C, M>
where
    C: Path<ClientState, CharacterSlots>,
    M: Path<ClientState, Option<usize>>,
{
    fn new(
        character_slots: C,
        switch_request: M,
        selected: Rc<Cell<usize>>,
        pending_delete: Rc<Cell<Option<usize>>>,
        female: Arc<Texture>,
        male: Arc<Texture>,
    ) -> Self {
        let slot_handlers = (0..MAX_VISIBLE_SLOTS)
            .map(|index| SelectSlot {
                index,
                selected: selected.clone(),
                pending_delete: pending_delete.clone(),
                switch_request,
            })
            .collect();
        Self {
            character_slots,
            switch_request,
            selected,
            pending_delete,
            female,
            male,
            slot_handlers,
            slot_labels: Vec::new(),
            selected_details: String::new(),
        }
    }
}

fn draw_text<'a>(
    layout: &mut WindowLayout<'a, ClientState>,
    area: Area,
    text: &'a str,
    size: f32,
    color: Color,
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
) {
    layout.add_text(
        area,
        text,
        FontSize(size),
        color,
        Color::rgb_u8(186, 230, 182),
        horizontal,
        vertical,
        OverflowBehavior::Shrink,
    );
}

impl<C, M> Element<ClientState> for CharacterStage<C, M>
where
    C: Path<ClientState, CharacterSlots>,
    M: Path<ClientState, Option<usize>>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        let slots = state.get(&self.character_slots);
        let count = slots.get_slot_count().min(MAX_VISIBLE_SLOTS);
        if self.selected.get() >= count {
            self.selected
                .set((0..count).find(|&index| slots.get_slot(index).is_some()).unwrap_or(0));
        }
        self.slot_labels = (0..count)
            .map(|index| match slots.get_slot(index) {
                Some(character) => format!("{:02}  {}  ·  Lv {}", index + 1, character.name, character.base_level),
                None => format!("{:02}  Create character", index + 1),
            })
            .collect();
        self.selected_details = slots.get_slot(self.selected.get()).map_or(String::new(), |character| {
            format!(
                "Job {}  ·  Base {} / Job {}  ·  {}",
                character.job_id.0,
                character.base_level,
                character.job_level,
                character.map_name.strip_suffix(".gat").unwrap_or(&character.map_name)
            )
        });
        with_single_resolver(resolvers, |resolver| BaseLayoutInfo {
            area: resolver.with_height(490.0),
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
        let hero = Area {
            left: area.left + 6.0,
            top: area.top + 6.0,
            width: area.width * 0.53 - 10.0,
            height: area.height - 12.0,
        };
        let roster = Area {
            left: hero.left + hero.width + 16.0,
            top: area.top + 6.0,
            width: area.width - hero.width - 28.0,
            height: area.height - 12.0,
        };
        let border = Color::rgb_u8(70, 111, 91);
        layout.add_rectangle(
            hero,
            CornerDiameter::uniform(10.0),
            Color::rgba_u8(8, 22, 20, 205),
            border,
            ShadowPadding::uniform(0.0),
        );
        layout.add_rectangle(
            roster,
            CornerDiameter::uniform(10.0),
            Color::rgba_u8(8, 22, 20, 225),
            border,
            ShadowPadding::uniform(0.0),
        );

        let selected = self.selected.get();
        let slots = state.get(&self.character_slots);
        let character = slots.get_slot(selected);
        let portrait = if character.is_some_and(|character| character.sex == Sex::Male) {
            &self.male
        } else {
            &self.female
        };
        let portrait_area = Area {
            left: hero.left + (hero.width - 256.0) / 2.0,
            top: hero.top + 32.0,
            width: 256.0,
            height: 384.0,
        };
        layout.add_texture(portrait_area, portrait.clone(), Color::WHITE, false);
        let title = character.map_or("A NEW ADVENTURE", |character| character.name.as_str());
        draw_text(
            layout,
            hero,
            title,
            25.0,
            Color::rgb_u8(235, 232, 196),
            HorizontalAlignment::Center { offset: 0.0, border: 12.0 },
            VerticalAlignment::Top { offset: 5.0 },
        );
        if character.is_some() {
            draw_text(
                layout,
                hero,
                &self.selected_details,
                15.0,
                Color::rgb_u8(190, 212, 198),
                HorizontalAlignment::Center { offset: 0.0, border: 12.0 },
                VerticalAlignment::Bottom { offset: 12.0 },
            );
        } else {
            draw_text(
                layout,
                hero,
                "Choose a name and appearance to begin.",
                16.0,
                Color::rgb_u8(190, 212, 198),
                HorizontalAlignment::Center { offset: 0.0, border: 12.0 },
                VerticalAlignment::Bottom { offset: 12.0 },
            );
        }

        draw_text(
            layout,
            roster,
            "YOUR CHARACTERS",
            20.0,
            Color::rgb_u8(235, 232, 196),
            HorizontalAlignment::Center { offset: 0.0, border: 10.0 },
            VerticalAlignment::Top { offset: 10.0 },
        );
        let moving = state.get(&self.switch_request).is_some();
        for (index, label) in self.slot_labels.iter().enumerate() {
            let row = Area {
                left: roster.left + 12.0,
                top: roster.top + 46.0 + index as f32 * 27.7,
                width: roster.width - 24.0,
                height: 25.0,
            };
            let hovered = row.check().run(layout);
            let active = index == selected;
            let color = if active {
                Color::rgba_u8(30, 80, 64, 245)
            } else if hovered {
                Color::rgba_u8(39, 62, 53, 230)
            } else {
                Color::rgba_u8(16, 34, 30, 210)
            };
            layout.add_rectangle(row, CornerDiameter::uniform(4.0), color, border, ShadowPadding::uniform(0.0));
            draw_text(
                layout,
                row,
                label,
                14.0,
                if active {
                    Color::rgb_u8(241, 234, 190)
                } else {
                    Color::rgb_u8(204, 221, 207)
                },
                HorizontalAlignment::Left { offset: 9.0, border: 4.0 },
                VerticalAlignment::Center { offset: 0.0 },
            );
            if hovered {
                layout.register_click_handler(MouseButton::Left, &self.slot_handlers[index]);
            }
        }
        if moving {
            draw_text(
                layout,
                roster,
                "Select a destination slot; click the source to cancel.",
                12.0,
                Color::rgb_u8(235, 205, 143),
                HorizontalAlignment::Center { offset: 0.0, border: 8.0 },
                VerticalAlignment::Bottom { offset: 5.0 },
            );
        } else if self.pending_delete.get() == Some(selected) {
            draw_text(
                layout,
                roster,
                "Click Delete again to confirm character removal.",
                12.0,
                Color::rgb_u8(235, 205, 143),
                HorizontalAlignment::Center { offset: 0.0, border: 8.0 },
                VerticalAlignment::Bottom { offset: 5.0 },
            );
        }
    }
}

pub struct CharacterSelectionWindow<C, M> {
    character_slots: C,
    switch_request: M,
    female: Arc<Texture>,
    male: Arc<Texture>,
}

impl<C, M> CharacterSelectionWindow<C, M> {
    pub fn new(character_slots: C, switch_request: M, female: Arc<Texture>, male: Arc<Texture>) -> Self {
        Self {
            character_slots,
            switch_request,
            female,
            male,
        }
    }
}

impl<C, M> CustomWindow<ClientState> for CharacterSelectionWindow<C, M>
where
    C: Path<ClientState, CharacterSlots>,
    M: Path<ClientState, Option<usize>>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::CharacterSelection)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let selected = Rc::new(Cell::new(usize::MAX));
        let pending_delete = Rc::new(Cell::new(None));
        let slots_path = self.character_slots;
        let switch_path = self.switch_request;
        let enter_selected = selected.clone();
        let create_selected = selected.clone();
        let move_selected = selected.clone();
        let delete_selected = selected.clone();
        let delete_pending = pending_delete.clone();

        window! {
            title: "FreokRO · Select Character",
            class: Self::window_class(),
            theme: InterfaceThemeType::Menu,
            background_color: Color::rgba_u8(5, 18, 16, 100),
            minimum_width: 1040.0,
            maximum_width: 1040.0,
            elements: (
                CharacterStage::new(slots_path, switch_path, selected, pending_delete, self.female, self.male),
                split! {
                    gaps: 8.0,
                    children: (
                        button! {
                            text: "Enter World",
                            event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                let slot = enter_selected.get();
                                if state.get(&slots_path).get_slot(slot).is_some() {
                                    queue.queue(InputEvent::SelectCharacter { slot });
                                }
                            },
                        },
                        button! {
                            text: "Create",
                            event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                let slots = state.get(&slots_path);
                                let slot = create_selected.get();
                                if slots.get_slot(slot).is_none() && slot < slots.get_slot_count() {
                                    queue.queue(InputEvent::OpenCharacterCreationWindow { slot });
                                } else if let Some(slot) = slots.first_empty_slot() {
                                    queue.queue(InputEvent::OpenCharacterCreationWindow { slot });
                                }
                            },
                        },
                        button! {
                            text: "Move",
                            event: move |state: &Context<ClientState>, _: &mut EventQueue<ClientState>| {
                                let slot = move_selected.get();
                                if state.get(&slots_path).get_slot(slot).is_some() {
                                    state.update_value(switch_path, Some(slot));
                                }
                            },
                        },
                        button! {
                            text: "Delete",
                            event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                let slot = delete_selected.get();
                                if let Some(character) = state.get(&slots_path).get_slot(slot) {
                                    if delete_pending.get() == Some(slot) {
                                        queue.queue(InputEvent::DeleteCharacter { character_id: character.character_id });
                                        delete_pending.set(None);
                                    } else {
                                        delete_pending.set(Some(slot));
                                    }
                                }
                            },
                        },
                        button! { text: "Log Out", event: InputEvent::LogOutCharacter },
                    ),
                },
            ),
        }
    }
}
