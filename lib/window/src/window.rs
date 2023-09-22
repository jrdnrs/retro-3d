use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

use crate::internal::{GraphicsContext, InternalWindow};

pub struct Window {
    pub(crate) internal: InternalWindow,
    pub(crate) attributes: WindowAttributes,
    pub(crate) event_loop: Option<EventLoop<()>>,
}

impl Window {
    pub fn new(config: WindowAttributes) -> Self {
        let window_builder = WindowBuilder::new()
            .with_title(config.title.to_owned())
            .with_resizable(config.resizable)
            .with_inner_size::<PhysicalSize<u32>>(config.size.into())
            .with_position::<PhysicalPosition<u32>>(config.position.into())
            .with_fullscreen(config.fullscreen.then(|| Fullscreen::Borderless(None)));

        let (mut internal, event_loop) = InternalWindow::new(window_builder);

        // Set other window attributes not handled by the window builder
        internal.set_minimised(config.minimised);
        internal.set_maximised(config.maximised);
        internal.set_min_window_size(config.min_size);
        internal.set_max_window_size(config.max_size);
        if config.focused {
            internal.focus()
        };

        config.surface_size.map(|size| internal.set_surface_size(size));


        Self {
            internal,
            attributes: config,
            event_loop: Some(event_loop),
        }
    }

    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take().expect("Event loop already taken")
    }

    pub fn set_title(&mut self, title: &str) {
        if self.attributes.title == title {
            return;
        };
        self.attributes.title = title.to_owned();
        self.internal.set_title(title);
    }

    pub fn get_title(&self) -> &str {
        &self.attributes.title
    }

    pub fn set_size(&mut self, size: WindowSize) {
        if self.attributes.size == size {
            return;
        }

        self.attributes.size = size;
        self.internal.set_window_size(size);
    }

    pub fn get_size(&self) -> WindowSize {
        self.attributes.size
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        if self.attributes.resizable == resizable {
            return;
        }

        self.attributes.resizable = resizable;
        self.internal.set_resizable(resizable);
    }

    pub fn get_resizable(&self) -> bool {
        self.attributes.resizable
    }

    pub fn set_maximised(&mut self, maximised: bool) {
        if self.attributes.maximised == maximised {
            return;
        }

        self.attributes.maximised = maximised;
        self.internal.set_maximised(maximised);
    }

    pub fn get_maximised(&self) -> bool {
        self.attributes.maximised
    }

    pub fn get_minimised(&self) -> bool {
        self.attributes.minimised
    }

    pub fn set_minimised(&mut self, minimised: bool) {
        if self.attributes.minimised == minimised {
            return;
        }

        self.attributes.minimised = minimised;
        self.internal.set_minimised(minimised);
    }

    pub fn focus(&mut self) {
        self.internal.focus();
    }

    pub fn set_cursor_grab(&mut self, grabbed: bool) {
        self.internal.set_cursor_grab(grabbed);
        self.attributes.grabbed_cursor = grabbed;
    }

    pub fn get_cursor_grab(&self) -> bool {
        self.attributes.grabbed_cursor
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.internal.set_cursor_visible(visible);
        self.attributes.visible_cursor = visible;
    }

    pub fn get_cursor_visible(&self) -> bool {
        self.attributes.visible_cursor
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.internal.set_fullscreen(fullscreen);
    }

    pub fn get_fullscreen(&self) -> bool {
        self.attributes.fullscreen
    }

    pub fn graphics_context(&mut self) -> GraphicsContext {
        self.internal.graphics_context()
    }

    pub fn request_redraw(&self) {
        self.internal.request_redraw();
    }

    pub fn swap_buffers(&mut self) {
        self.internal.swap_buffers();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowSize {
    pub width: usize,
    pub height: usize,
}

impl WindowSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl From<WindowSize> for winit::dpi::PhysicalSize<u32> {
    fn from(value: WindowSize) -> Self {
        winit::dpi::PhysicalSize::new(value.width as u32, value.height as u32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPosition {
    pub x: usize,
    pub y: usize,
}

impl WindowPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<WindowPosition> for winit::dpi::PhysicalPosition<u32> {
    fn from(value: WindowPosition) -> Self {
        winit::dpi::PhysicalPosition::new(value.x as u32, value.y as u32)
    }
}

pub struct WindowAttributes {
    pub title: String,
    pub resizable: bool,
    pub maximised: bool,
    pub minimised: bool,
    pub focused: bool,
    pub fullscreen: bool,
    pub position: WindowPosition,
    pub size: WindowSize,
    pub max_size: Option<WindowSize>,
    pub min_size: Option<WindowSize>,
    pub surface_size: Option<WindowSize>,
    pub grabbed_cursor: bool,
    pub visible_cursor: bool,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: String::from("Espresso App"),
            resizable: true,
            maximised: false,
            minimised: false,
            focused: false,
            fullscreen: false,
            position: WindowPosition::new(0, 0),
            size: WindowSize::new(640, 480),
            max_size: None,
            min_size: None,
            surface_size: None,
            grabbed_cursor: false,
            visible_cursor: true,
        }
    }
}
