use uuid::Uuid;

use crate::universe::{DmxAddress, UniverseId};

#[derive(Clone, Copy)]
pub enum MergeMode {
    HTP,
    LTP,
}

//TODO: 占有するチャンネルの計算
#[derive(Clone)]
pub struct Fixture {
    id: Uuid,
    name: String,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_def_id: Uuid,
    fixture_mode: String,
}

impl Fixture {
    pub fn new(
        name: &str,
        universe_id: UniverseId,
        address: DmxAddress,
        fixture_def_id: Uuid,
        fixture_mode: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            universe_id,
            address,
            fixture_def_id,
            fixture_mode,
        }
    }
    pub fn id(&self) -> Uuid {
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
    pub fn fixture_def(&self) -> Uuid {
        self.fixture_def_id
    }
    pub fn fixture_mode(&self) -> &str {
        &self.fixture_mode
    }
}
