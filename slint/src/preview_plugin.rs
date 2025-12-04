use tsukuyomi_core::{engine::OutputPluginId, plugins::Plugin};

pub struct PreviewOutput {
    id: OutputPluginId,
}

impl PreviewOutput {
    pub fn new() -> Self {
        Self {
            id: OutputPluginId::new(),
        }
    }
}

impl Plugin for PreviewOutput {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        println!("send_dmx is called");
        Ok(())
    }

    fn id(&self) -> OutputPluginId {
        self.id
    }
}
