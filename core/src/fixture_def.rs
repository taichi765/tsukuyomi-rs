use std::collections::HashMap;

use crate::fixture::MergeMode;

declare_id_newtype!(FixtureDefId);

pub struct FixtureDef {
    id: FixtureDefId,
    manufacturer: String,
    model: String,
    channel_templates: HashMap<String, ChannelDef>,
    modes: HashMap<String, FixtureMode>,
}

impl FixtureDef {
    pub fn new(manufacturer: String, model: String) -> Self {
        Self {
            id: FixtureDefId::new(),
            manufacturer,
            model,
            modes: HashMap::new(),
            channel_templates: HashMap::new(),
        }
    }

    pub fn id(&self) -> FixtureDefId {
        self.id
    }

    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn modes(&self) -> &HashMap<String, FixtureMode> {
        &self.modes
    }

    pub fn channel_templates(&self) -> &HashMap<String, ChannelDef> {
        &self.channel_templates
    }

    /// Same as [std::collections::HashMap::insert()]
    pub fn insert_mode(&mut self, name: String, mode: FixtureMode) -> Option<FixtureMode> {
        self.modes.insert(name, mode)
    }

    /// Same as [std::collections::HashMap::insert()]
    pub fn insert_channel(&mut self, name: String, channel: ChannelDef) -> Option<ChannelDef> {
        self.channel_templates.insert(name, channel)
    }
}

pub struct FixtureMode {
    channel_order: HashMap<String, Option<usize>>,
}

impl FixtureMode {
    // TODO: 引数取らない方がいいかも
    // TODO: Validate that channel order is contiguous
    pub fn new(channel_order: HashMap<String, Option<usize>>) -> Self {
        Self { channel_order }
    }

    pub fn offset(&self) -> usize {
        self.channel_order
            .iter()
            .filter(|(_, offset)| offset.is_some())
            .count()
    }

    pub fn channel_order(&self) -> &HashMap<String, Option<usize>> {
        &self.channel_order
    }
}

pub struct ChannelDef {
    merge_mode: MergeMode,
    kind: ChannelKind,
}

impl ChannelDef {
    pub fn new(merge_mode: MergeMode, kind: ChannelKind) -> Self {
        Self { merge_mode, kind }
    }

    pub fn merge_mode(&self) -> MergeMode {
        self.merge_mode
    }

    pub fn kind(&self) -> &ChannelKind {
        &self.kind
    }
}

// TODO: Add more kinds
pub enum ChannelKind {
    Dimmer,
    Red,
    Blue,
    Green,
    White,
}
