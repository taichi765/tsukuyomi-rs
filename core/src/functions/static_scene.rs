use std::collections::HashMap;
use std::time::Duration;

use super::{FunctionData, FunctionRuntime};
use crate::{engine::FunctionCommand, functions::FunctionDataGetters};

pub type SceneValue = HashMap<u16, u8>;

pub struct StaticSceneData {
    id: usize,
    name: String,
    /// fixture_id->values
    values: HashMap<usize, SceneValue>,
}

impl FunctionDataGetters for StaticSceneData {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

//TODO: 同じfixture_idかつ同じchannelにvalueを設定できちゃう
impl StaticSceneData {
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

    pub fn insert_value(&mut self, fixture_id: usize, value: SceneValue) {
        self.values.insert(fixture_id, value);
    }
}

pub struct StaticSceneRuntime {}

impl StaticSceneRuntime {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl FunctionRuntime for StaticSceneRuntime {
    fn run(&mut self, data: &FunctionData, _tick_duration: Duration) -> Vec<FunctionCommand> {
        let FunctionData::StaticScene(data) = data else {
            panic!("unknown data type")
        };

        let mut commands = Vec::new();
        for (fixture_id, scene_value) in &data.values {
            for (channel, value) in scene_value {
                commands.push(FunctionCommand::WriteUniverse {
                    fixture_id: *fixture_id,
                    channel: *channel,
                    value: *value,
                });
            }
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::Duration;

    #[test]
    fn static_scene_runtime_writes_commands_for_single_fixture() {
        let mut scene = StaticSceneData::new(1, "test");
        let mut sv = SceneValue::new();
        sv.insert(1, 128);
        sv.insert(2, 255);
        scene.insert_value(10, sv);

        let mut runtime = StaticSceneRuntime::new();
        let commands = runtime.run(
            &FunctionData::StaticScene(scene),
            Duration::from_millis(100),
        );

        let mut found = vec![];
        for cmd in commands {
            match cmd {
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => {
                    found.push((fixture_id, channel, value));
                }
                _ => panic!("unexpected command"),
            }
        }
        assert_eq!(found.len(), 2);
        assert!(found.contains(&(10, 1, 128)));
        assert!(found.contains(&(10, 2, 255)));
    }

    #[test]
    fn static_scene_runtime_writes_commands_for_multiple_fixtures() {
        let mut scene = StaticSceneData::new(2, "multi");

        let mut sv1 = SceneValue::new();
        sv1.insert(1, 10);
        scene.insert_value(1, sv1);

        let mut sv2 = SceneValue::new();
        sv2.insert(5, 200);
        sv2.insert(6, 201);
        scene.insert_value(2, sv2);

        let mut runtime = StaticSceneRuntime::new();
        let commands = runtime.run(&FunctionData::StaticScene(scene), Duration::from_millis(50));

        let expected: HashSet<_> = vec![(1, 1, 10), (2, 5, 200), (2, 6, 201)]
            .into_iter()
            .collect();

        let found: HashSet<_> = commands
            .into_iter()
            .map(|cmd| match cmd {
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => (fixture_id, channel, value),
                _ => panic!("unexpected command"),
            })
            .collect();

        assert_eq!(found, expected);
    }
}
