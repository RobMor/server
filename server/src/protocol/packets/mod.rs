use anyhow::Result;
use bytes::BytesMut;

pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub struct ServerboundPacket {
    packet_id: i32,
    data: BytesMut,
}

impl ServerboundPacket {
    pub fn new(packet_id: i32, data: BytesMut) -> ServerboundPacket {
        ServerboundPacket { packet_id, data }
    }

    pub fn packet_id(&self) -> i32 {
        self.packet_id
    }

    pub fn data(self) -> BytesMut {
        self.data
    }

    pub fn parse<T>(self) -> Result<T>
    where
        T: FromPacket,
    {
        T::from_packet(self)
    }
}

pub struct ClientboundPacket {
    packet_id: i32,
    data: BytesMut,
}

impl ClientboundPacket {
    pub fn new(packet_id: i32, data: BytesMut) -> ClientboundPacket {
        ClientboundPacket { packet_id, data }
    }

    pub fn packet_id(&self) -> i32 {
        self.packet_id
    }

    pub fn data(self) -> BytesMut {
        self.data
    }
}

pub trait FromPacket: Sized {
    fn from_packet(packet: ServerboundPacket) -> Result<Self>;
}

pub trait IntoPacket: Sized {
    fn into_packet(self) -> ClientboundPacket;
}
