use std::{collections::HashMap, fmt::Debug, ops::Range};

use serde::{Deserialize, Serialize};

pub type PuzzleMove<T> = Box<dyn Fn(&T) -> T + Send + Sync>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub struct PieceStateStub;
pub type CompiledPieceState = i32;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition<T> {
    pub slice: Range<i32>,
    pub pieces: Vec<i32>,
    pub states: Vec<T>,
}

pub struct PuzzleDefinition<T> {
    pub moves: HashMap<String, PuzzleMove<T>>,
    pub solved_state: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledMoveDefinition {
    pub name: String,
    pub transform: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPuzzleDefinition<T> {
    pub moves: Vec<CompiledMoveDefinition>,
    pub orbits: Vec<OrbitDefinition<T>>,
    pub piece_orbit_map: Vec<i32>,
    pub piece_index_map: Vec<i32>,
    pub solved_state: Vec<CompiledPieceState>,
}
