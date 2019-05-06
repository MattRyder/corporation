use std::collections::HashMap;
use winit::{ElementState, VirtualKeyCode};

pub struct InputState {
    key_state: HashMap<VirtualKeyCode, ElementState>,
}

impl Default for InputState {
    fn default() -> Self {
        InputState { key_state: HashMap::new() }
    }
}

impl InputState {
    pub fn on_key_input(&mut self, key_code: VirtualKeyCode, state: winit::ElementState) {
        self.key_state.insert(key_code, state);
    }

    pub fn is_key_down(&self, key_code: &VirtualKeyCode) -> bool {
        match self.key_state.get(key_code) {
            Some(element_state) => match element_state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            },
            None => false,
        }
    }
}
