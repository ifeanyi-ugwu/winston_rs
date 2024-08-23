use std::collections::HashMap;

pub fn default_levels() -> HashMap<String, u8> {
    let mut levels = HashMap::new();
    levels.insert("error".to_string(), 0);
    levels.insert("warn".to_string(), 1);
    levels.insert("info".to_string(), 2);
    levels.insert("debug".to_string(), 3);
    levels.insert("trace".to_string(), 4);
    levels
}
