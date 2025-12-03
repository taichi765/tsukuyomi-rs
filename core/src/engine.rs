use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;

use crate::doc::Doc;
use crate::fixture::FixtureId;
use crate::functions::{FunctionCommand, FunctionId, FunctionRuntime};
use crate::plugins::Plugin;
use crate::readonly::ReadOnly;
use crate::universe::{UniverseId, UniverseState};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

declare_id_newtype!(OutputPluginId);

// TODO: unwrap, expectを減らす
/// Orchestrates [`FunctionRuntime`]s
pub struct Engine {
    doc: ReadOnly<Doc>,
    command_rx: Receiver<EngineCommand>,
    message_tx: Sender<EngineMessage>,

    active_runtimes: HashMap<FunctionId, Box<dyn FunctionRuntime>>,
    output_plugins: HashMap<OutputPluginId, Box<dyn Plugin>>,
    universe_states: HashMap<UniverseId, UniverseState>,
    output_map_cache: HashMap<OutputPluginId, Vec<UniverseId>>,
    live_values: HashMap<(FixtureId, String), u8>,

    should_shutdown: bool,
}

impl Engine {
    pub fn new(
        doc: ReadOnly<Doc>,
        command_rx: Receiver<EngineCommand>,
        message_tx: Sender<EngineMessage>,
    ) -> Self {
        Self {
            doc: doc,
            command_rx,
            message_tx,
            active_runtimes: HashMap::new(),
            output_plugins: HashMap::new(),
            universe_states: HashMap::new(),
            output_map_cache: HashMap::new(),
            live_values: HashMap::new(),
            should_shutdown: false,
        }
    }

    pub fn start_loop(mut self) {
        println!("starting engine...");
        loop {
            self.handle_engine_commands();

            self.universe_states.iter_mut().for_each(|(_, u)| u.clear());

            // apply live values before running function, so LTP channels will be overridden.
            let live_values = self.live_values.clone();
            live_values.iter().for_each(|((id, ch), v)| {
                self.write_universe(*id, ch, *v);
            });

            self.run_active_functions();

            self.output_map_cache.par_iter().for_each(|(p_id, u_ids)| {
                let plugin = self.output_plugins.get(p_id).unwrap();
                u_ids.iter().for_each(|u_id| {
                    let universe_data = self.universe_states.get(u_id).unwrap();
                    if let Err(e) = plugin.send_dmx(u_id.value(), &universe_data.values()) {
                        self.message_tx
                            .send(EngineMessage::ErrorOccured(EngineError {
                                context: ErrorContext::SendingDmx {
                                    universe_id: *u_id,
                                    plugin_id: *p_id,
                                },
                                source: Box::new(e),
                            }))
                            .unwrap();
                    }
                });
            });

            if self.should_shutdown {
                break;
            }
            std::thread::sleep(TICK_DURATION); //TODO: フレームレートを安定させる
        }
    }

    fn add_output_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.output_plugins.insert(OutputPluginId::new(), plugin);
    }

    fn handle_engine_commands(&mut self) {
        if let Ok(cmd) = self.command_rx.try_recv() {
            match cmd {
                EngineCommand::StartFunction(id) => self.start_function(id),
                EngineCommand::StopFunction(id) => self.stop_function(id),
                EngineCommand::AddPlugin(p) => self.add_output_plugin(p),
                EngineCommand::AddUniverse => {
                    self.universe_states.insert(
                        UniverseId::new(self.universe_states.len() as u8), // TODO 雑 IDをUIスレッドに返すとかしてもいい
                        UniverseState::new(),
                    );
                }
                EngineCommand::SetLiveValue {
                    fixture_id,
                    channel,
                    value,
                } => {
                    if value == 0 {
                        // エントリが存在しなかった場合も何もしない
                        let _ = self.live_values.remove(&(fixture_id, channel));
                    } else {
                        let _ = self.live_values.insert((fixture_id, channel), value);
                    }
                }
                EngineCommand::OutputMapChanged => self.update_output_map_cache(),
                EngineCommand::Shutdown => self.should_shutdown = true,
            }
        }
    }

    fn run_active_functions(&mut self) {
        let mut commands_list = Vec::new();
        {
            for (function_id, runtime) in &mut self.active_runtimes {
                let doc = self.doc.read();
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
    fn start_function(&mut self, function_id: FunctionId) {
        let runtime = self
            .doc
            .read()
            .get_function_data(function_id)
            .expect(format!("could not find function with id {}", function_id).as_str())
            .create_runtime();
        self.active_runtimes.insert(function_id, runtime);
    }

    ///既にstopしてた/そもそも存在しなかった場合、何もしない
    fn stop_function(&mut self, function_id: FunctionId) {
        self.active_runtimes.remove(&function_id);
    }

    fn write_universe(
        &mut self,
        fixture_id: FixtureId,
        channel: String,
        value: u8,
    ) -> Result<(), ResolveError> {
        let (universe_id, address) = self.doc.read().resolve_address(fixture_id, &channel)?;
        let universe = self.universe_states.get_mut(&universe_id).unwrap();
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

    fn update_output_map_cache(&mut self) {
        let doc = self.doc.read();
        let mut new_map: HashMap<OutputPluginId, Vec<UniverseId>> = HashMap::new();
        for (u_id, setting) in doc.universe_settings() {
            setting.output_plugins().iter().for_each(|p_id| {
                if let Some(universes) = new_map.get_mut(p_id) {
                    universes.push(*u_id);
                } else {
                    new_map.insert(*p_id, vec![*u_id]);
                }
            });
        }

        self.output_map_cache = new_map;
    }
}

/// Message from the main thread to [`Engine`]
pub enum EngineCommand {
    StartFunction(FunctionId),
    StopFunction(FunctionId),
    AddPlugin(Box<dyn Plugin>),
    AddUniverse,
    SetLiveValue {
        fixture_id: FixtureId,
        channel: String,
        value: u8,
    },
    OutputMapChanged, // FIXME: Docの監視にメインスレッドを介すのは正しいか？
    Shutdown,
}

/// Message from [`Engine`] to the main thread
pub enum EngineMessage {
    ErrorOccured(EngineError),
}

/// The errors occured in [`Engine`]
#[derive(Debug)]
pub struct EngineError {
    context: ErrorContext,
    source: Box<dyn Error + Send + Sync>,
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.context, self.source)
    }
}

impl Error for EngineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub enum ErrorContext {
    ResolvingAddress {
        fixture_id: FixtureId,
        channel: String,
    },
    RunningFunction {
        function_id: FunctionId,
    },
    SendingDmx {
        universe_id: UniverseId,
        plugin_id: OutputPluginId,
    },
}

#[cfg(test)]
mod tests {}
