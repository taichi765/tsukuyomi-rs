use crate::plugins::Plugin;

pub struct StdoutPlugin {}

impl StdoutPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for StdoutPlugin {
    fn send_dmx(&self, _universe_id: u8, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        for i in 0..10 {
            print!("{}, ", dmx_data[i])
        }
        println!();
        Ok(())
    }
}
