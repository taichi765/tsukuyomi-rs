use std::fmt::Debug;

use crate::engine::OutputPluginId;

pub mod artnet;

pub trait Plugin: Send + Sync {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error>;
    fn id(&self) -> OutputPluginId;
}

impl Debug for dyn Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
