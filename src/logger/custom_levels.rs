use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CustomLevels {
    levels: HashMap<String, u8>,
}

impl CustomLevels {
    pub fn new(levels: HashMap<String, u8>) -> Self {
        CustomLevels { levels }
    }

    pub fn get_severity(&self, key: &str) -> Option<u8> {
        self.levels.get(key).copied()
    }
}

impl Default for CustomLevels {
    fn default() -> Self {
        let mut default_levels = HashMap::new();
        default_levels.insert("error".to_string(), 0);
        default_levels.insert("warn".to_string(), 1);
        default_levels.insert("info".to_string(), 2);
        default_levels.insert("debug".to_string(), 3);
        default_levels.insert("trace".to_string(), 4);

        CustomLevels::new(default_levels)
    }
}
