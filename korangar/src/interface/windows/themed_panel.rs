use std::cell::Cell;
use std::sync::Arc;

use korangar_interface::MouseMode;
use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{Element, ElementBox};
use korangar_interface::event::{ClickHandler, Event, EventQueue};
use korangar_interface::layout::alignment::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{MouseButton, Resolver, Resolvers, WindowLayout, with_single_resolver};
use rust_state::Context;

use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::ClientState;

#[derive(Clone, Copy)]
pub struct PanelStyle {
    pub header: f32,
    pub inset: f32,
    pub footer: f32,
    pub minimum_height: f32,
    pub title_left: f32,
    pub title_width: f32,
    pub close_right: f32,
    pub close_top: f32,
}

#[derive(Default)]
struct PanelDrag {
    window_id: Cell<u64>,
}

impl ClickHandler<ClientState> for PanelDrag {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(Event::SetMouseMode {
            mouse_mode: MouseMode::MovingWindow {
                window_id: self.window_id.get(),
            },
        });
    }
}

#[derive(Default)]
struct PanelClose {
    window_id: Cell<u64>,
}

impl ClickHandler<ClientState> for PanelClose {
    fn handle_click(&self, _: &Context<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(Event::CloseWindow {
            window_id: self.window_id.get(),
        });
    }
}

pub struct ThemedPanel {
    frame: Arc<Texture>,
    style: PanelStyle,
    gaps: f32,
    title: Box<dyn Fn(&Context<ClientState>) -> String>,
    title_text: String,
    content: ElementBox<ClientState>,
    drag: PanelDrag,
    close: PanelClose,
}

impl ThemedPanel {
    pub fn with_gaps(mut self, gaps: f32) -> Self {
        self.gaps = gaps;
        self
    }

    pub fn new(
        frame: Arc<Texture>,
        style: PanelStyle,
        title: impl Fn(&Context<ClientState>) -> String + 'static,
        content: ElementBox<ClientState>,
    ) -> Self {
        Self {
            frame,
            style,
            gaps: 0.0,
            title: Box::new(title),
            title_text: String::new(),
            content,
            drag: PanelDrag::default(),
            close: PanelClose::default(),
        }
    }
}

pub struct PanelLayout {
    area: Area,
}

impl Element<ClientState> for ThemedPanel {
    type LayoutInfo = PanelLayout;

    fn create_layout_info(
        &mut self,
        state: &Context<ClientState>,
        mut store: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        self.drag.window_id.set(store.get_window_id());
        self.close.window_id.set(store.get_window_id());
        self.title_text = (self.title)(state);
        with_single_resolver(resolvers, |resolver| {
            let available = resolver.push_available_area();
            let content_area = Area {
                left: available.left + self.style.inset,
                top: available.top + self.style.header,
                width: available.width - self.style.inset * 2.0,
                height: available.height.unwrap_or(self.style.minimum_height),
            };
            let mut content_resolver = Resolver::new(content_area, self.gaps, resolver.get_text_layouter());
            self.content.create_layout_info(state, store.child_store(0), &mut content_resolver);
            let content_height = content_resolver.push_available_area().top - content_area.top;
            let height = (self.style.header + content_height + self.style.footer).max(self.style.minimum_height);
            PanelLayout {
                area: resolver.with_height(height),
            }
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
        layout.add_texture(area, self.frame.clone(), Color::WHITE, true);

        let close_area = Area {
            left: area.left + area.width - self.style.close_right - 18.0,
            top: area.top + self.style.close_top,
            width: 18.0,
            height: 18.0,
        };
        let close_hovered = close_area.check().run(layout);
        if close_hovered {
            layout.register_click_handler(MouseButton::Left, &self.close);
        }
        let header_area = Area {
            height: self.style.header,
            ..area
        };
        if header_area.check().run(layout) && !close_hovered {
            layout.register_click_handler(MouseButton::Left, &self.drag);
        }

        layout.with_layer(|layout| {
            self.content.lay_out(state, store.child_store(0), &(), layout);
            layout.add_text(
                Area {
                    left: area.left + self.style.title_left,
                    top: area.top + self.style.close_top + 4.0,
                    width: self.style.title_width,
                    height: 22.0,
                },
                &self.title_text,
                FontSize(13.0),
                Color::rgb_u8(244, 225, 201),
                Color::rgb_u8(255, 191, 116),
                HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
                VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );
            if close_hovered {
                layout.add_rectangle(
                    close_area,
                    CornerDiameter::uniform(12.0),
                    Color::rgba_u8(155, 32, 47, 105),
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
            }
        });
    }
}
