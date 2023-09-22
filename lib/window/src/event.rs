use crate::{WindowPosition, WindowSize};

pub enum Event {
    WindowEvent(WindowEvent),
    PointerEvent(PointerEvent),
    KeyboardEvent(KeyboardEvent),
    ClipboardEvent(ClipboardEvent),
    RenderEvent(RenderEvent),
}

impl Event {
    pub fn try_from_winit_event(value: &winit::event::Event<'_, ()>) -> Result<Self, &'static str> {
        match value {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    return Ok(Event::WindowEvent(WindowEvent::Resized(WindowSize::new(
                        size.width as usize,
                        size.height as usize,
                    ))))
                }

                winit::event::WindowEvent::Moved(position) => {
                    return Ok(Event::WindowEvent(WindowEvent::Moved(WindowPosition::new(
                        position.x as usize,
                        position.y as usize,
                    ))))
                }

                winit::event::WindowEvent::CloseRequested => {
                    return Ok(Event::WindowEvent(WindowEvent::CloseRequested))
                }

                winit::event::WindowEvent::Focused(_) => {
                    return Ok(Event::WindowEvent(WindowEvent::FocusChanged))
                }

                winit::event::WindowEvent::CursorEntered { .. } => {
                    return Ok(Event::PointerEvent(PointerEvent::MouseEntered))
                }

                winit::event::WindowEvent::CursorLeft { .. } => {
                    return Ok(Event::PointerEvent(PointerEvent::MouseExited))
                }

                winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                    winit::event::ElementState::Pressed => {
                        return Ok(Event::PointerEvent(PointerEvent::MouseButtonPressed(
                            (*button).into(),
                        )))
                    }
                    winit::event::ElementState::Released => {
                        return Ok(Event::PointerEvent(PointerEvent::MouseButtonReleased(
                            (*button).into(),
                        )))
                    }
                },

                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key_code) = input.virtual_keycode {
                        match input.state {
                            winit::event::ElementState::Pressed => {
                                return Ok(Event::KeyboardEvent(KeyboardEvent::KeyPressed(
                                    key_code.into(),
                                )))
                            }
                            winit::event::ElementState::Released => {
                                return Ok(Event::KeyboardEvent(KeyboardEvent::KeyReleased(
                                    key_code.into(),
                                )))
                            }
                        }
                    }
                }

                _ => (),
            },

            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    return Ok(Event::PointerEvent(PointerEvent::MouseMoved {
                        delta: *delta,
                    }))
                }

                _ => (),
            },

            winit::event::Event::RedrawRequested(_) => {
                return Ok(Event::RenderEvent(RenderEvent::RedrawRequested))
            }

            _ => (),
        }

        return Err("Unsupported event");
    }
}

pub enum RenderEvent {
    RedrawRequested,
}

pub enum PointerEvent {
    MouseMoved { delta: (f64, f64) },
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseEntered,
    MouseExited,
}

pub enum WindowEvent {
    Resized(WindowSize),
    Moved(WindowPosition),
    CloseRequested,
    FocusChanged,
    DroppedFile,
    HoveredFile,
}

pub enum KeyboardEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
}

