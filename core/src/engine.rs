use crate::doc::Doc;
use crate::functions::{Fader, Function, FunctionType, StaticSceneData};
use crate::plugins::Plugin;
use crate::plugins::artnet::ArtNetPlugin;
use crate::universe::DmxAddress;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

/// Engine is the single source of true.
/// It also manages the timer.
pub struct Engine {
    doc: Arc<RwLock<Doc>>,
    running_functions: HashSet<usize>,
    output_plugin: Box<dyn Plugin>,
}

/* ---------- running ---------- */
impl Engine {
    //数ミリ秒ごとにEngine::run()から呼ぶ
    fn tick(&mut self) {
        let mut commands_list = Vec::new();
        for function_id in &self.running_functions {
            let function: &mut dyn Function =
                self.doc.get_function_mut(*function_id).unwrap().as_mut();

            commands_list.append(&mut function.run(
                &self.function_infos,
                &self.fixtures,
                TICK_DURATION,
            ));
        }

        for command in commands_list {
            match command {
                FunctionCommand::StartFunction(function_id) => self.start_function(function_id),
                FunctionCommand::StopFuntion(function_id) => self.stop_function(function_id),
                FunctionCommand::WriteUniverse { address, value } => {
                    self.universe_mut(0).unwrap().set_value(address, value)
                }
                FunctionCommand::StartFade {
                    from_id,
                    to_id,
                    chaser_id,
                    duration,
                } => self.start_fade(from_id, to_id, chaser_id, duration),
            }
        }
        self.output_plugin
            .send_dmx(0, &self.universe(0).unwrap().values().to_vec()[..])
            .unwrap();
        //println!("{:?}", self.universe(0).unwrap().values[0]); //アウトプット
    }

    pub fn start_loop(&mut self, function_id: usize) {
        println!("starting engine...");
        self.start_function(function_id);
        //let mut i: i32 = 0;
        loop {
            //print!("{}:", i);
            //let start = Instant::now();
            /*let names: Vec<String> = self
            .running_functions
            .iter()
            .map(|id| self.get_function(*id).name())
            .collect();*/
            //print!("{:?}", names);
            //print!("{}, ", self.running_functions.len());
            if self.running_functions.len() == 0 {
                println!("stopping engine");
                return;
            }
            self.tick();
            //i += 1;
            //println!("running late: {}μs", start.elapsed().as_millis());
            std::thread::sleep(TICK_DURATION);
        }
    }

    ///既にstartしてた場合は何もしない
    fn start_function(&mut self, function_id: usize) {
        self.running_functions.insert(function_id);
    }

    ///既にstopしてた/そもそも存在しなかった場合、何もしない
    fn stop_function(&mut self, function_id: usize) {
        self.running_functions.remove(&function_id);
    }

    fn start_fade(&mut self, from_id: usize, to_id: usize, chaser_id: usize, duration: Duration) {
        //必要な値だけを取り出す
        let (from_values, to_values) = {
            let from_scene = self.get_function(from_id).as_ref();
            let from_scene = match from_scene.function_type() {
                FunctionType::Scene => (from_scene as &dyn Any)
                    .downcast_ref::<StaticSceneData>()
                    .unwrap(),
                _ => panic!("unimplemented type"),
            };

            let to_scene = self.get_function(to_id).as_ref();
            let to_scene = match to_scene.function_type() {
                FunctionType::Scene => (to_scene as &dyn Any)
                    .downcast_ref::<StaticSceneData>()
                    .unwrap(),
                _ => panic!("unimplemented type"),
            };
            // TODO: 無駄なclone?
            (from_scene.values().clone(), to_scene.values().clone())
        };
        let fader = Fader::new(
            self.next_internal_function_id(),
            to_id,
            chaser_id,
            from_values,
            to_values,
            duration,
        );
        let fader_id = fader.id();
        self.push_function(Box::new(fader))
            .expect("functionの追加に失敗しました");
        self.start_function(fader_id);
    }
}

/* ---------- getter/setter, initialization ----------*/
impl Engine {
    pub fn new(doc: Arc<RwLock<Doc>>) -> Self {
        Self {
            doc: doc,
            running_functions: HashSet::new(),
            output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()), //output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()),
        }
    }
}

pub enum FunctionCommand {
    /// if the function is already started, `Engine` do nothing.
    StartFunction(usize),
    /// if the function is already stoped, `Engine` do nothing.
    StopFuntion(usize),
    WriteUniverse {
        fixture_id: usize,
        channel: u16,
        value: u8,
    },
    StartFade {
        from_id: usize,
        to_id: usize,
        chaser_id: usize,
        duration: Duration,
    },
}

// helper functions for test
impl FunctionCommand {
    ///テスト用
    pub fn is_start_function(&self) -> bool {
        if let FunctionCommand::StartFunction(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_start_function_and(&self, want: usize) -> bool {
        if let FunctionCommand::StartFunction(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function(&self) -> bool {
        if let FunctionCommand::StopFuntion(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function_and(&self, want: usize) -> bool {
        if let FunctionCommand::StopFuntion(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe(&self) -> bool {
        if let FunctionCommand::WriteUniverse { .. } = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe_and(&self, want: (u16, u8)) -> bool {
        if let FunctionCommand::WriteUniverse { address, value } = self
            && DmxAddress::new(want.0).unwrap() == *address
            && want.1 == *value
        {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_engine_init_empty() {
        let engine = Engine::new();
        assert_eq!(engine.universes.len(), 1);
        assert_eq!(engine.functions.len(), 0);
        assert_eq!(engine.fixtures.len(), 0);
    }

    #[test]
    fn test_engine_push_function_works() {
        let mut engine = Engine::new();
        let scene = StaticSceneData::new(0, "this_should_work");
        assert!(engine.push_function(Box::new(scene)).is_ok());

        let scene_invalid = StaticSceneData::new(0, "this should be error");
        assert!(engine.push_function(Box::new(scene_invalid)).is_err());
    }
    #[test]
    fn test_engine_push_fixture_works() {
        let mut engine = Engine::new();
        let fixture = Fixture::new(0, "this should work", 0);
        assert!(engine.push_fixture(fixture).is_ok());

        let fixture_invalid = Fixture::new(0, "this should be error", 1);
        assert!(engine.push_fixture(fixture_invalid).is_err());
    }
    #[test]
    fn test_engine_push_universe_works() {
        let mut engine = Engine::new();
        let universe = Universe::new(1);
        assert!(engine.push_universe(universe).is_ok());

        let universe_invalid = Universe::new(1);
        assert!(engine.push_universe(universe_invalid).is_err());
    }
}
