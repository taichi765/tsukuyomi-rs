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
    // TODO: すべての関数でimpl Into<String>を使うようにする
    pub fn new(manufacturer: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: FixtureDefId::new(),
            manufacturer: manufacturer.into(),
            model: model.into(),
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
    pub fn insert_mode(
        &mut self,
        name: impl Into<String>,
        mode: FixtureMode,
    ) -> Option<FixtureMode> {
        self.modes.insert(name.into(), mode)
    }

    /// Same as [std::collections::HashMap::insert()]
    pub fn insert_channel(
        &mut self,
        name: impl Into<String>,
        channel: ChannelDef,
    ) -> Option<ChannelDef> {
        self.channel_templates.insert(name.into(), channel)
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

    pub fn get_channel_by_offset(&self, offset: usize) -> Option<&str> {
        let found = self
            .channel_order
            .iter()
            .find(|(_, opt)| opt.is_some_and(|n| n == offset));
        found.map(|(ch, _)| ch.as_str())
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
    WarmWhite,
    ColdWhite,
    Amber,
    UV,
    Custom, // TODO: open-fixture-library互換にする
}
