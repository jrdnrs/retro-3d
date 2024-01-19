use maths::linear::Vec2f;
use window::event::{Event, KeyCode, KeyboardEvent, MouseButton, PointerEvent};

#[derive(Clone, Copy, Default)]
pub struct State {
    pressed: bool,
    held: bool,
    released: bool,
}

pub struct Mouse {
    delta_x: f64,
    delta_y: f64,
    pos_x: f64,
    pos_y: f64,
    on_window: bool,
    moved: bool,
    pub grabbed: bool,
    button_states: Vec<State>,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            delta_x: 0.0,
            delta_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            on_window: false,
            moved: false,
            grabbed: false,
            // `button_states` length of 6 corresponds to `MouseButton` enum length of 6
            button_states: vec![State::default(); 6],
        }
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.button_states[button as usize].pressed
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.button_states[button as usize].held
    }

    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.button_states[button as usize].released
    }

    pub fn is_moved(&self) -> bool {
        self.moved
    }

    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    pub fn is_on_window(&self) -> bool {
        self.on_window
    }

    pub fn position(&self) -> Vec2f {
        Vec2f::new(self.pos_x as f32, self.pos_y as f32)
    }

    pub fn delta(&self) -> Vec2f {
        Vec2f::new(self.delta_x as f32, self.delta_y as f32)
    }

    fn on_pointer_event(&mut self, event: &PointerEvent) {
        match event {
            PointerEvent::MouseMoved { delta: (x, y) } => {
                self.delta_x += x;
                self.delta_y += y;
                self.moved = true;
            }

            PointerEvent::MouseButtonPressed(button) => {
                let state = &mut self.button_states[*button as usize];
                state.pressed = true;
                state.held = true;
            }

            PointerEvent::MouseButtonReleased(button) => {
                self.button_states[*button as usize].held = false;
                self.button_states[*button as usize].released = true;
            }

            PointerEvent::MouseEntered => {
                self.on_window = true;
            }

            PointerEvent::MouseExited => {
                self.on_window = false;
            }
        }
    }

    fn update(&mut self) {
        for state in self.button_states.iter_mut() {
            state.pressed = false;
            state.released = false;
        }

        self.moved = false;
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }
}

pub struct Keyboard {
    key_states: Vec<State>,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            // `key_states` length of 106 corresponds to `KeyCode` enum length of 106
            key_states: vec![State::default(); 108],
        }
    }

    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.key_states[keycode as usize].pressed
    }

    pub fn is_key_held(&self, keycode: KeyCode) -> bool {
        self.key_states[keycode as usize].held
    }

    pub fn is_key_released(&self, keycode: KeyCode) -> bool {
        self.key_states[keycode as usize].released
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        match event {
            KeyboardEvent::KeyPressed(keycode) => {
                let state = &mut self.key_states[*keycode as usize];
                state.pressed = !state.held;
                state.held = true;
            }

            KeyboardEvent::KeyReleased(keycode) => {
                self.key_states[*keycode as usize].held = false;
                self.key_states[*keycode as usize].released = true;
            }
        }
    }

    fn update(&mut self) {
        for state in self.key_states.iter_mut() {
            state.pressed = false;
            state.released = false;
        }
    }
}

pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::PointerEvent(event) => self.mouse.on_pointer_event(event),
            Event::KeyboardEvent(event) => self.keyboard.handle_keyboard_event(event),
            _ => (),
        }
    }

    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input() {
        let mut input = Input::new();
        let event = Event::KeyboardEvent(KeyboardEvent::KeyPressed(KeyCode::A));
        input.update();
        input.handle_event(&event);
        assert!(input.keyboard.is_key_pressed(KeyCode::A));
    }

    #[test]
    fn test_mouse() {
        let mut mouse = Mouse::new();
        let event = PointerEvent::MouseMoved { delta: (1.0, 1.0) };
        mouse.update();
        mouse.on_pointer_event(&event);
        assert!(mouse.moved);
    }
}
