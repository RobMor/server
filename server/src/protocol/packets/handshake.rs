use anyhow::{anyhow, Result};

use crate::protocol::data_types::{DataType, SizedDataType, UnsignedShort, VarInt};
use crate::protocol::packets::{FromPacket, ServerboundPacket};

#[derive(Copy, Clone)]
pub enum NextState {
    Status,
    Login,
}

pub struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: UnsignedShort,
    next_state: NextState,
}

impl Handshake {
    pub fn protocol_version(&self) -> VarInt {
        self.protocol_version
    }

    pub fn server_address(&self) -> String {
        self.server_address.clone()
    }

    pub fn server_port(&self) -> UnsignedShort {
        self.server_port
    }

    pub fn next_state(&self) -> NextState {
        self.next_state
    }
}

impl FromPacket for Handshake {
    fn from_packet(packet: ServerboundPacket) -> Result<Handshake> {
        let mut buf = packet.data();

        Ok(Handshake {
            protocol_version: VarInt::read_from(&mut buf)?,
            server_address: String::read_from_sized(&mut buf, 255)?,
            server_port: UnsignedShort::read_from(&mut buf)?,
            next_state: match VarInt::read_from(&mut buf)?.value() {
                1 => NextState::Status,
                2 => NextState::Login,
                s => return Err(anyhow!("Unknown next state {}", s)),
            },
        })
    }
}
