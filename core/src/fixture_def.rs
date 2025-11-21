use std::collections::HashMap;

use uuid::Uuid;

pub(crate) struct FixtureDef {
    id: Uuid,
    pub manufacturer: String,
    pub model: String,
    pub channels: HashMap<String, ChannelDef>,
    pub modes: HashMap<String, FixtureMode>,
}

pub(crate) struct FixtureMode {
    pub channel_order: HashMap<usize, ChannelDef>,
}

pub(crate) struct ChannelDef {}
