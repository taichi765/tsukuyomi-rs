use std::usize;

use uuid::Uuid;

use crate::{fixture_def::FixtureDef, universe::DmxAddress};

pub(crate) enum MergeMode {
    HTP,
    LTP,
}

//TODO: 占有するチャンネルの計算
#[derive(Clone)]
pub struct Fixture {
    id: usize,
    name: String,
    address: DmxAddress,
    fixture_def: Uuid,
}

impl Fixture {
    pub fn new(id: usize, name: &str, address: u16) -> Self {
        Self {
            id,
            name: String::from(name),
            address: DmxAddress::new(address).unwrap(),
        }
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn address(&self) -> DmxAddress {
        self.address
    }
    pub fn fixture_def(&self) -> &FixtureDef {
        self.fixture_def
    }
}
