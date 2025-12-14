mod errors;
pub use errors::*;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{Arc, RwLock, Weak},
};

use tracing::warn;

use crate::{
    fixture::{Fixture, FixtureId, MergeMode},
    fixture_def::{FixtureDef, FixtureDefId},
    functions::{FunctionData, FunctionId},
    readonly::ReadOnly,
    universe::{DmxAddress, UniverseId},
};

declare_id_newtype!(OutputPluginId);

/// Handle to [DocStore].
/// Manages write lock and event.
pub struct DocHandle {
    /// DocStore is shared across the threads([`Engine`][crate::engine::Engine] has [`ReadOnly<DocStore>`]),
    /// so we use [`Arc`].
    inner: Arc<RwLock<DocStore>>,
    /// [`DocEventBus`] is only used within the main thread, so [`Rc`] is enough.
    /// [`Engine`][crate::engine::Engine] watches [`DocStore`] via [`EngineCommand`] and [`DocEventBridge`].
    event_bus: Rc<RefCell<DocEventBus>>,
}

// TODO: 通知するときとしない時の条件を一貫させる
impl DocHandle {
    pub fn new(doc: Arc<RwLock<DocStore>>, event_bus: Rc<RefCell<DocEventBus>>) -> Self {
        Self {
            inner: doc,
            event_bus,
        }
    }

    pub fn as_readonly(&self) -> ReadOnly<DocStore> {
        ReadOnly::new(Arc::clone(&self.inner))
    }

    pub fn add_function(&self, function: FunctionData) -> Option<FunctionData> {
        let id = function.id();
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.add_function(function)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::FunctionInserted(id));
        opt
    }

    pub fn remove_function(&self, id: &FunctionId) -> Option<FunctionData> {
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.remove_function(id)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::FunctionRemoved(*id));
        opt
    }

    pub fn insert_fixture_def(&self, fixture_def: FixtureDef) -> Option<FixtureDef> {
        let id = fixture_def.id();
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.insert_fixture_def(fixture_def)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::FixtureDefInserted(id));
        opt
    }

    pub fn remove_fixture_def(&self, id: &FixtureDefId) -> Option<FixtureDef> {
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.remove_fixture_def(id)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::FixtureDefRemoved(*id));
        opt
    }

    pub fn insert_fixture(&self, fixture: Fixture) -> Result<Option<Fixture>, FixtureInsertError> {
        let id = fixture.id();
        let result = {
            let mut guard = self.inner.write().unwrap();
            guard.insert_fixture(fixture)
        };
        if let Ok(_) = result {
            self.event_bus
                .borrow_mut()
                .notify(DocEvent::FixtureInserted(id));
        }
        result
    }

    pub fn remove_fixture(&self, id: &FixtureId) -> Result<Option<Fixture>, FixtureRemoveError> {
        let result = {
            let mut guard = self.inner.write().unwrap();
            guard.remove_fixture(id)
        };
        if let Ok(_) = result {
            self.event_bus
                .borrow_mut()
                .notify(DocEvent::FixtureRemoved(*id));
        }
        result
    }

    pub fn add_universe(&self, id: UniverseId) -> Option<UniverseSetting> {
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.add_universe(id)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::UniverseAdded(id));
        opt
    }

    pub fn remove_universe(&self, id: &UniverseId) -> Option<UniverseSetting> {
        let opt = {
            let mut guard = self.inner.write().unwrap();
            guard.remove_universe(id)
        };
        self.event_bus
            .borrow_mut()
            .notify(DocEvent::UniverseRemoved(*id));
        opt
    }

    pub fn add_output(
        &self,
        universe_id: UniverseId,
        plugin: OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let result = {
            let mut guard = self.inner.write().unwrap();
            guard.add_output(universe_id, plugin)
        };
        if let Ok(_) = result {
            self.event_bus
                .borrow_mut()
                .notify(DocEvent::UniverseSettingsChanged);
        }
        result
    }

    pub fn remove_output(
        &self,
        universe_id: &UniverseId,
        plugin: &OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let result = {
            let mut guard = self.inner.write().unwrap();
            guard.remove_output(universe_id, plugin)
        };
        if let Ok(_) = result {
            self.event_bus
                .borrow_mut()
                .notify(DocEvent::UniverseSettingsChanged);
        }
        result
    }
}

