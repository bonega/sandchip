use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::fs;

pub struct Input {
    pub keymap: HashMap<Keycode, usize>,
}

impl Input {
    pub fn from_string(string: &str) -> Self {
        let keymap: Vec<String> = serde_json::from_str(string).unwrap();
        let mut keycodemap = HashMap::new();
        for (i, v) in keymap.iter().enumerate() {
            keycodemap.insert(Keycode::from_name(v).unwrap(), i);
        }
        Input { keymap: keycodemap }
    }

    pub fn load() -> Self {
        let data = fs::read_to_string("keymap.json").unwrap();
        Input::from_string(&data)
    }
}
