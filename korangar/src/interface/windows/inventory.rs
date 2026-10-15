use std::sync::Arc;

use korangar_components::item_box;
use korangar_interface::window::{CustomWindow, Window};
use korangar_networking::{InventoryItem, InventoryItemDetails};
use rust_state::{Path, PathExt, Selector};

use crate::ItemSource;
use crate::graphics::{Color, Texture};
use crate::interface::windows::WindowClass;
use super::themed_panel::{PanelStyle, ThemedPanel};
use crate::state::theme::InterfaceThemeType;
use crate::state::ClientState;
use crate::world::ResourceMetadata;

struct UnequippedItemPath<P> {
    items_path: P,
    visible_index: usize,
}

impl<P: Copy> Copy for UnequippedItemPath<P> {}
impl<P: Copy> Clone for UnequippedItemPath<P> {
    fn clone(&self) -> Self { *self }
}

impl<P> Selector<ClientState, InventoryItem<ResourceMetadata>, false> for UnequippedItemPath<P>
where
    P: Path<ClientState, Vec<InventoryItem<ResourceMetadata>>>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a InventoryItem<ResourceMetadata>> {
        self.follow(state)
    }
}

impl<P> Path<ClientState, InventoryItem<ResourceMetadata>, false> for UnequippedItemPath<P>
where
    P: Path<ClientState, Vec<InventoryItem<ResourceMetadata>>>,
{
    fn follow<'a>(&self, state: &'a ClientState) -> Option<&'a InventoryItem<ResourceMetadata>> {
        self.items_path.follow_safe(state).iter().filter(|item| match &item.details {
            InventoryItemDetails::Regular { equipped_position, .. } | InventoryItemDetails::Equippable { equipped_position, .. } => {
                equipped_position.is_empty()
            }
        }).nth(self.visible_index)
    }

    fn follow_mut<'a>(&self, state: &'a mut ClientState) -> Option<&'a mut InventoryItem<ResourceMetadata>> {
        self.items_path.follow_mut_safe(state).iter_mut().filter(|item| match &item.details {
            InventoryItemDetails::Regular { equipped_position, .. } | InventoryItemDetails::Equippable { equipped_position, .. } => {
                equipped_position.is_empty()
            }
        }).nth(self.visible_index)
    }
}

pub struct InventoryWindow<P> {
    items_path: P,
    frame: Arc<Texture>,
}

impl<P> InventoryWindow<P> {
    pub fn new(items_path: P, frame: Arc<Texture>) -> Self {
        Self { items_path, frame }
    }
}

impl<P> CustomWindow<ClientState> for InventoryWindow<P>
where
    P: Path<ClientState, Vec<InventoryItem<ResourceMetadata>>>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Inventory)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        // TODO: Probably this should be more dynamic
        const INVENTORY_ROWS: usize = 4;
        const INVENTORY_COLUMNS: usize = 10;

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 340.0,
            maximum_width: 340.0,
            minimum_height: 250.0,
            maximum_height: 250.0,
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
                PanelStyle { header: 36.0, inset: 25.0, footer: 15.0, minimum_height: 250.0,
                    title_left: 45.0, title_width: 250.0, close_right: 9.0, close_top: 11.0 },
                |_| "Inventory".to_owned(),
                ErasedElement::new(std::array::from_fn::<_, INVENTORY_ROWS, _>(|row| {
                split! {
                    gaps: theme().window().gaps(),
                    children: std::array::from_fn::<_, INVENTORY_COLUMNS, _>(|column| {
                        let path = UnequippedItemPath {
                            items_path: self.items_path,
                            visible_index: row * INVENTORY_COLUMNS + column,
                        };

                        item_box! {
                            item_path: path,
                            source: ItemSource::Inventory,
                        }
                    }),
                }
                })),
            ),
        }
    }
}
