use std::collections::HashMap;
use uuid::Uuid;

use crate::ai::Ai;
use crate::grid::Grid;
use crate::proto::Player;

#[derive(Debug)]
#[allow(dead_code)]
pub struct GameStatus {
    board: Grid,
    player_list: Vec<Player>,
    spectators: Vec<Uuid>,
    bots: HashMap<Uuid, Box<dyn Ai>>,
    current_player: usize,
    current_turn_number: usize,
}
