use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Range};

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

pub type Move<T> = dyn Fn(&T) -> T;
pub type CompiledMove = Vec<Vec<i32>>;

pub type CompiledPieceState = i32;

pub type DfaEvaluator<T, U> = dyn Fn(&T, &U) -> Option<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition<T> {
    pub slice: Range<i32>,
    pub pieces: Vec<i32>,
    pub states: Vec<T>,
}

pub struct PuzzleDefinition<T, U, V> {
    pub moves: HashMap<U, Box<Move<T>>>,
    pub solved_state: Vec<T>,
    pub dfa_eval: Box<DfaEvaluator<V, U>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPuzzleDefinition<T, U: Eq + Hash> {
    pub moves: IndexSet<U>,
    pub compiled_moves: Vec<CompiledMove>,
    pub orbits: Vec<OrbitDefinition<T>>,
    pub piece_orbit_map: Vec<i32>,
    pub piece_index_map: Vec<i32>,
    pub solved_state: Vec<CompiledPieceState>,
    pub allowed_moves: Vec<Vec<i32>>,
    pub next_dfa_states: Vec<Vec<i32>>,
}
