use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    fixture::{Fixture, MergeMode},
    fixture_def::FixtureDef,
    functions::FunctionData,
    universe::{DmxAddress, UniverseId},
};

// TODO: type aliasの活用
#[derive(Debug)]
pub enum ResolveError {
    FixtureNotFound(Uuid),
    FixtureDefNotFound {
        fixture_id: Uuid,
        fixture_def_id: Uuid,
    },
    ModeNotFound {
        fixture_def: Uuid,
        mode: String,
    },
    ChannelNotFound {
        fixturedef: Uuid,
        mode: String,
        channel: String,
    },
}

pub trait DocCommand {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String>;

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String>;
}

pub struct Doc {
    fixtures: HashMap<Uuid, Fixture>,
    fixture_definitions: HashMap<Uuid, FixtureDef>,
    functions: HashMap<Uuid, FunctionData>,
    universe_settings: HashMap<usize, UniverseSetting>,
}

pub(crate) struct UniverseSetting {}

pub(crate) struct ResolvedAddress {
    pub merge_mode: MergeMode,
    pub address: DmxAddress,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            fixture_definitions: HashMap::new(),
            functions: HashMap::new(),
            universe_settings: HashMap::new(),
        }
    }

    pub fn get_function_data(&self, function_id: Uuid) -> Option<&FunctionData> {
        self.functions.get(&function_id)
    }

    pub(crate) fn universe_settings(&self) -> &HashMap<usize, UniverseSetting> {
        &self.universe_settings
    }

    pub(crate) fn add_function(&mut self, function: FunctionData) -> Result<(), String> {
        if self.functions.contains_key(&function.id()) {
            return Err(format!("function id {} already exsists", function.id(),));
        }
        self.functions.insert(function.id(), function);
        //TODO: self.update_function_infos();
        Ok(())
    }

    pub(crate) fn remove_function(&mut self, function_id: Uuid) -> Option<FunctionData> {
        self.functions.remove(&function_id)
    }

    pub(crate) fn add_fixture(&mut self, fixture: Fixture) -> Uuid {
        let id = Uuid::new_v4();
        // TODO: fixture_defがあるか確認
        self.fixtures.insert(id, fixture);
        id
    }

    pub(crate) fn remove_fixture(&mut self, fixture_id: Uuid) -> Option<Fixture> {
        self.fixtures.remove(&fixture_id)
    }

    pub(crate) fn add_fixture_def(&mut self, fixture_def: FixtureDef) -> Uuid {
        let id = Uuid::new_v4();
        self.fixture_definitions.insert(id, fixture_def);
        id
    }

    pub(crate) fn remove_fixture_def(&mut self, fixture_def_id: Uuid) -> Option<FixtureDef> {
        self.fixture_definitions.remove(&fixture_def_id)
    }

    pub(crate) fn resolve_address(
        &self,
        fixture_id: Uuid,
        channel: &str,
    ) -> Result<(UniverseId, ResolvedAddress), ResolveError> {
        let fixture = self
            .fixtures
            .get(&fixture_id)
            .ok_or(ResolveError::FixtureNotFound(fixture_id))?;

        let fixture_def = self.fixture_definitions.get(&fixture.fixture_def()).ok_or(
            ResolveError::FixtureDefNotFound {
                fixture_id: fixture.id(),
                fixture_def_id: fixture.fixture_def(),
            },
        )?;
        let mode =
            fixture_def
                .modes
                .get(fixture.fixture_mode())
                .ok_or(ResolveError::ModeNotFound {
                    fixture_def: fixture.fixture_def(),
                    mode: fixture.fixture_mode().into(),
                })?;
        let channel = mode.channel_order.get(channel).unwrap().as_ref().ok_or(
            ResolveError::ChannelNotFound {
                fixturedef: fixture.fixture_def(),
                mode: fixture.fixture_mode().into(),
                channel: channel.into(),
            },
        )?;

        let merge_mode = channel.1.merge_mode;
        Ok((
            fixture.universe_id(),
            ResolvedAddress {
                merge_mode,
                address: DmxAddress::new(fixture.address().value() + channel.0).unwrap(),
            },
        ))
    }
}
