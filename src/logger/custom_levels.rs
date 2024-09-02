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
