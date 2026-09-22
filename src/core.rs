use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Range};

use serde::{Deserialize, Serialize};

pub type Move<T> = Box<dyn Fn(&T) -> T + Send + Sync>;
pub type CompiledMove = Vec<Vec<i32>>;

pub type CompiledPieceState = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition<T> {
    pub slice: Range<i32>,
    pub pieces: Vec<i32>,
    pub states: Vec<T>,
}

pub struct PuzzleDefinition<T, U> {
    pub moves: HashMap<U, Move<T>>,
    pub solved_state: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPuzzleDefinition<T, U: Eq + Hash> {
    pub moves: HashMap<U, CompiledMove>,
    pub orbits: Vec<OrbitDefinition<T>>,
    pub piece_orbit_map: Vec<i32>,
    pub piece_index_map: Vec<i32>,
    pub solved_state: Vec<CompiledPieceState>,
}
