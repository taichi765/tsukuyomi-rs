use std::collections::HashMap;

use uuid::Uuid;

use crate::fixture::MergeMode;

pub struct FixtureDef {
    id: Uuid,
    pub manufacturer: String,
    pub model: String,
    pub modes: HashMap<String, FixtureMode>,
}

impl FixtureDef {
    pub fn new(manufacturer: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer,
            model,
            modes: HashMap::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn add_mode(&mut self, name: String, mode: FixtureMode) {
        self.modes.insert(name, mode);
    }
}

pub struct FixtureMode {
    pub channel_order: HashMap<String, Option<(usize, ChannelDef)>>,
}

pub struct ChannelDef {
    pub merge_mode: MergeMode,
}
