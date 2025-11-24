use crate::{
    doc::{Doc, DocCommand},
    engine::OutputPluginId,
    fixture::{Fixture, FixtureId},
    fixture_def::{FixtureDef, FixtureDefId},
    functions::{FunctionData, FunctionId},
    universe::UniverseId,
};
// TODO: エラー型の定義(enum or trait)
// TODO: 重複部分のマクロ定義？
pub struct AddFunction {
    function: Option<FunctionData>,
    function_id: FunctionId,
}

impl AddFunction {
    pub fn new(function: FunctionData) -> Self {
        Self {
            function_id: function.id(),
            function: Some(function),
        }
    }
}

impl DocCommand for AddFunction {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = self.function.take() {
            doc.add_function(f)
        } else {
            Err("function is already moved".into())
        }
    }
    fn revert(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = doc.remove_function(self.function_id) {
            self.function = Some(f);
            Ok(())
        } else {
            Err("function is already removed from doc".into())
        }
    }
}

pub struct AddFixture {
    fixture: Option<Fixture>,
    fixture_id: FixtureId,
}

impl AddFixture {
    pub fn new(fixture: Fixture) -> Self {
        Self {
            fixture_id: fixture.id(),
            fixture: Some(fixture),
        }
    }
}

impl DocCommand for AddFixture {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = self.fixture.take() {
            doc.add_fixture(f);
            Ok(())
        } else {
            Err("fixture is already moved".into())
        }
    }

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = doc.remove_fixture(self.fixture_id) {
            self.fixture = Some(f);
            Ok(())
        } else {
            Err("fixture is already removed".into())
        }
    }
}

pub struct AddFixtureDef {
    fixture_def_id: FixtureDefId,
    fixture_def: Option<FixtureDef>,
}

impl AddFixtureDef {
    pub fn new(fixture_def: FixtureDef) -> Self {
        Self {
            fixture_def_id: fixture_def.id(),
            fixture_def: Some(fixture_def),
        }
    }
}

impl DocCommand for AddFixtureDef {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(def) = self.fixture_def.take() {
            doc.add_fixture_def(def);
            Ok(())
        } else {
            Err("fixture definition is already moved".into())
        }
    }
    fn revert(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(def) = doc.remove_fixture_def(self.fixture_def_id) {
            self.fixture_def = Some(def);
            Ok(())
        } else {
            Err("fixture definition is already removed".into())
        }
    }
}

pub struct AddOutput {
    universe_id: UniverseId,
    plugin: OutputPluginId,
}

impl AddOutput {
    pub fn new(universe_id: UniverseId, plugin: OutputPluginId) -> Self {
        Self {
            universe_id,
            plugin,
        }
    }
}

impl DocCommand for AddOutput {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String> {
        doc.add_output(self.universe_id, self.plugin);
        Ok(())
    }

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String> {
        doc.remove_output(self.universe_id, self.plugin);
        Ok(())
    }
}
