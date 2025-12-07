mod errors;
pub use errors::*;

use std::{
    collections::{HashMap, HashSet},
    sync::{RwLock, Weak},
};

use tracing::{trace, warn};

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
    fixture_defs: HashMap<FixtureDefId, FixtureDef>,
    functions: HashMap<FunctionId, FunctionData>,
    universe_settings: HashMap<UniverseId, UniverseSetting>,
    observers: Vec<Weak<RwLock<dyn DocObserver>>>,
}

/* ---------- public, readonly ---------- */
impl Doc {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            fixture_defs: HashMap::new(),
            functions: HashMap::new(),
            universe_settings: HashMap::new(),
            observers: Vec::new(),
        }
    }

    /* ---------- publics ---------- */
    /// Same as [std::collections::HashMap::get()]
    pub fn get_function_data(&self, id: &FunctionId) -> Option<&FunctionData> {
        self.functions.get(id)
    }

    /// Same as [std::collections::HashMap::get()]
    pub fn get_fixture(&self, id: &FixtureId) -> Option<&Fixture> {
        self.fixtures.get(id)
    }

    /// Same as [std::collections::HashMap::get()]
    pub fn get_fixture_def(&self, id: &FixtureDefId) -> Option<&FixtureDef> {
        self.fixture_defs.get(id)
    }

    pub fn universe_settings(&self) -> &HashMap<UniverseId, UniverseSetting> {
        &self.universe_settings
    }

    // TODO: イベント種類を指定できるようにする
    pub fn subscribe(&mut self, observer: Weak<RwLock<dyn DocObserver>>) {
        self.observers.push(observer);
    }

    pub fn resolve_address(
        &self,
        fixture_id: FixtureId,
        channel: &str,
    ) -> Result<(UniverseId, ResolvedAddress), ResolveError> {
        let fixture = self
            .fixtures
            .get(&fixture_id)
            .ok_or(ResolveError::FixtureNotFound(FixtureNotFound(fixture_id)))?;

        let fixture_def = self.fixture_defs.get(&fixture.fixture_def()).ok_or(
            ResolveError::FixtureDefNotFound(FixtureDefNotFound {
                fixture_id: fixture.id(),
                fixture_def_id: fixture.fixture_def(),
            }),
        )?;
        let mode =
            fixture_def
                .modes()
                .get(fixture.fixture_mode())
                .ok_or(ResolveError::ModeNotFound(ModeNotFound {
                    fixture_def: fixture.fixture_def(),
                    mode: fixture.fixture_mode().into(),
                }))?;
        let channel_offset =
            mode.channel_order()
                .get(channel)
                .unwrap()
                .ok_or(ResolveError::ChannelNotFound {
                    fixture_def: fixture.fixture_def(),
                    mode: fixture.fixture_mode().into(),
                    channel: channel.into(),
                })?;

        let merge_mode = fixture_def
            .channel_templates()
            .get(channel)
            .unwrap() // TODO: should return Err
            .merge_mode();
        Ok((
            fixture.universe_id(),
            ResolvedAddress {
                merge_mode,
                address: DmxAddress::new(fixture.address().value() + channel_offset).unwrap(),
            },
        ))
    }

    pub fn get_fixture_by_address(
        &self,
        universe_id: &UniverseId,
        address: DmxAddress,
    ) -> Option<&(FixtureId, usize)> {
        self.fixture_by_address_index.get(&(*universe_id, address))
    }
}

#[derive(Clone)]
pub enum DocEvent {
    UniverseSettingsChanged,
    UniverseAdded(UniverseId),
    UniverseRemoved(UniverseId),
    /// Also emitted when [`Fixture`] is updated
    FixtureInserted(FixtureId),
    FixtureRemoved(FixtureId),
    /// Also emitted when [`FixtureDef`] is updated
    FixtureDefInserted(FixtureDefId),
    FixtureDefRemoved(FixtureDefId),
    /// Also emitted when [`FunctionData`] is updated
    FunctionInserted(FunctionId),
    FunctionRemoved(FunctionId),
}

pub trait DocObserver: Send + Sync {
    fn on_doc_event(&mut self, event: &DocEvent);
}

pub struct UniverseSetting {
    output_plugins: HashSet<OutputPluginId>, //TODO: Engineへの依存->PluginIdはdoc.rsで定義
}

