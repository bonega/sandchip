use crate::cpu::CPU;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::fs;

pub struct Input {
    pub keymap: HashMap<Keycode, usize>,
}

impl Input {
    pub fn from_string(string: &str) -> Self {
        let keymap: Vec<String> =
            serde_json::from_str(string).expect("Parsing keymap from json failed");
        let mut keycodemap = HashMap::new();
        for (i, v) in keymap.iter().enumerate() {
            keycodemap.insert(Keycode::from_name(v).unwrap(), i);
        }
        Input { keymap: keycodemap }
    }

    pub fn load() -> Self {
        let data = fs::read_to_string("keymap.json").expect("Failed to open keymap.json");
        Input::from_string(&data)
    }

    pub fn handle_key_down(&self, keycode: &Keycode, cpu: &mut CPU) {
        if *keycode == Keycode::Escape {
            std::process::exit(0);
        } else if self.keymap.contains_key(keycode) {
            cpu.keypad[*self.keymap.get(keycode).unwrap()] = 1;
        }
    }

    pub fn handle_key_up(&self, keycode: &Keycode, cpu: &mut CPU) {
        if self.keymap.contains_key(keycode) {
            cpu.keypad[*self.keymap.get(keycode).unwrap()] = 0;
        }
    }
}
