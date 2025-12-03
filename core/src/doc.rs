use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    sync::{RwLock, Weak},
};

use crate::{
    engine::OutputPluginId,
    fixture::{Fixture, FixtureId, MergeMode},
    fixture_def::{FixtureDef, FixtureDefId},
    functions::{FunctionData, FunctionId},
    universe::{DmxAddress, UniverseId},
};

/// Single source of true
pub struct Doc {
    fixtures: HashMap<FixtureId, Fixture>,
    fixture_definitions: HashMap<FixtureDefId, FixtureDef>,
    functions: HashMap<FunctionId, FunctionData>,
    universe_settings: HashMap<UniverseId, UniverseSetting>,
    observers: Vec<Weak<RwLock<dyn DocObserver>>>,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            fixture_definitions: HashMap::new(),
            functions: HashMap::new(),
            universe_settings: HashMap::new(),
            observers: Vec::new(),
        }
    }

    /* ---------- publics ---------- */
    pub fn get_function_data(&self, function_id: FunctionId) -> Option<&FunctionData> {
        self.functions.get(&function_id)
    }

    pub fn universe_settings(&self) -> &HashMap<UniverseId, UniverseSetting> {
        &self.universe_settings
    }

    // TODO: イベント種類を指定できるようにする
    pub fn subscribe(&mut self, observer: Weak<RwLock<dyn DocObserver>>) {
        self.observers.push(observer);
    }

    /* ---------- internals ---------- */
    pub(crate) fn resolve_address(
        &self,
        fixture_id: FixtureId,
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

    /* ---------- mutable functions ---------- */
    pub(crate) fn add_function(&mut self, function: FunctionData) -> Result<(), String> {
        if self.functions.contains_key(&function.id()) {
            return Err(format!("function id {} already exsists", function.id(),));
        }
        self.functions.insert(function.id(), function);
        Ok(())
    }

    pub(crate) fn remove_function(&mut self, function_id: FunctionId) -> Option<FunctionData> {
        self.functions.remove(&function_id)
    }

    pub(crate) fn add_fixture(&mut self, fixture: Fixture) {
        // TODO: fixture_defがあるか確認
        self.fixtures.insert(fixture.id(), fixture);
    }

    pub(crate) fn remove_fixture(&mut self, fixture_id: FixtureId) -> Option<Fixture> {
        self.fixtures.remove(&fixture_id)
    }

    pub(crate) fn add_fixture_def(&mut self, fixture_def: FixtureDef) {
        self.fixture_definitions
            .insert(fixture_def.id(), fixture_def);
    }

    pub(crate) fn remove_fixture_def(
        &mut self,
        fixture_def_id: FixtureDefId,
    ) -> Option<FixtureDef> {
        self.fixture_definitions.remove(&fixture_def_id)
    }

    pub(crate) fn add_output(&mut self, universe_id: UniverseId, plugin: OutputPluginId) {
        // TODO: universeが存在しない時どうする？
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .expect("something went wrong");
        setting.output_plugins.insert(plugin);
        self.notify(DocEvent::UniverseSettingsChanged);
    }

    pub(crate) fn remove_output(&mut self, universe_id: UniverseId, plugin: OutputPluginId) {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .expect("something went wrong");
        //TODO: Optionを返す
        setting.output_plugins.remove(&plugin);
    }

    /// Notifies event to all observers
    fn notify(&mut self, event: DocEvent) {
        self.observers.retain(|weak_ob| {
            if let Some(ob) = weak_ob.upgrade() {
                ob.write().unwrap().on_doc_event(&event);
                true
            } else {
                false
            }
        });
    }
}

#[derive(Debug)]
pub enum ResolveError {
    FixtureNotFound(FixtureId),
    FixtureDefNotFound {
        fixture_id: FixtureId,
        fixture_def_id: FixtureDefId,
    },
    ModeNotFound {
        fixture_def: FixtureDefId,
        mode: String,
    },
    ChannelNotFound {
        fixturedef: FixtureDefId,
        mode: String,
        channel: String,
    },
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error while resolving address: {:?}", self)
    }
}

impl Error for ResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct UniverseSetting {
    output_plugins: HashSet<OutputPluginId>, //TODO: Engineへの依存->PluginIdはdoc.rsで定義
}

impl UniverseSetting {
    pub fn output_plugins(&self) -> &HashSet<OutputPluginId> {
        &self.output_plugins
    }
}

#[derive(Clone)]
pub enum DocEvent {
    UniverseSettingsChanged,
    FixtureAdded { id: FixtureId },
    FixtureRemoved { id: FixtureId },
}

pub trait DocObserver: Send + Sync {
    fn on_doc_event(&mut self, event: &DocEvent);
}

pub(crate) struct ResolvedAddress {
    pub merge_mode: MergeMode,
    pub address: DmxAddress,
}
