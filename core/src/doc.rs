use std::collections::HashMap;

use crate::{fixture::Fixture, functions::FunctionData, universe::Universe};

pub trait DocCommand {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String>;

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String>;
}

pub struct Doc {
    universes: Vec<Universe>,
    fixtures: HashMap<usize, Fixture>,
    functions: HashMap<usize, FunctionData>,
    /*function_infos: HashMap<usize, FunctionInfo>, */
}

/*struct IdGenerator {
    id: usize,
}
impl IdGenerator {
    fn new() -> Self {
        Self { id: 0 }
    }
    fn new_with_start(start: usize) -> Self {
        Self { id: start }
    }

    fn next(&mut self) -> usize {
        let id = self.id;
        self.id += 1;
        id
    }
}*/

// public
impl Doc {
    pub fn new() -> Self {
        Self {
            universes: Vec::new(),
            fixtures: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_function_data(&self, function_id: usize) -> Option<&FunctionData> {
        self.functions.get(&function_id)
    }
}

// internal
impl Doc {
    pub(crate) fn add_function(&mut self, function: FunctionData) -> Result<(), String> {
        if self.functions.contains_key(&function.id()) {
            return Err(format!("function id {} already exsists", function.id(),));
        }
        self.functions.insert(function.id(), function);
        //TODO: self.update_function_infos();
        Ok(())
    }

    pub(crate) fn remove_function(&mut self, function_id: usize) -> Option<FunctionData> {
        self.functions.remove(&function_id)
    }

    //仮
    /*pub(crate) fn universe(&self, index: usize) -> Option<&Universe> {
        self.universes.get(&index)
    }
    pub(crate) fn universe_mut(&mut self, index: usize) -> Option<&mut Universe> {
        self.universes.get_mut(&index)
    }
    fn update_function_infos(&mut self) {
        self.function_infos = self
            .functions
            .iter()
            .map(|(id, func)| {
                (
                    *id,
                    FunctionInfo {
                        id: func.id(),
                        function_type: func.function_type(),
                    },
                )
            })
            .collect();
    }
    pub(crate) fn push_universe(&mut self, universe: Universe) -> Result<(), String> {
        if self.universes.contains_key(&universe.id()) {
            return Err(format!("universe id {} already exsists", universe.id()));
        }
        self.universes.insert(universe.id(), universe);
        Ok(())
    }
    pub fn next_universe_id(&mut self) -> usize {
        self.universe_id_gen.next()
    }

    pub fn get_fixture(&self, id: usize) -> Option<&Fixture> {
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
    }

    //TODO: Resultを返すようにしたい
    pub fn get_function(&self, id: usize) -> &Box<dyn Function> {
        if let Some(some) = self.functions.get(&id) {
            some
        } else {
            panic!("{}", format!("function id {} not found", id))
        }
    }
    pub fn push_function(&mut self, function: Box<dyn Function>) -> Result<(), String> {
        if self.functions.contains_key(&function.id()) {
            return Err(format!("function id {} already exsists", function.id(),));
        }
        self.functions.insert(function.id(), function);
        self.update_function_infos();
        Ok(())
    }
    pub fn next_function_id(&mut self) -> usize {
        self.function_id_gen.next()
    }
    pub(crate) fn next_internal_function_id(&mut self) -> usize {
        self.internal_function_id_gen.next()
    }*/
}
