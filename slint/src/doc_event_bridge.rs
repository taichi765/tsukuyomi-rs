use std::sync::mpsc::Sender;

use tsukuyomi_core::doc::{DocEvent, DocObserver};
use tsukuyomi_core::engine::EngineCommand;

/// Sends [`EngineCommand`][tsukuyomi_core::engine::EngineCommand] when [`Doc`][tsukuyomi_core::doc::Doc] changed
pub struct DocEventBridge {
    command_tx: Sender<EngineCommand>,
}

impl DocObserver for DocEventBridge {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::UniverseSettingsChanged => self
                .command_tx
                .send(EngineCommand::OutputMapChanged)
                .unwrap(),
            _ => (),
        }
    }
}

impl DocEventBridge {
    pub fn new(command_tx: Sender<EngineCommand>) -> Self {
        Self { command_tx }
    }
}
