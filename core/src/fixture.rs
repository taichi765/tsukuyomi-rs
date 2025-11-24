use crate::{
    fixture_def::FixtureDefId,
    universe::{DmxAddress, UniverseId},
};

declare_id_newtype!(FixtureId);

#[derive(Clone, Copy)]
pub enum MergeMode {
    HTP,
    LTP,
}

//TODO: 占有するチャンネルの計算
#[derive(Clone)]
pub struct Fixture {
    id: FixtureId,
    name: String,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_def_id: FixtureDefId,
    fixture_mode: String,
}
// TODO: modeが一つ以上あることを保証
impl Fixture {
    pub fn new(
        name: &str,
        universe_id: UniverseId,
        address: DmxAddress,
        fixture_def_id: FixtureDefId,
        fixture_mode: String,
    ) -> Self {
        Self {
            id: FixtureId::new(),
            name: String::from(name),
            universe_id,
            address,
            fixture_def_id,
            fixture_mode,
        }
    }
    pub fn id(&self) -> FixtureId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn universe_id(&self) -> UniverseId {
        self.universe_id
    }
    pub fn address(&self) -> DmxAddress {
        self.address
    }
    pub fn fixture_def(&self) -> FixtureDefId {
        self.fixture_def_id
    }
    pub fn fixture_mode(&self) -> &str {
        &self.fixture_mode
    }
}
