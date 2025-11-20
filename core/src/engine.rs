use crate::doc::Doc;
use crate::functions::{Fader, FunctionRuntime, FunctionType, StaticSceneData};
use crate::plugins::Plugin;
use crate::plugins::artnet::ArtNetPlugin;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

/// Engine is the single source of true.
/// It also manages the timer.
pub struct Engine {
    doc: Arc<RwLock<Doc>>,
    active_runtimes: HashMap<usize, Box<dyn FunctionRuntime>>,
    output_plugin: Box<dyn Plugin>,
}

/* ---------- running ---------- */
impl Engine {
    pub fn new(doc: Arc<RwLock<Doc>>) -> Self {
        Self {
            doc: doc,
            active_runtimes: HashMap::new(),
            output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()), //output_plugin: Box::new(ArtNetPlugin::new("127.0.0.1").unwrap()),
        }
    }

    pub fn start_loop(&mut self, function_id: usize) {
        println!("starting engine...");
        self.start_function(function_id);
        loop {
            if self.active_runtimes.len() == 0 {
                println!("stopping engine");
                return;
            }
            self.tick();
            std::thread::sleep(TICK_DURATION);
        }
    }

    //数ミリ秒ごとにEngine::run()から呼ぶ
    fn tick(&mut self) {
        let mut commands_list = Vec::new();
        {
            let doc = self.doc.read().unwrap();
            for (function_id, runtime) in &mut self.active_runtimes {
                let data = doc.get_function_data(*function_id).unwrap();
                commands_list.append(&mut runtime.run(data, TICK_DURATION));
            }
        }

        for command in commands_list {
            match command {
                FunctionCommand::StartFunction(function_id) => self.start_function(function_id),
                FunctionCommand::StopFuntion(function_id) => self.stop_function(function_id),
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => unimplemented!("誰に渡すのか"),
                FunctionCommand::StartFade {
                    from_id,
                    to_id,
                    chaser_id,
                    duration,
                } => self.start_fade(from_id, to_id, chaser_id, duration),
            }
        }
        /*self.output_plugin
        .send_dmx(0, &self.universe(0).unwrap().values().to_vec()[..])
        .unwrap();*/
    }

    ///既にstartしてた場合は何もしない
    fn start_function(&mut self, function_id: usize) {
        let doc = self.doc.read().unwrap();
        let runtime = doc.get_function_data(function_id).unwrap().create_runtime();
        self.active_runtimes.insert(function_id, runtime);
    }

    ///既にstopしてた/そもそも存在しなかった場合、何もしない
    fn stop_function(&mut self, function_id: usize) {
        self.active_runtimes.remove(&function_id);
    }

    fn start_fade(&mut self, from_id: usize, to_id: usize, chaser_id: usize, duration: Duration) {
        //必要な値だけを取り出す
        /*let (from_values, to_values) = {
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
        self.start_function(fader_id);*/
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
/*impl FunctionCommand {
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
}*/

#[cfg(test)]
mod tests {}
