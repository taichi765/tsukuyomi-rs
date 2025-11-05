pub mod artnet;
pub mod stdout;

pub trait Plugin {
    fn send_dmx(&self, universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error>;
}
