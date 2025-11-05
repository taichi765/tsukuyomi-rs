use crate::fixture::Fixture;
use crate::functions::Fader;
use crate::functions::Scene;
use crate::functions::{Context, Function, FunctionInfo, FunctionType};
use crate::plugins::Plugin;
use crate::plugins::artnet::ArtNetPlugin;
use crate::universe::{DmxAddress, Universe};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

/// Engine is the single source of true.
/// It also manages the timer.
pub struct Engine {
    /* ----- doc ----- */
    ///universe_id-> universe
    universes: HashMap<usize, Universe>,
    ///fixture_id-> fixture
    fixtures: HashMap<usize, Fixture>,
    ///function_id-> function
    functions: HashMap<usize, Box<dyn Function>>,
    function_infos: HashMap<usize, FunctionInfo>,

    /* ----- running ----- */
    ///function_id(unique)
    running_functions: HashSet<usize>,

    /* ----- id ----- */
    function_id_gen: IdGenerator,
    internal_function_id_gen: IdGenerator,
    fixture_id_gen: IdGenerator,
    universe_id_gen: IdGenerator,

    /* ----- IO ----- */
    output_plugin: Box<dyn Plugin>,
}

/* ---------- running ---------- */
impl Engine {
    //数ミリ秒ごとにEngine::run()から呼ぶ
    fn tick(&mut self) {
        let mut commands_list = Vec::new();
        for function_id in &self.running_functions {
            let function: &mut Box<dyn Function> = self.functions.get_mut(function_id).unwrap();

            commands_list.append(&mut function.run(
                &self.function_infos,
                &self.fixtures,
                &Context {
                    tick_duration: TICK_DURATION,
                },
            ));
        }

        for command in commands_list {
            match command {
                EngineCommand::StartFunction(function_id) => self.start_function(function_id),
                EngineCommand::StopFuntion(function_id) => self.stop_function(function_id),
                EngineCommand::WriteUniverse { address, value } => {
                    self.universe_mut(0).unwrap().set_value(address, value)
                }
                EngineCommand::StartFade {
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

    pub fn run(&mut self, function_id: usize) {
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
                FunctionType::Scene => (from_scene as &dyn Any).downcast_ref::<Scene>().unwrap(),
                _ => panic!("unimplemented type"),
            };

            let to_scene = self.get_function(to_id).as_ref();
            let to_scene = match to_scene.function_type() {
                FunctionType::Scene => (to_scene as &dyn Any).downcast_ref::<Scene>().unwrap(),
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
    pub fn new() -> Self {
        let mut universe_id_gen = IdGenerator::new();
        let universe_id = universe_id_gen.next();
        let mut universes = HashMap::new();
        universes.insert(universe_id, Universe::new(universe_id));
        Self {
            universes: universes,
            fixtures: HashMap::new(),
            functions: HashMap::new(),
            function_infos: HashMap::new(),
            running_functions: HashSet::new(),
            function_id_gen: IdGenerator::new(),
            internal_function_id_gen: IdGenerator::new_with_start(usize::MAX / 2),
            fixture_id_gen: IdGenerator::new(),
            universe_id_gen,
            output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()), //output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()),
        }
    }
    pub fn universe(&self, index: usize) -> Option<&Universe> {
        self.universes.get(&index)
    }
    pub(crate) fn universe_mut(&mut self, index: usize) -> Option<&mut Universe> {
        self.universes.get_mut(&index)
    }
    pub fn push_universe(&mut self, universe: Universe) -> Result<(), String> {
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
}

pub enum EngineCommand {
    /// if the function is already started, `Engine` do nothing.
    StartFunction(usize),
    /// if the function is already stoped, `Engine` do nothing.
    StopFuntion(usize),
    WriteUniverse {
        address: DmxAddress,
        value: u8,
    },
    StartFade {
        from_id: usize,
        to_id: usize,
        chaser_id: usize,
        duration: Duration,
    },
}

// helper funtions for test
impl EngineCommand {
    ///テスト用
    pub fn is_start_function(&self) -> bool {
        if let EngineCommand::StartFunction(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_start_function_and(&self, want: usize) -> bool {
        if let EngineCommand::StartFunction(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function(&self) -> bool {
        if let EngineCommand::StopFuntion(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function_and(&self, want: usize) -> bool {
        if let EngineCommand::StopFuntion(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe(&self) -> bool {
        if let EngineCommand::WriteUniverse { .. } = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe_and(&self, want: (u16, u8)) -> bool {
        if let EngineCommand::WriteUniverse { address, value } = self
            && DmxAddress::new(want.0).unwrap() == *address
            && want.1 == *value
        {
            return true;
        }
        false
    }
}

struct IdGenerator {
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
}

#[cfg(test)]
mod tests {
    use crate::functions::Scene;

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
        let scene = Scene::new(0, "this_should_work");
        assert!(engine.push_function(Box::new(scene)).is_ok());

        let scene_invalid = Scene::new(0, "this should be error");
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
