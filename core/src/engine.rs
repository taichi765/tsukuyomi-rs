use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;

use crate::doc::{Doc, ResolveError};
use crate::functions::{FunctionCommand, FunctionRuntime};
use crate::plugins::Plugin;
use crate::universe::{UniverseId, UniverseState};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

pub enum EngineCommand {
    StartFunction(Uuid),
    StopFunction(Uuid),
    AddPlugin(Box<dyn Plugin>),
    Shutdown,
}

/// Engine is the single source of true.
/// It also manages the timer.
pub struct Engine {
    doc: Arc<Doc>,
    command_rx: Receiver<EngineCommand>,

    active_runtimes: HashMap<Uuid, Box<dyn FunctionRuntime>>,
    output_plugins: HashMap<Uuid, Box<dyn Plugin>>,
    universe_states: HashMap<UniverseId, UniverseState>,
    plugin_universe_map_cache: HashMap<Uuid, Vec<UniverseId>>,

    should_shutdown: bool,
}

impl Engine {
    pub fn new(doc: Arc<Doc>, command_rx: Receiver<EngineCommand>) -> Self {
        Self {
            doc: doc,
            command_rx,
            active_runtimes: HashMap::new(),
            output_plugins: HashMap::new(),
            universe_states: HashMap::new(),
            plugin_universe_map_cache: HashMap::new(),
            should_shutdown: false,
        }
    }

    pub fn start_loop(mut self) {
        println!("starting engine...");
        loop {
            if let Ok(cmd) = self.command_rx.try_recv() {
                match cmd {
                    EngineCommand::StartFunction(id) => self.start_function(id),
                    EngineCommand::StopFunction(id) => self.stop_function(id),
                    EngineCommand::AddPlugin(p) => self.add_output_plugin(p),
                    EngineCommand::Shutdown => self.should_shutdown = true,
                }
            }

            self.run_active_functions();

            self.plugin_universe_map_cache
                .par_iter()
                .for_each(|(p_id, u_ids)| {
                    let plugin = self.output_plugins.get(p_id).unwrap();
                    u_ids.iter().for_each(|u_id| {
                        let universe_data = self.universe_states.get(u_id).unwrap();
                        plugin
                            .send_dmx((*u_id).into(), &universe_data.values())
                            .expect("something went wrong");
                    });
                });

            if self.should_shutdown {
                break;
            }
            std::thread::sleep(TICK_DURATION);
        }
    }

    fn add_output_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.output_plugins.insert(Uuid::new_v4(), plugin);
    }

    fn run_active_functions(&mut self) {
        let mut commands_list = Vec::new();
        {
            for (function_id, runtime) in &mut self.active_runtimes {
                let data = self.doc.get_function_data(*function_id).unwrap();
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
                } => self
                    .write_universe(fixture_id, channel, value)
                    .expect("something went wrong"),
                FunctionCommand::StartFade {
                    from_id,
                    to_id,
                    chaser_id,
                    duration,
                } => self.start_fade(from_id, to_id, chaser_id, duration),
            }
        }
    }

    ///既にstartしてた場合は何もしない
    fn start_function(&mut self, function_id: Uuid) {
        let runtime = self
            .doc
            .get_function_data(function_id)
            .expect(format!("could not find function with id {}", function_id).as_str())
            .create_runtime();
        self.active_runtimes.insert(function_id, runtime);
    }

    ///既にstopしてた/そもそも存在しなかった場合、何もしない
    fn stop_function(&mut self, function_id: Uuid) {
        self.active_runtimes.remove(&function_id);
    }

    fn write_universe(
        &mut self,
        fixture_id: Uuid,
        channel: String,
        value: u8,
    ) -> Result<(), ResolveError> {
        let address = self.doc.resolve_address(fixture_id, &channel)?;
        let universe = self
            .universe_states
            .get_mut(&address.address.universe_id())
            .unwrap();
        universe.set_value(address, value);
        Ok(())
    }

    fn start_fade(&mut self, from_id: Uuid, to_id: Uuid, chaser_id: Uuid, duration: Duration) {
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

    fn update_plugin_universe_map_cache(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
