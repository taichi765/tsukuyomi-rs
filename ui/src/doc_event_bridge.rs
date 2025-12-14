use std::sync::mpsc::Sender;

use tracing::debug;
use tsukuyomi_core::doc::{DocEvent, DocObserver};
use tsukuyomi_core::engine::EngineCommand;

/// Sends [`EngineCommand`][tsukuyomi_core::engine::EngineCommand] when [`Doc`][tsukuyomi_core::doc::Doc] changed
pub struct DocEventBridge {
    command_tx: Sender<EngineCommand>,
}

impl DocEventBridge {
    pub fn new(command_tx: Sender<EngineCommand>) -> Self {
        Self { command_tx }
    }

    fn send(&mut self, command: EngineCommand) {
        self.command_tx
            .send(command)
            .expect("failed to send message to engine");
    }
}

impl DocObserver for DocEventBridge {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::UniverseSettingsChanged => {
                self.send(EngineCommand::OutputMapChanged);
            }
            DocEvent::UniverseAdded(id) => self.send(EngineCommand::UniverseAdded(*id)),
            DocEvent::UniverseRemoved(id) => self.send(EngineCommand::UniverseRemoved(*id)),
            _ => (),
        }
    }
}

impl Drop for DocEventBridge {
    fn drop(&mut self) {
        debug!("DocEventBridge is dropping");
    }
}
