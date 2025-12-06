use std::net::{SocketAddr, UdpSocket};

use artnet_protocol::{ArtCommand, Output};

use crate::{engine::OutputPluginId, plugins::Plugin, universe::UniverseId};

const ARTNET_PORT: u16 = 6454;

pub struct ArtNetPlugin {
    id: OutputPluginId,
    socket: UdpSocket,
    destination: SocketAddr,
}

impl ArtNetPlugin {
    pub fn new(target_ip: &str) -> Result<Self, std::io::Error> {
        println!("binding udp socket...");
        //適当なポートから送信
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        println!("done.");

        let destination = format!("{}:{}", target_ip, ARTNET_PORT)
            .parse()
            .expect("invalid target IP address");
        Ok(Self {
            id: OutputPluginId::new(),
            socket,
            destination,
        })
    }
}

impl Plugin for ArtNetPlugin {
    fn send_dmx(&self, universe_id: UniverseId, dmx_data: &[u8]) -> Result<(), std::io::Error> {
        if cfg!(debug_assertions) {
            println!("send_dmx is called")
        }
        let command = ArtCommand::Output(Output {
            port_address: universe_id.value().into(),
            data: dmx_data.to_vec().into(),
            ..Default::default()
        });
        let buf = command.write_to_buffer().unwrap();
        self.socket.send_to(&buf, self.destination)?;
        Ok(())
    }

    fn id(&self) -> OutputPluginId {
        self.id
    }
}
