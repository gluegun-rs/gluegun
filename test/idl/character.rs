pub struct Character {
    name: String,
    class: Class,
    level: u32,
}

impl Character {
    pub fn new(name: &str, class: Class) -> Self {
        Character {
            name: name.to_string(),
            class,
            level: 1,
        }
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn level_up(&mut self) {
        self.level += 1;
    }
    
    pub fn level(&self) -> u32 {
        self.level
    }
}

#[derive(Copy, Clone)]
pub enum Class {
    Fighter,
    Wizard,
    Rogue,
    Cleric,
}