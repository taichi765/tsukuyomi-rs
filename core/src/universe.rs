use crate::{doc::ResolvedAddress, fixture::MergeMode};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct UniverseId(u8);

impl Into<u8> for UniverseId {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmxAddress {
    universe_id: UniverseId,
    address: usize,
}

impl DmxAddress {
    pub fn new(universe_id: UniverseId, address: usize) -> Option<Self> {
        if address < 512 {
            Some(Self {
                universe_id,
                address,
            })
        } else {
            None
        }
    }

    pub fn universe_id(&self) -> UniverseId {
        self.universe_id
    }
    pub fn address(&self) -> usize {
        self.address
    }
}

#[derive(Clone, Copy)]
pub struct Universe {
    values: [u8; 512],
}

// TODO: LTP, HTP
// TODO: 別スレッドにする
impl Universe {
    pub fn new() -> Self {
        Self { values: [0; 512] }
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    pub(crate) fn set_value(&mut self, address: ResolvedAddress, value: u8) {
        self.values[address.address.address as usize] = value;
    }
}