pub struct DocEventBus {
    observers: Vec<Weak<RwLock<dyn DocObserver>>>,
}

impl DocEventBus {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    // TODO: イベント種類を指定できるようにする
    pub fn subscribe(&mut self, observer: Weak<RwLock<dyn DocObserver>>) {
        self.observers.push(observer);
    }

    fn notify(&self, event: DocEvent) {
        // FIXME: 死んだObserverの削除をどうするか？retainとかは&mut selfが必要
        self.observers.iter().for_each(|weak_ob| {
            if let Some(ob) = weak_ob.upgrade() {
                ob.write().unwrap().on_doc_event(&event);
            } else {
                warn!("failed to upgrade weak reference");
            }
        });
    }
}

#[derive(Debug, Clone)]
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

/// Single source of true
pub struct DocStore {
    fixtures: HashMap<FixtureId, Fixture>,
    fixture_defs: HashMap<FixtureDefId, FixtureDef>,
    functions: HashMap<FunctionId, FunctionData>,
    universe_settings: HashMap<UniverseId, UniverseSetting>,

    fixture_by_address_index: HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
}

/* ---------- public, readonly ---------- */
impl DocStore {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            fixture_defs: HashMap::new(),
            functions: HashMap::new(),
            universe_settings: HashMap::new(),

            fixture_by_address_index: HashMap::new(),
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
        let channel_offset = mode
            .channel_order()
            .get(channel)
            .unwrap() // FIXME: unwrap
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
                address: fixture.address().checked_add(channel_offset).unwrap(), //FIXME: unwrap
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

    /// Returns max address which is occupied by a fixture.
    ///
    /// If there's no fixture in the universe, `None` is returned.
    /// If universe does not exist in the DocStore, `None` is returned.
    pub fn current_max_address(&self, universe: UniverseId) -> Option<DmxAddress> {
        let max_fixture = self
            .fixtures
            .iter()
            .filter(|(_, fxt)| fxt.universe_id() == universe)
            .map(|(_, fxt)| fxt)
            .max_by(|a, b| a.address().cmp(&b.address()));
        if let Some(max_fixture) = max_fixture {
            let fixture_def = self.get_fixture_def(&max_fixture.fixture_def()).unwrap();
            let adr = max_fixture
                .occupied_addresses(fixture_def)
                .expect("invariant is broken")
                .iter()
                .last()
                .unwrap() // This unwrap() is safe because occupied addresses can't be empty
                .to_owned();
            Some(adr)
        } else {
            None
        }
    }
}

/* ---------- private, mutable ---------- */
impl DocStore {
    /// Same as [std::collections::HashMap::remove()]
    fn add_function(&mut self, function: FunctionData) -> Option<FunctionData> {
        let id = function.id();
        self.functions.insert(id, function)
    }

    /// Same as [std::collections::HashMap::remove()]
    fn remove_function(&mut self, id: &FunctionId) -> Option<FunctionData> {
        self.functions.remove(id)
    }

    // TODO: FixtureDefが変更されたときに不変条件が崩れないようにする
    /// TODO: update this comment Same as [std::collections::HashMap::remove()]
    fn insert_fixture(&mut self, fixture: Fixture) -> Result<Option<Fixture>, FixtureInsertError> {
        // FIXME: signature is complicated. Using enum(Outcome::Created/Updated) would be good.
        let def_id = fixture.fixture_def();
        let fixture_def =
            self.get_fixture_def(&def_id)
                .ok_or(FixtureInsertError::FixtureDefNotFound(FixtureDefNotFound {
                    fixture_id: fixture.id(),
                    fixture_def_id: def_id,
                }))?;
        let occupied_addresses = fixture
            .occupied_addresses(fixture_def)
            .map_err(|e| FixtureInsertError::ModeNotFound(e))?;

        self.validate_fixture_address(&fixture, &occupied_addresses)
            .map_err(|e| FixtureInsertError::AddressValidateError(e))?;

        for adr in occupied_addresses {
            if let Some(_) = self.fixture_by_address_index.insert(
                (fixture.universe_id(), adr),
                (fixture.id(), adr.checked_sub(fixture.address()).unwrap()),
            ) {
                warn!("there must be logic error in address validation");
            }
        }

        let id = fixture.id();
        let opt = self.fixtures.insert(id, fixture.clone());

        Ok(opt)
    }

