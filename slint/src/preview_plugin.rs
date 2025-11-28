use tsukuyomi_core::plugins::Plugin;

pub struct PreviewOutput {}

impl PreviewOutput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for PreviewOutput {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        Ok(())
    }
}
