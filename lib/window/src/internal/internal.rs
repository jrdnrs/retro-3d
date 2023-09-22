use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{WindowPosition, WindowSize};

pub type GraphicsContext<'a> = super::software::GraphicsContext<'a>;

pub enum InternalWindow {
    Software(super::software::SoftWindow),
}

impl InternalWindow {
    pub fn new(window_builder: WindowBuilder) -> (Self, EventLoop<()>) {
        let (window, event_loop) = super::software::SoftWindow::new(window_builder);
        return (Self::Software(window), event_loop);
    }

    pub fn set_title(&mut self, title: &str) {
        match self {
            Self::Software(window) => window.set_title(title),
        }
    }

    pub fn set_min_window_size(&mut self, size: Option<WindowSize>) {
        match self {
            Self::Software(window) => window.set_min_window_size(size),
        }
    }

    pub fn set_max_window_size(&mut self, size: Option<WindowSize>) {
        match self {
            Self::Software(window) => window.set_max_window_size(size),
        }
    }

    pub fn set_window_size(&mut self, size: WindowSize) {
        match self {
            Self::Software(window) => window.set_window_size(size),
        }
    }

    pub fn set_surface_size(&mut self, size: WindowSize) {
        match self {
            Self::Software(window) => window.set_surface_size(size),
        }
    }

    pub fn set_position(&mut self, position: WindowPosition) {
        match self {
            Self::Software(window) => window.set_position(position),
        }
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        match self {
            Self::Software(window) => window.set_resizable(resizable),
        }
    }

    pub fn set_minimised(&mut self, minimised: bool) {
        match self {
            Self::Software(window) => window.set_minimised(minimised),
        }
    }

    pub fn focus(&mut self) {
        match self {
            Self::Software(window) => window.focus(),
        }
    }

    pub fn swap_buffers(&mut self) {
        match self {
            Self::Software(window) => window.swap_buffers(),
        }
    }

    pub fn set_maximised(&mut self, maximised: bool) {
        match self {
            Self::Software(window) => window.set_maximised(maximised),
        }
    }

    pub fn request_redraw(&self) {
        match self {
            Self::Software(window) => window.request_redraw(),
        }
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        match self {
            Self::Software(window) => window.set_fullscreen(fullscreen),
        }
    }

    pub fn set_cursor_grab(&mut self, grabbed: bool) {
        match self {
            Self::Software(window) => window.set_cursor_grab(grabbed),
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        match self {
            Self::Software(window) => window.set_cursor_visible(visible),
        }
    }

    pub fn graphics_context(&mut self) -> GraphicsContext {
        match self {
            Self::Software(window) => window.graphics_context(),
        }
    }
}
