use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    fixture::{Fixture, MergeMode},
    fixture_def::FixtureDef,
    functions::FunctionData,
    universe::DmxAddress,
};

pub trait DocCommand {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String>;

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String>;
}

pub struct Doc {
    fixtures: HashMap<Uuid, Fixture>,
    fixture_definitions: HashMap<Uuid, FixtureDef>,
    functions: HashMap<Uuid, FunctionData>,
}

pub(crate) struct ResolvedAddress {
    address: DmxAddress,
    merge_mode: MergeMode,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            fixture_definitions: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_function_data(&self, function_id: Uuid) -> Option<&FunctionData> {
        self.functions.get(&function_id)
    }

    pub(crate) fn add_function(&mut self, function: FunctionData) -> Result<(), String> {
        if self.functions.contains_key(&function.id()) {
            return Err(format!("function id {} already exsists", function.id(),));
        }
        self.functions.insert(function.id(), function);
        //TODO: self.update_function_infos();
        Ok(())
    }

    pub(crate) fn remove_function(&mut self, function_id: Uuid) -> Option<FunctionData> {
        self.functions.remove(&function_id)
    }

    pub(crate) fn resolve_address(
        &self,
        fixture_id: Uuid,
        channel: &str,
    ) -> Option<ResolvedAddress> {
        if let Some(fixture) = self.fixtures.get(&fixture_id) {
            Some(ResolvedAddress {
                address: fixture.address(),
                merge_mode: fixture.fixture_def(),
            })
        } else {
            None
        }
    }

    //仮

    /*pub fn get_fixture(&self, id: usize) -> Option<&Fixture> {
        self.fixtures.get(&id)
    }
    pub fn push_fixture(&mut self, fixture: Fixture) -> Result<(), String> {
        if self.fixtures.contains_key(&fixture.id()) {
            return Err(format!("fxiture id {} already exsits", fixture.id(),));
        }
        self.fixtures.insert(fixture.id(), fixture);
        Ok(())
    }
    pub fn next_fixture_id(&mut self) -> usize {
        self.fixture_id_gen.next()
    }*/
}