pub enum ClipboardEvent {
    // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(id) => match id {
                // TODO: Find out what these are
                _ => MouseButton::Other,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyCode {
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Apostrophe,
    Backquote,
    Colon,
    Semicolon,
    Period,
    Comma,
    Equal,
    Minus,
    Plus,
    Asterisk,
    Slash,
    Backslash,
    BracketLeft,
    BracketRight,

    Escape,
    Tab,
    CapsLock,
    Backspace,
    Enter,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    Pause,
    Space,
    PageUp,
    PageDown,
    End,
    Home,
    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,
    PrintScreen,
    Insert,
    Delete,
    ScrollLock,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    OSLeft,
    OSRight,
}

impl From<winit::event::VirtualKeyCode> for KeyCode {
    fn from(value: winit::event::VirtualKeyCode) -> Self {
        match value {
            winit::event::VirtualKeyCode::Key1 => KeyCode::Digit1,
            winit::event::VirtualKeyCode::Key2 => KeyCode::Digit2,
            winit::event::VirtualKeyCode::Key3 => KeyCode::Digit3,
            winit::event::VirtualKeyCode::Key4 => KeyCode::Digit4,
            winit::event::VirtualKeyCode::Key5 => KeyCode::Digit5,
            winit::event::VirtualKeyCode::Key6 => KeyCode::Digit6,
            winit::event::VirtualKeyCode::Key7 => KeyCode::Digit7,
            winit::event::VirtualKeyCode::Key8 => KeyCode::Digit8,
            winit::event::VirtualKeyCode::Key9 => KeyCode::Digit9,
            winit::event::VirtualKeyCode::Key0 => KeyCode::Digit0,

            winit::event::VirtualKeyCode::A => KeyCode::A,
            winit::event::VirtualKeyCode::B => KeyCode::B,
            winit::event::VirtualKeyCode::C => KeyCode::C,
            winit::event::VirtualKeyCode::D => KeyCode::D,
            winit::event::VirtualKeyCode::E => KeyCode::E,
            winit::event::VirtualKeyCode::F => KeyCode::F,
            winit::event::VirtualKeyCode::G => KeyCode::G,
            winit::event::VirtualKeyCode::H => KeyCode::H,
            winit::event::VirtualKeyCode::I => KeyCode::I,
            winit::event::VirtualKeyCode::J => KeyCode::J,
            winit::event::VirtualKeyCode::K => KeyCode::K,
            winit::event::VirtualKeyCode::L => KeyCode::L,
            winit::event::VirtualKeyCode::M => KeyCode::M,
            winit::event::VirtualKeyCode::N => KeyCode::N,
            winit::event::VirtualKeyCode::O => KeyCode::O,
            winit::event::VirtualKeyCode::P => KeyCode::P,
            winit::event::VirtualKeyCode::Q => KeyCode::Q,
            winit::event::VirtualKeyCode::R => KeyCode::R,
            winit::event::VirtualKeyCode::S => KeyCode::S,
            winit::event::VirtualKeyCode::T => KeyCode::T,
            winit::event::VirtualKeyCode::U => KeyCode::U,
            winit::event::VirtualKeyCode::V => KeyCode::V,
            winit::event::VirtualKeyCode::W => KeyCode::W,
            winit::event::VirtualKeyCode::X => KeyCode::X,
            winit::event::VirtualKeyCode::Y => KeyCode::Y,
            winit::event::VirtualKeyCode::Z => KeyCode::Z,

            winit::event::VirtualKeyCode::F1 => KeyCode::F1,
            winit::event::VirtualKeyCode::F2 => KeyCode::F2,
            winit::event::VirtualKeyCode::F3 => KeyCode::F3,
            winit::event::VirtualKeyCode::F4 => KeyCode::F4,
            winit::event::VirtualKeyCode::F5 => KeyCode::F5,
            winit::event::VirtualKeyCode::F6 => KeyCode::F6,
            winit::event::VirtualKeyCode::F7 => KeyCode::F7,
            winit::event::VirtualKeyCode::F8 => KeyCode::F8,
            winit::event::VirtualKeyCode::F9 => KeyCode::F9,
            winit::event::VirtualKeyCode::F10 => KeyCode::F10,
            winit::event::VirtualKeyCode::F11 => KeyCode::F11,
            winit::event::VirtualKeyCode::F12 => KeyCode::F12,

            winit::event::VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            winit::event::VirtualKeyCode::Grave => KeyCode::Backquote,
            winit::event::VirtualKeyCode::Colon => KeyCode::Colon,
            winit::event::VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            winit::event::VirtualKeyCode::Period => KeyCode::Period,
            winit::event::VirtualKeyCode::Comma => KeyCode::Comma,
            winit::event::VirtualKeyCode::Equals => KeyCode::Equal,
            winit::event::VirtualKeyCode::Minus => KeyCode::Minus,
            winit::event::VirtualKeyCode::Plus => KeyCode::Plus,
            winit::event::VirtualKeyCode::Asterisk => KeyCode::Asterisk,
            winit::event::VirtualKeyCode::Slash => KeyCode::Slash,
            winit::event::VirtualKeyCode::Backslash => KeyCode::Backslash,
            winit::event::VirtualKeyCode::LBracket => KeyCode::BracketLeft,
            winit::event::VirtualKeyCode::RBracket => KeyCode::BracketRight,

            winit::event::VirtualKeyCode::Escape => KeyCode::Escape,
            winit::event::VirtualKeyCode::Tab => KeyCode::Tab,
            winit::event::VirtualKeyCode::Capital => KeyCode::CapsLock,
            winit::event::VirtualKeyCode::Back => KeyCode::Backspace,
            winit::event::VirtualKeyCode::Return => KeyCode::Enter,
            winit::event::VirtualKeyCode::LShift => KeyCode::ShiftLeft,
            winit::event::VirtualKeyCode::RShift => KeyCode::ShiftRight,
            winit::event::VirtualKeyCode::LControl => KeyCode::ControlLeft,
            winit::event::VirtualKeyCode::RControl => KeyCode::ControlRight,
            winit::event::VirtualKeyCode::LAlt => KeyCode::AltLeft,
            winit::event::VirtualKeyCode::RAlt => KeyCode::AltRight,
            winit::event::VirtualKeyCode::Pause => KeyCode::Pause,
            winit::event::VirtualKeyCode::Space => KeyCode::Space,
            winit::event::VirtualKeyCode::PageUp => KeyCode::PageUp,
            winit::event::VirtualKeyCode::PageDown => KeyCode::PageDown,
            winit::event::VirtualKeyCode::End => KeyCode::End,
            winit::event::VirtualKeyCode::Home => KeyCode::Home,
            winit::event::VirtualKeyCode::Left => KeyCode::ArrowLeft,
            winit::event::VirtualKeyCode::Up => KeyCode::ArrowUp,
            winit::event::VirtualKeyCode::Right => KeyCode::ArrowRight,
            winit::event::VirtualKeyCode::Down => KeyCode::ArrowDown,
            winit::event::VirtualKeyCode::Snapshot => KeyCode::PrintScreen,
            winit::event::VirtualKeyCode::Insert => KeyCode::Insert,
            winit::event::VirtualKeyCode::Delete => KeyCode::Delete,
            winit::event::VirtualKeyCode::Scroll => KeyCode::ScrollLock,

            winit::event::VirtualKeyCode::Numlock => KeyCode::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            winit::event::VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            winit::event::VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            winit::event::VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            winit::event::VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            winit::event::VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
            winit::event::VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            winit::event::VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,

            winit::event::VirtualKeyCode::LWin => KeyCode::OSLeft,
            winit::event::VirtualKeyCode::RWin => KeyCode::OSRight,

            _ => unreachable!(),
        }
    }
}
