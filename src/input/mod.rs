// src/input/mod.rs
use winit::event::{KeyboardInput, MouseButton, WindowEvent};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    Space,
}

pub struct InputManager {
    keys_pressed: HashSet<Key>,
    mouse_x: f32,
    mouse_y: f32,
    mouse_button: Option<MouseButton>,
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            keys_pressed: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_button: None,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    let key_enum = match key {
                        winit::keyboard::VirtualKeyCode::W => Some(Key::W),
                        winit::keyboard::VirtualKeyCode::A => Some(Key::A),
                        winit::keyboard::VirtualKeyCode::S => Some(Key::S),
                        winit::keyboard::VirtualKeyCode::D => Some(Key::D),
                        winit::keyboard::VirtualKeyCode::Space => Some(Key::Space),
                        _ => None,
                    };
                    if let Some(key_enum) = key_enum {
                        if input.state == winit::event::ElementState::Pressed {
                            self.keys_pressed.insert(key_enum);
                        } else {
                            self.keys_pressed.remove(&key_enum);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Handle mouse input if needed
            }
            _ => {}
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn set_key(&mut self, key: Key, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y)
    }
}
