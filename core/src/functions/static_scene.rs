use std::collections::HashMap;
use std::time::Duration;

use super::Function;
use super::{FunctionInfo, FunctionType};
use crate::engine::FunctionCommand;
use crate::fixture::Fixture;
use crate::universe::DmxAddress;

pub struct StaticScene {
    id: usize,
    name: String,
    /// fixture_id->values
    values: HashMap<usize, SceneValue>,
}

//TODO: 同じfixture_idかつ同じchannelにvalueを設定できちゃう
impl StaticScene {
    pub fn new(id: usize, name: &str) -> Self {
        Self {
            id: id,
            name: String::from(name),
            values: HashMap::new(),
        }
    }

    pub fn values(&self) -> &HashMap<usize, SceneValue> {
        &self.values
    }
    //TODO: insertかaddの方が良い？
    pub fn push_value(&mut self, fixture_id: usize, value: SceneValue) {
        self.values.insert(fixture_id, value);
    }
}

impl Function for StaticScene {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    ///sceneは自分でstopしない(Chaserに任せる)
    fn run(
        &mut self,
        _function_infos: &HashMap<usize, FunctionInfo>,
        fixtures: &HashMap<usize, Fixture>,
        _tick_duration: Duration,
    ) -> Vec<EngineCommand> {
        let mut commands = Vec::new();
        for (fixture_id, scene_value) in data.values {
            for (channel, value) in scene_value {
                commands.push(FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value: value,
                });
            }
        }
        commands
    }
    fn function_type(&self) -> FunctionType {
        FunctionType::Scene
    }
}

pub type SceneValue = HashMap<u16, u8>;

#[cfg(test)]
mod tests {
    //use std::time::Duration;

    //use super::*;

    #[test]
    fn test_scene_works() {
        /*let mut scene = Scene::new(0, "");
        let scene_value = SceneValue1 {
            fixture_id: 0,
            channel: 10,
            value: 123,
        };
        scene.push_value(scene_value);
        let fixture = Fixture::new(0, "", 5);
        let mut fixture_map = HashMap::new();
        fixture_map.insert(0, fixture);
        let context = &Context {
            tick_duration: Duration::ZERO,
        };

        let commands = scene.run(&HashMap::new(), &fixture_map, context);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].is_write_universe_and((15, 123)));*/
        unimplemented!()
    }
}
