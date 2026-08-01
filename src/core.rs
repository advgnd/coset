use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceMove {
    pub piece: i32,
    pub state_map: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub name: String,
    pub piece_map: Vec<PieceMove>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orbit {
    pub piece_ids: Vec<i32>,
    pub state_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleRepr {
    pub moves: Vec<Move>,
    pub orbits: Vec<Orbit>,
    pub state_size: usize,
}