impl UniverseSetting {
    pub fn new() -> Self {
        Self {
            output_plugins: HashSet::new(),
        }
    }

    pub fn output_plugins(&self) -> &HashSet<OutputPluginId> {
        &self.output_plugins
    }
}

#[derive(Debug)]
pub struct ResolvedAddress {
    pub merge_mode: MergeMode,
    pub address: DmxAddress,
}

/* ---------- pub(crate), mutables ---------- */
impl Doc {
    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn add_function(&mut self, function: FunctionData) -> Option<FunctionData> {
        let id = function.id();
        let opt = self.functions.insert(id, function);
        self.notify(DocEvent::FunctionInserted(id));
        opt
    }

    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn remove_function(&mut self, id: &FunctionId) -> Option<FunctionData> {
        let opt = self.functions.remove(id);
        self.notify(DocEvent::FunctionRemoved(*id));
        opt
    }

    /// TODO: update this comment Same as [std::collections::HashMap::remove()]
    pub(crate) fn insert_fixture(
        &mut self,
        fixture: Fixture,
    ) -> Result<Option<Fixture>, FixtureInsertError> {
        // FIXME: signature is complicated
        self.validate_fixture(&fixture)
            .map_err(|e| FixtureInsertError::AddressValidateError(e))?;
        let id = fixture.id();
        let opt = self.fixtures.insert(id, fixture);
        self.notify(DocEvent::FixtureInserted(id));
        Ok(opt)
    }

    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn remove_fixture(&mut self, id: &FixtureId) -> Option<Fixture> {
        let opt = self.fixtures.remove(id);
        self.notify(DocEvent::FixtureRemoved(*id));
        opt
    }

    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn insert_fixture_def(&mut self, fixture_def: FixtureDef) -> Option<FixtureDef> {
        let id = fixture_def.id();
        let opt = self.fixture_defs.insert(id, fixture_def);
        self.notify(DocEvent::FixtureDefInserted(id));
        opt
    }

    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn remove_fixture_def(&mut self, id: &FixtureDefId) -> Option<FixtureDef> {
        // TODO: このFixtureDefを参照しているFixtureの処理
        let opt = self.fixture_defs.remove(id);
        self.notify(DocEvent::FixtureDefRemoved(*id));
        opt
    }

    /// Returns `Some(old_setting)` or `None`
    pub(crate) fn add_universe(&mut self, id: UniverseId) -> Option<UniverseSetting> {
        let opt = self.universe_settings.insert(id, UniverseSetting::new());
        self.notify(DocEvent::UniverseAdded(id));
        opt
    }

    /// Same as [std::collections::HashMap::remove()]
    pub(crate) fn remove_universe(&mut self, id: &UniverseId) -> Option<UniverseSetting> {
        let opt = self.universe_settings.remove(id);
        self.notify(DocEvent::UniverseRemoved(*id));
        opt
    }

    /// Returns `true` when plugin already exists.
    pub(crate) fn add_output(
        &mut self,
        universe_id: UniverseId,
        plugin: OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(universe_id))?;
        let is_inserted = setting.output_plugins.insert(plugin);
        if is_inserted {
            trace!("notifying setting change");
            self.notify(DocEvent::UniverseSettingsChanged);
        }
        Ok(is_inserted)
    }

    /// Returns `true` when plugin was not in the list.
    pub(crate) fn remove_output(
        &mut self,
        universe_id: &UniverseId,
        plugin: &OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(*universe_id))?;
        let is_removed = setting.output_plugins.remove(&plugin);
        if is_removed {
            self.notify(DocEvent::UniverseRemoved(*universe_id));
        }
        Ok(is_removed)
    }
}

/* ---------- privates ---------- */
impl Doc {

    /// Notifies event to all observers
    fn notify(&mut self, event: DocEvent) {
        trace!("observers: {}", self.observers.len());
        self.observers.retain(|weak_ob| {
            if let Some(ob) = weak_ob.upgrade() {
                ob.write().unwrap().on_doc_event(&event);
                true
            } else {
                warn!("failed to upgrade weak reference");
                false
            }
        });
    }
}

#[cfg(test)]
mod tests {
    mod events;
    mod fixture_defs;
    mod fixtures;
    mod functions;
    mod helpers;
    mod resolve;
    mod universe_outputs;
}
