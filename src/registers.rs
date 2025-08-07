use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterType {
    Character, // Text within lines
    Line,      // Complete lines
    Block,     // Rectangular block (visual block mode)
}

#[derive(Debug, Clone)]
pub struct RegisterData {
    pub content: String,
    pub register_type: RegisterType,
}

impl RegisterData {
    pub fn new(content: String, register_type: RegisterType) -> Self {
        Self {
            content,
            register_type,
        }
    }
}

pub struct RegisterManager {
    // Named registers (a-z for replace, A-Z for append)
    named_registers: HashMap<char, RegisterData>,

    // Default unnamed register (last yank/delete)
    unnamed_register: RegisterData,

    // Numbered registers (0-9) for delete history
    numbered_registers: [RegisterData; 10],

    // System clipboard register (*)
    #[allow(dead_code)]
    clipboard_register: Option<RegisterData>,
}

impl RegisterManager {
    pub fn new() -> Self {
        Self {
            named_registers: HashMap::new(),
            unnamed_register: RegisterData::new(String::new(), RegisterType::Character),
            numbered_registers: std::array::from_fn(|_| {
                RegisterData::new(String::new(), RegisterType::Character)
            }),
            clipboard_register: None,
        }
    }

    /// Store text in a register
    pub fn store_in_register(
        &mut self,
        register_name: Option<char>,
        content: String,
        register_type: RegisterType,
    ) {
        let data = RegisterData::new(content.clone(), register_type.clone());

        match register_name {
            Some(name) => {
                match name {
                    'a'..='z' => {
                        // Lowercase: replace register content
                        self.named_registers.insert(name, data);
                    }
                    'A'..='Z' => {
                        // Uppercase: append to register content
                        let lowercase = name.to_lowercase().next().unwrap();
                        if let Some(existing) = self.named_registers.get_mut(&lowercase) {
                            existing.content.push_str(&content);
                        } else {
                            self.named_registers.insert(lowercase, data);
                        }
                    }
                    '"' => {
                        // Explicit unnamed register
                        self.unnamed_register = data;
                    }
                    _ => {
                        // Invalid register, use unnamed
                        self.unnamed_register = data;
                    }
                }
            }
            None => {
                // No register specified, use unnamed register
                self.unnamed_register = data;
            }
        }

        // Always update unnamed register with the content (vi behavior)
        if register_name != Some('"') {
            self.unnamed_register = RegisterData::new(content, register_type);
        }
    }

    /// Get content from a register
    pub fn get_register_content(&self, register_name: Option<char>) -> Option<&RegisterData> {
        match register_name {
            Some(name) => {
                match name {
                    'a'..='z' => self.named_registers.get(&name),
                    'A'..='Z' => {
                        let lowercase = name.to_lowercase().next().unwrap();
                        self.named_registers.get(&lowercase)
                    }
                    '"' => Some(&self.unnamed_register),
                    '0'..='9' => {
                        let index = name.to_digit(10).unwrap() as usize;
                        Some(&self.numbered_registers[index])
                    }
                    _ => Some(&self.unnamed_register), // Invalid register, return unnamed
                }
            }
            None => Some(&self.unnamed_register), // Default to unnamed register
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnamed_register() {
        let mut manager = RegisterManager::new();

        manager.store_in_register(None, "hello".to_string(), RegisterType::Character);

        let content = manager.get_register_content(None).unwrap();
        assert_eq!(content.content, "hello");
        assert_eq!(content.register_type, RegisterType::Character);
    }

    #[test]
    fn test_named_registers() {
        let mut manager = RegisterManager::new();

        manager.store_in_register(Some('a'), "test".to_string(), RegisterType::Line);

        let content = manager.get_register_content(Some('a')).unwrap();
        assert_eq!(content.content, "test");
        assert_eq!(content.register_type, RegisterType::Line);
    }

    #[test]
    fn test_append_register() {
        let mut manager = RegisterManager::new();

        manager.store_in_register(Some('a'), "hello".to_string(), RegisterType::Character);
        manager.store_in_register(Some('A'), " world".to_string(), RegisterType::Character);

        let content = manager.get_register_content(Some('a')).unwrap();
        assert_eq!(content.content, "hello world");
    }

    #[test]
    fn test_numbered_registers_access() {
        let manager = RegisterManager::new();

        // Test that numbered registers can be accessed (even if empty)
        let reg0 = manager.get_register_content(Some('0')).unwrap();
        assert_eq!(reg0.content, "");

        let reg1 = manager.get_register_content(Some('1')).unwrap();
        assert_eq!(reg1.content, "");
    }
}
