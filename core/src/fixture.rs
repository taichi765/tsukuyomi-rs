use crate::{
    doc::ModeNotFound,
    fixture_def::{FixtureDef, FixtureDefId},
    universe::{DmxAddress, UniverseId},
};

declare_id_newtype!(FixtureId);

#[derive(Clone, Copy, Debug)]
pub enum MergeMode {
    HTP,
    LTP,
}

//TODO: 占有するチャンネルの計算
// TODO: クロスユニバース
#[derive(Debug, Clone)]
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

    /// Number of channels in the current mode.
    pub fn footprint(&self, fixture_def: &FixtureDef) -> Result<usize, ModeNotFound> {
        let mode_name = self.fixture_mode();
        let mode = fixture_def.modes().get(mode_name).ok_or(ModeNotFound {
            fixture_def: fixture_def.id(),
            mode: String::from(mode_name),
        })?;
        Ok(mode.offset())
    }

    /// Enumurates all addresses ocuppied by this [Fixture].
    pub fn ocuppied_addresses(
        &self,
        fixture_def: &FixtureDef,
    ) -> Result<Vec<DmxAddress>, ModeNotFound> {
        let footprint = self.footprint(fixture_def)?;
        let address_base = self.address();
        let mut addresses = Vec::new();
        for i in 0..footprint {
            addresses.push(DmxAddress::new(address_base.value() + i).expect("address overflow"));
        }
        Ok(addresses)
    }
}
