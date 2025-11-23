use std::collections::HashMap;

use crate::fixture::MergeMode;

declare_id_newtype!(FixtureDefId);

pub struct FixtureDef {
    id: FixtureDefId,
    pub manufacturer: String,
    pub model: String,
    pub modes: HashMap<String, FixtureMode>,
}

impl FixtureDef {
    pub fn new(manufacturer: String, model: String) -> Self {
        Self {
            id: FixtureDefId::new(),
            manufacturer,
            model,
            modes: HashMap::new(),
        }
    }

    pub fn id(&self) -> FixtureDefId {
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
