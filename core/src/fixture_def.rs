use std::collections::HashMap;

use uuid::Uuid;

use crate::fixture::MergeMode;

pub(crate) struct FixtureDef {
    id: Uuid,
    pub manufacturer: String,
    pub model: String,
    pub modes: HashMap<String, FixtureMode>,
}

pub(crate) struct FixtureMode {
    pub channel_order: HashMap<String, Option<(usize, ChannelDef)>>,
}

pub(crate) struct ChannelDef {
    pub merge_mode: MergeMode,
}
