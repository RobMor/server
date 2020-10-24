use anyhow::Result;
use bytes::BytesMut;
use log::trace;
use uuid::Uuid;

use crate::protocol::data_types::{DataType, SizedDataType};
use crate::protocol::packets::{ClientboundPacket, FromPacket, IntoPacket, ServerboundPacket};

pub struct Start {
    username: String,
}

impl Start {
    pub fn username(self) -> String {
        self.username
    }
}

impl FromPacket for Start {
    fn from_packet(packet: ServerboundPacket) -> Result<Self> {
        let mut data = packet.data();

        Ok(Start {
            username: String::read_from_sized(&mut data, 16)?,
        })
    }
}

pub struct EncryptionRequest {
    server_id: String, // Always empty...
    public_key: Vec<u8>,
    verify_token: Vec<u8>,
}

impl EncryptionRequest {
    pub fn new(rsa: Vec<u8>, verify_token: [u8; 4]) -> EncryptionRequest {
        EncryptionRequest {
            server_id: "".to_string(),
            public_key: rsa,
            verify_token: verify_token.to_vec(),
        }
    }
}

impl IntoPacket for EncryptionRequest {
    fn into_packet(self) -> ClientboundPacket {
        let mut data = BytesMut::with_capacity(
            self.server_id.size() + self.public_key.size() + self.verify_token.size(),
        );
        self.server_id.write_to(&mut data);
        self.public_key.write_to(&mut data);
        self.verify_token.write_to(&mut data);
        trace!("Data Size: {}", data.len());

        ClientboundPacket::new(0x01, data)
    }
}

pub struct EncryptionResponse {
    shared_secret: Vec<u8>,
    verify_token: Vec<u8>,
}

impl EncryptionResponse {
    pub fn into_parts(self) -> (Vec<u8>, Vec<u8>) {
        (self.shared_secret, self.verify_token)
    }
}

impl FromPacket for EncryptionResponse {
    fn from_packet(packet: ServerboundPacket) -> Result<EncryptionResponse> {
        let mut data = packet.data();

        Ok(EncryptionResponse {
            shared_secret: Vec::<u8>::read_from_sized(&mut data, 128)?,
            verify_token: Vec::<u8>::read_from_sized(&mut data, 128)?,
        })
    }
}

pub struct Success {
    uuid: Uuid,
    username: String,
}

impl Success {
    pub fn new(uuid: &Uuid, username: &str) -> Success {
        Success {
            uuid: uuid.clone(),
            username: username.to_string(),
        }
    }
}

impl IntoPacket for Success {
    fn into_packet(self) -> ClientboundPacket {
        let mut data = BytesMut::with_capacity(self.uuid.size() + self.username.size());
        self.uuid.write_to(&mut data);
        self.username.write_to(&mut data);

        ClientboundPacket::new(0x02, data)
    }
}
