use std::ops::{Add, Sub};

use crate::{doc::ResolvedAddress, fixture::MergeMode};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct UniverseId(u8);

impl UniverseId {
    pub fn new(v: u8) -> Self {
        Self(v)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DmxAddress(usize);

impl DmxAddress {
    pub fn new(value: usize) -> Option<Self> {
        if value < 512 { Some(Self(value)) } else { None }
    }

    pub fn value(&self) -> usize {
        self.0
    }

    pub fn checked_add(self, rhs: usize) -> Option<DmxAddress> {
        DmxAddress::new(self.0 + rhs)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<usize> {
        self.0.checked_sub(rhs.0)
    }
}

#[derive(Clone, Copy)]
pub struct UniverseState {
    values: [u8; 512],
}

impl UniverseState {
    pub fn new() -> Self {
        Self { values: [0; 512] }
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    pub(crate) fn clear(&mut self) {
        self.values.fill(0);
    }

    pub(crate) fn set_value(&mut self, address: ResolvedAddress, value: u8) {
        match address.merge_mode {
            MergeMode::HTP => {
                if value > self.values[address.address.value()] {
                    self.values[address.address.value()] = value
                }
            }
            MergeMode::LTP => self.values[address.address.value()] = value,
        }
    }
}
