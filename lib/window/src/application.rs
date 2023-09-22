use crate::{event::Event, Window, WindowPosition, WindowSize};

use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};

pub trait WindowApplication: Sized + 'static {
    fn get_window(&self) -> &Window;
    fn get_window_mut(&mut self) -> &mut Window;
    fn on_event(&mut self, event: &Event);

    /// Default implementation of `run` for `WindowApplication` handles the backend event loop
    /// and calls the user defined event handler `on_event`.
    ///
    /// Typical window functions are handled by this, such as resizing, moving, closing, etc.
    fn run(mut self) -> ! {
        let event_loop = self.get_window_mut().take_event_loop();

        event_loop.run(move |event, window_target, control_flow| {
            // TODO: Allow user to define control flow (default is Poll)

            // Call user defined event handler
            let user_event = Event::try_from_winit_event(&event);
            let _ = user_event.map(|event| self.on_event(&event));

            // Native handling of winit events
            match event {
                WinitEvent::LoopDestroyed => {}

                WinitEvent::Resumed => {}

                WinitEvent::Suspended => {}

                WinitEvent::DeviceEvent { event, .. } => match event {
                    _ => (),
                },

                WinitEvent::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::Resized(ref size) => {
                        let width = size.width as usize;
                        let height = size.height as usize;

                        let window = self.get_window_mut();

                        if size.width == 0 && size.height == 0 {
                            window.attributes.minimised = true;
                        } else {
                            window.attributes.minimised = false;
                            window.attributes.size = WindowSize::new(width, height);

                            if window.attributes.surface_size.is_none() {
                                window.internal.set_surface_size(window.attributes.size);
                            }
                        }
                    }

                    WinitWindowEvent::Moved(ref position) => {
                        self.get_window_mut().attributes.position =
                            WindowPosition::new(position.x as usize, position.y as usize);
                    }

                    WinitWindowEvent::Focused(focused) => {
                        self.get_window_mut().attributes.focused = focused;
                    }

                    WinitWindowEvent::CloseRequested => control_flow.set_exit(),

                    _ => {}
                },

                WinitEvent::MainEventsCleared => {
                    self.get_window().request_redraw();
                }

                WinitEvent::RedrawRequested(_) => {
                    self.get_window_mut().swap_buffers();
                }

                _ => (),
            }
        });
    }
}
