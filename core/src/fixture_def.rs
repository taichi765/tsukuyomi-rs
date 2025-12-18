use std::collections::HashMap;

use bimap::BiHashMap;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum FixtureModeCreateError {
    #[error("{} offset are duplicated",.duplicates.len())]
    Duplicated { duplicates: Vec<DuplicatedError> },
    #[error("Mode must include at least 1 channel")]
    Empty,
    #[error("channel order was not contiguous")]
    NotContiguous,
}

#[derive(Debug, Error)]
pub enum DuplicatedError {
    #[error("offset {offset} is used by these channels: {channels:?}")]
    OffsetDuplicated {
        offset: usize,
        channels: Vec<String>,
    },
    #[error("channel {channel} is used by these offsets: {offsets:?}")]
    ChannelDuplicated {
        channel: String,
        offsets: Vec<usize>,
    },
}

pub struct FixtureMode {
    channel_order: BiHashMap<String, usize>,
}

impl FixtureMode {
    /// Creates new `FixtureMode`.
    pub fn new(
        channel_order: impl Iterator<Item = (String, usize)>,
    ) -> Result<Self, FixtureModeCreateError> {
        let mut map = BiHashMap::new();
        let mut ch_duplicates: HashMap<String, Vec<usize>> = HashMap::new();
        let mut off_duplicates: HashMap<usize, Vec<String>> = HashMap::new();
        for (ch, off) in channel_order {
            if let Some(first_off) = map.get_by_left(&ch) {
                ch_duplicates
                    .entry(ch)
                    .and_modify(|v| v.push(off))
                    .or_insert(vec![*first_off, off]);
                continue;
            }

            if let Some(first_ch) = map.get_by_right(&off).cloned() {
                off_duplicates
                    .entry(off)
                    .and_modify(|v| v.push(ch.clone())) // TODO: clone
                    .or_insert(vec![first_ch, ch]);
                continue;
            }

            map.insert_no_overwrite(ch, off).expect("logic error");
        }

        let ch_errors = ch_duplicates
            .into_iter()
            .map(|(channel, offsets)| DuplicatedError::ChannelDuplicated { channel, offsets });
        let offset_errors = off_duplicates
            .into_iter()
            .map(|(offset, channels)| DuplicatedError::OffsetDuplicated { offset, channels });
        let errors: Vec<DuplicatedError> = ch_errors.chain(offset_errors).collect();

        if !errors.is_empty() {
            return Err(FixtureModeCreateError::Duplicated { duplicates: errors });
        }

        let Some(max) = map.right_values().copied().max() else {
            return Err(FixtureModeCreateError::Empty);
        };
        if max != map.len() - 1 {
            return Err(FixtureModeCreateError::NotContiguous);
        }

        Ok(Self { channel_order: map })
    }

    /// Total length of the channels in this mode.
    pub fn footprint(&self) -> usize {
        self.channel_order.len()
    }

    pub fn get_offset_by_channel(&self, channel: &str) -> Option<usize> {
        self.channel_order.get_by_left(channel).map(|n| *n)
    }

    pub fn get_channel_by_offset(&self, offset: usize) -> Option<&str> {
        self.channel_order.get_by_right(&offset).map(|s| s.as_str())
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
