use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use serde_json::json;

use crate::protocol::data_types::{DataType, Long, SizedDataType};
use crate::protocol::packets::{ClientboundPacket, FromPacket, IntoPacket, ServerboundPacket};

pub struct Request;

impl FromPacket for Request {
    fn from_packet(packet: ServerboundPacket) -> Result<Self> {
        // TODO make sure we are at the end of the packet here...
        // The request packet has no payload
        if packet.data().remaining() == 0 {
            Ok(Request)
        } else {
            Err(anyhow!("Bytes remaining in request packet"))
        }
    }
}

pub struct Response {
    response: String,
}

impl Response {
    pub fn new(
        players_max: usize,
        players_online: usize,
        motd: String,
        favicon: Option<String>,
    ) -> Response {
        let favicon = favicon.unwrap_or_else(|| "".to_string());

        // TODO dynamic protocol number and version name
        let response = json!({
            "version": {
                "name": "MC Server 1.15.2",
                "protocol": 578,
            },
            "players": {
                "max": players_max,
                "online": players_online,
            },
            "description": {
                "text": motd,
            },
            "favicon": favicon
        })
        .to_string();

        Response { response }
    }
}

impl IntoPacket for Response {
    fn into_packet(self) -> ClientboundPacket {
        // TODO Is there a better way to make a temporary buffer?
        let mut data = BytesMut::with_capacity(self.response.size());
        self.response.write_to(&mut data);

        ClientboundPacket::new(0x00, data)
    }
}

pub struct Ping {
    payload: Long,
}

impl FromPacket for Ping {
    fn from_packet(packet: ServerboundPacket) -> Result<Ping> {
        let mut data = packet.data();

        Ok(Ping {
            payload: Long::read_from(&mut data)?,
        })
    }
}

pub struct Pong {
    payload: Long,
}

impl Pong {
    pub fn new(ping: Ping) -> Pong {
        Pong {
            payload: ping.payload,
        }
    }
}

impl IntoPacket for Pong {
    fn into_packet(self) -> ClientboundPacket {
        let mut data = BytesMut::with_capacity(self.payload.size());
        self.payload.write_to(&mut data);

        ClientboundPacket::new(0x01, data)
    }
}
