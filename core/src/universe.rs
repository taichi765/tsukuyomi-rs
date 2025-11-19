#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmxAddress(u16);

impl DmxAddress {
    pub fn new(address: u16) -> Option<Self> {
        if address < 512 {
            Some(Self(address))
        } else {
            None
        }
    }

    pub fn from_usize(address: usize) -> Option<Self> {
        if let Ok(addr_u16) = u16::try_from(address) {
            Self::new(addr_u16)
        } else {
            None
        }
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
    pub fn as_u16(&self) -> u16 {
        self.0
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

    pub fn set_value(&mut self, address: DmxAddress, value: u8) {
        self.values[address.as_usize()] = value;
    }
}
