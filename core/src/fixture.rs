use uuid::Uuid;

use crate::universe::DmxAddress;

#[derive(Clone, Copy)]
pub(crate) enum MergeMode {
    HTP,
    LTP,
}

//TODO: 占有するチャンネルの計算
#[derive(Clone)]
pub struct Fixture {
    id: Uuid,
    name: String,
    address: DmxAddress,
    fixture_def_id: Uuid,
    fixture_mode: String,
}

impl Fixture {
    pub fn new(name: &str, address: u16, fixture_def_id: Uuid, fixture_mode: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            address: DmxAddress::new(address).unwrap(),
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