    /// Same as [std::collections::HashMap::remove()]
    fn remove_fixture(&mut self, id: &FixtureId) -> Result<Option<Fixture>, FixtureRemoveError> {
        if !self.fixtures.contains_key(id) {
            return Ok(None);
        }
        let fixture = self.fixtures.get(id).unwrap();
        let def_id = fixture.fixture_def();
        let fixture_def =
            self.fixture_defs
                .get(&def_id)
                .ok_or(FixtureRemoveError::FixtureDefNotFound(FixtureDefNotFound {
                    fixture_id: *id,
                    fixture_def_id: def_id,
                }))?;
        let occupied_addresses = fixture
            .occupied_addresses(fixture_def)
            .map_err(|e| FixtureRemoveError::ModeNotFound(e))?; // FIXME: .expect()でもいいかも?

        for adr in occupied_addresses {
            if let Some((old_id, offset)) = self
                .fixture_by_address_index
                .remove(&(fixture.universe_id(), adr))
            {
                // FIXME: unwrap
                if old_id != *id || offset != adr.checked_sub(fixture.address()).unwrap() {
                    warn!(address=?adr,fixture_id=?id,?old_id,?offset,"address index had unexpected value");
                }
            } else {
                warn!("the states of address index was invalid");
            }
        }

        let old = self.fixtures.remove(id).unwrap();
        Ok(Some(old))
    }

    /// Same as [std::collections::HashMap::remove()]
    fn insert_fixture_def(&mut self, fixture_def: FixtureDef) -> Option<FixtureDef> {
        let id = fixture_def.id();
        self.fixture_defs.insert(id, fixture_def)
    }

    /// Same as [std::collections::HashMap::remove()]
    fn remove_fixture_def(&mut self, id: &FixtureDefId) -> Option<FixtureDef> {
        // TODO: このFixtureDefを参照しているFixtureの処理
        self.fixture_defs.remove(id)
    }

    /// Returns `Some(old_setting)` or `None`
    fn add_universe(&mut self, id: UniverseId) -> Option<UniverseSetting> {
        self.universe_settings.insert(id, UniverseSetting::new())
    }

    /// Same as [std::collections::HashMap::remove()]
    fn remove_universe(&mut self, id: &UniverseId) -> Option<UniverseSetting> {
        self.universe_settings.remove(id)
    }

    /// Returns `true` when plugin already exists.
    fn add_output(
        &mut self,
        universe_id: UniverseId,
        plugin: OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(universe_id))?;
        let is_inserted = setting.output_plugins.insert(plugin);

        Ok(is_inserted)
    }

    /// Returns `true` when plugin was not in the list.
    fn remove_output(
        &mut self,
        universe_id: &UniverseId,
        plugin: &OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(*universe_id))?;
        let is_removed = setting.output_plugins.remove(&plugin);

        Ok(is_removed)
    }
}

/* ---------- helpers ---------- */
impl DocStore {
    /// Validates that the fixture does not conflict with existing [Fixture]s' address.
    fn validate_fixture_address(
        &self,
        fixture: &Fixture,
        occupied_addresses: &[DmxAddress],
    ) -> Result<(), ValidateError> {
        let mut conflicts = Vec::new();

        for adr in occupied_addresses {
            if let Some((old_fixture_id, old_offset)) = self
                .fixture_by_address_index
                .get(&(fixture.universe_id(), *adr))
            {
                if *old_fixture_id == fixture.id() {
                    continue;
                }
                conflicts.push(AddressConflictedError {
                    address: *adr,
                    old_fixture_id: *old_fixture_id,
                    old_offset: *old_offset,
                    new_fixture_id: fixture.id(),
                    new_offset: adr.checked_sub(fixture.address()).unwrap(), //TODO: use Err()
                });
            }
        }

        if conflicts.is_empty() {
            return Ok(());
        } else {
            return Err(ValidateError::AddressConflicted(conflicts));
        }
    }
}

#[cfg(test)]
mod tests {
    mod address_index;
    mod current_max_address;
    mod events;
    mod fixture_defs;
    mod fixtures;
    mod functions;
    mod helpers;
    mod resolve;
    mod universe_outputs;
}
