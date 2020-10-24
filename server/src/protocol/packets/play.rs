use crate::protocol::data_types::{DataType, Identifier, SizedDataType, VarInt};

#[derive(Constructor, IntoPacket)]
#[packet_id = 0x24]
pub struct JoinGame {
    entity_id: i32,
    is_hardcore: bool,
    gamemode: u8,
    previous_gamemode: i8,
    world_names: Vec<Identifier>,
    dimension_codec: Identifier,
    hashed_seed: i64,
    max_players: u8,
    level_type: String,
    view_distance: VarInt,
    reduced_debug_info: bool,
    enable_respawn_screen: bool,
}
