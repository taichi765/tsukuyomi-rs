pub mod doc_event_bridge;
pub mod fader_view_bridge;
pub mod preview_2d;
pub mod preview_3d;

use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak, mpsc};
use std::time::Duration;

use i_slint_backend_winit::WinitWindowAccessor;

use slint::wgpu_27::{WGPUConfiguration, WGPUSettings};
use slint::{Timer, TimerMode};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tsukuyomi_core::command_manager::CommandManager;
use tsukuyomi_core::commands::doc_commands;
use tsukuyomi_core::doc::{DocEventBus, DocHandle, DocStore};
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::fixture::FixtureId;
use tsukuyomi_core::fixture_def::ChannelKind;
use tsukuyomi_core::{
    commands,
    commands::DocCommand,
    doc::DocObserver,
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, FixtureDef, FixtureMode},
    readonly::ReadOnly,
    universe::{DmxAddress, UniverseId},
};

use crate::doc_event_bridge::DocEventBridge;
use crate::fader_view_bridge::setup_fader_view;
use crate::preview_2d::setup_2d_preview;
use crate::preview_3d::setup_3d_preview;
// TODO: tsukuyomi_core::prelude使いたい

slint::include_modules!();

pub fn run_main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // HACK: depending on the order of initialization, it would cause crash.
    let doc = Arc::new(RwLock::new(DocStore::new()));
    let mut event_bus = DocEventBus::new();

    // HACK: Initialize all observers before changing Doc.
    // If Doc is changed before engine initialized, states in observers(and engine) would be invalid.
    let (engine_handle, command_tx, error_rx, _bridge) =
        setup_engine(ReadOnly::new(Arc::clone(&doc)), &mut event_bus);

    let ui = setup_window().expect("failed to setup ui");

    let mut doc_commands = Vec::new();

    for i in 1..5 {
        doc_commands.push(Box::new(doc_commands::AddUniverse::new(UniverseId::new(i))) as _);
    }

    let (mut dc, fixture_id) = create_some_presets();
    doc_commands.append(&mut dc);

    setup_fader_view(&ui, command_tx.clone(), fixture_id);
    let (mut dc, mut update_2d_preview) = setup_2d_preview(
        &ui,
        ReadOnly::new(Arc::clone(&doc)),
        &mut event_bus,
        command_tx.clone(),
    );
    doc_commands.append(&mut dc);
    setup_3d_preview(&ui);

    let mut command_manager = CommandManager::new(DocHandle::new(doc, event_bus));
    doc_commands.into_iter().for_each(|cmd| {
        command_manager.execute(cmd).unwrap();
    });

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(33), move || {
        update_2d_preview();
    });

    ui.run()?;

    if let Err(e) = command_tx.send(EngineCommand::Shutdown) {
        eprintln!("failed to send message to engine:{}", e);
    }

    engine_handle.join().unwrap();

    Ok(())
}

fn setup_engine(
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
) -> (
    std::thread::JoinHandle<()>,
    Sender<EngineCommand>,
    Receiver<EngineMessage>,
    Arc<RwLock<DocEventBridge>>,
) {
    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

    let bridge = Arc::new(RwLock::new(DocEventBridge::new(command_tx.clone())));
    let bridge_weak: Weak<RwLock<dyn DocObserver>> = Arc::downgrade(&bridge) as _;
    event_bus.subscribe(bridge_weak);

    let engine = Engine::new(doc, command_rx, error_tx);

    let engine_handle = std::thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();
    (engine_handle, command_tx, error_rx, bridge)
}

fn setup_window() -> Result<AppWindow, Box<dyn Error>> {
    slint::BackendSelector::new()
        .require_wgpu_27(WGPUConfiguration::Automatic(WGPUSettings::default()))
        .select()
        .expect("unable to create Slint backend WGPU based renderer");

    let ui = AppWindow::new()?;
    // TODO: language switch(preferences)
    slint::select_bundled_translation("en".into()).unwrap();

    let ui_handle = ui.as_weak();
    ui.on_start_drag(move || {
        let ui = ui_handle.unwrap();
        ui.window().with_winit_window(|w| w.drag_window());
    });

    let ui_handle = ui.as_weak();
    ui.on_minimize(move || {
        let ui = ui_handle.unwrap();
        ui.window().set_minimized(true);
    });

    // TODO: toggle-fullscreen

    let ui_handle = ui.as_weak();
    ui.on_close(move || {
        let ui = ui_handle.unwrap();
        ui.window().hide().unwrap()
    });

    Ok(ui)
}

fn create_some_presets() -> (Vec<Box<dyn DocCommand>>, FixtureId) {
    let mut commands: Vec<Box<dyn DocCommand>> = Vec::new();

    let mut fixture_def = FixtureDef::new("Test Manufacturer".into(), "Test Model".into());
    fixture_def.insert_channel(
        "Dimmer".into(),
        ChannelDef::new(MergeMode::HTP, ChannelKind::Dimmer),
    );
    fixture_def.insert_channel(
        "Red".into(),
        ChannelDef::new(MergeMode::HTP, ChannelKind::Red),
    );
    let mut channel_order: HashMap<String, Option<usize>> = HashMap::new();
    channel_order.insert("Dimmer".into(), Some(0));
    channel_order.insert("Red".into(), Some(1));
    let mode = FixtureMode::new(channel_order);
    fixture_def.insert_mode("Mode 1".into(), mode);
    let fixture_def_id = fixture_def.id();
    commands.push(Box::new(commands::doc_commands::AddFixtureDef::new(
        fixture_def,
    )));

    let fixture = Fixture::new(
        "Fixture",
        UniverseId::new(1),
        DmxAddress::new(0).unwrap(),
        fixture_def_id,
        "Mode 1".into(),
    );

    let fixture_id = fixture.id();
    commands.push(Box::new(commands::doc_commands::AddFixture::new(fixture)));

    /*let mut scene = StaticSceneData::new("My Scene");
    let mut sv = SceneValue::new();
    sv.insert("Dimmer".into(), 200);
    sv.insert("Red".into(), 100);
    scene.insert_value(fixture_id, sv);
    let scene_id = scene.id();

    commands.push(Box::new(commands::doc_commands::AddFunction::new(
        FunctionData::StaticScene(scene),
    )));*/

    (commands, fixture_id)
}
