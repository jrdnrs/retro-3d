use core::num::NonZeroU32;

use crate::{WindowSize, WindowPosition};

pub struct GraphicsContext<'a> {
    framebuffer: softbuffer::Buffer<'a>,
}

impl<'a> GraphicsContext<'a> {
    pub fn new(surface: &'a mut softbuffer::Surface) -> Self {
        let buffer = surface.buffer_mut().unwrap_or_else(|e| {
            panic!("Failed to get buffer: {}", e);
        });

        Self {
            framebuffer: buffer,
        }
    }

    pub fn framebuffer(&self) -> &[u32] {
        &*self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut [u32] {
        &mut *self.framebuffer
    }
}

pub struct SoftWindow {
    pub(crate) surface: softbuffer::Surface,
    pub(crate) context: softbuffer::Context,
    pub(crate) winit_window: winit::window::Window,
}

impl SoftWindow {
    pub fn new(
        window_builder: winit::window::WindowBuilder,
    ) -> (Self, winit::event_loop::EventLoop<()>) {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = window_builder.build(&event_loop).unwrap();
        let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
        let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

        surface
            .resize(
                NonZeroU32::new(window.inner_size().width)
                    .expect("Width must be greater than zero"),
                NonZeroU32::new(window.inner_size().height)
                    .expect("Height must be greater than zero"),
            )
            .unwrap();

        (
            Self {
                surface,
                context,
                winit_window: window,
            },
            event_loop,
        )
    }

    pub fn set_title(&self, title: &str) {
        self.winit_window.set_title(title);
    }

    pub fn set_min_window_size(&mut self, size: Option<WindowSize>) {
        self.winit_window
            .set_min_inner_size(size.map(winit::dpi::PhysicalSize::from));
    }

    pub fn set_max_window_size(&mut self, size: Option<WindowSize>) {
        self.winit_window
            .set_max_inner_size(size.map(winit::dpi::PhysicalSize::from));
    }

    pub fn set_window_size(&mut self, size: WindowSize) {
        self.winit_window
            .set_inner_size(winit::dpi::PhysicalSize::from(size));
    }

    pub fn set_surface_size(&mut self, size: WindowSize) {
        self.surface
            .resize(
                NonZeroU32::new(size.width as u32).expect("Width must be greater than zero"),
                NonZeroU32::new(size.height as u32).expect("Height must be greater than zero"),
            )
            .unwrap();
    }

    pub fn set_position(&self, position: WindowPosition) {
        self.winit_window.set_outer_position(winit::dpi::PhysicalPosition::from(position));
    }

    pub fn set_resizable(&self, resizable: bool) {
        self.winit_window.set_resizable(resizable);
    }

    pub fn set_maximised(&self, maximised: bool) {
        self.winit_window.set_maximized(maximised);
    }

    pub fn set_minimised(&self, minimised: bool) {
        self.winit_window.set_minimized(minimised);
    }

    pub fn set_cursor_grab(&self, grabbed: bool) {
        if grabbed {
            self.winit_window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_| {
                    self.winit_window
                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                })
                .unwrap();
        } else {
            self.winit_window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .unwrap();
        }
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.winit_window.set_cursor_visible(visible);
    }

    pub fn focus(&self) {
        self.winit_window.focus_window();
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen {
            self.winit_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            self.winit_window.set_fullscreen(None);
        }
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    pub fn swap_buffers(&mut self) {
        let buffer = self.surface.buffer_mut().unwrap_or_else(|e| {
            panic!("Failed to get buffer: {}", e);
        });

        buffer.present().unwrap_or_else(|e| {
            panic!("Failed to present buffer: {}", e);
        });
    }

    pub fn graphics_context(&mut self) -> GraphicsContext {
        GraphicsContext::new(&mut self.surface)
    }
}
