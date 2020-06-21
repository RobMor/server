use bytes::BytesMut;

use crate::protocol::packets::{ClientboundPacket, IntoPacket};
use crate::protocol::data_types::{
    VarInt,
    DataType,
    SizedDataType,
};

pub struct JoinGame {
    entity_id: i32,
    gamemode: u8,
    dimension: i32,
    hashed_seed: i64,
    max_players: u8,
    level_type: String,
    view_distance: VarInt,
    reduced_debug_info: bool,
    enable_respawn_screen: bool,
}

impl JoinGame {
    pub fn new(
        entity_id: i32,
        gamemode: u8,
        dimension: i32,
        hashed_seed: i64,
        level_type: String,
        view_distance: i32,
        reduced_debug_info: bool,
        enable_respawn_screen: bool,
    ) -> JoinGame {
        JoinGame {
            entity_id,
            gamemode,
            dimension,
            hashed_seed,
            max_players: 0,
            level_type,
            view_distance: VarInt::new(view_distance),
            reduced_debug_info,
            enable_respawn_screen, 
        }
    }
}

impl IntoPacket for JoinGame {
    fn into_packet(self) -> ClientboundPacket {
        let mut data = BytesMut::with_capacity(
            self.entity_id.size() +
            self.gamemode.size() +
            self.dimension.size() +
            self.hashed_seed.size() +
            self.max_players.size() +
            self.level_type.size() +
            self.view_distance.size() +
            self.reduced_debug_info.size() +
            self.enable_respawn_screen.size()
        );

        self.entity_id.write_to(&mut data);
        self.gamemode.write_to(&mut data);
        self.dimension.write_to(&mut data);
        self.hashed_seed.write_to(&mut data);
        self.max_players.write_to(&mut data);
        self.level_type.write_to(&mut data);
        self.view_distance.write_to(&mut data);
        self.reduced_debug_info.write_to(&mut data);
        self.enable_respawn_screen.write_to(&mut data);

        ClientboundPacket::new(0x26, data)
    }
}
