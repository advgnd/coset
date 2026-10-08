use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Range};

use burn::{
    Tensor,
    tensor::{Bool, Int, TensorData, backend::Backend},
};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

pub type Move<T> = dyn Fn(&T) -> T;
pub type CompiledMove = Vec<Vec<i32>>;

pub type CompiledPieceState = i32;

pub type DfaEvaluator<T, U> = dyn Fn(&T, &U) -> Option<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition<T> {
    pub slice: Range<usize>,
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
    pub dfa_validity_mask: Vec<Vec<bool>>,
    pub dfa_next_map: Vec<Vec<i32>>,
}

pub struct Puzzle<B: Backend, T, U: Eq + Hash> {
    pub device: B::Device,
    pub definition: CompiledPuzzleDefinition<T, U>,
    pub moves: Tensor<B, 3, Int>,
    pub piece_index_map: Tensor<B, 1, Int>,
    pub solved_state: Tensor<B, 1, Int>,
    pub dfa_validity_mask: Tensor<B, 2, Bool>,
    pub dfa_next_map: Tensor<B, 2, Int>,
}

fn pad_nested_vec<T: Clone>(
    nested_vec: &Vec<Vec<T>>,
    pad_value: T,
    pad_size: usize,
) -> Vec<Vec<T>> {
    let padded_vec = nested_vec
        .iter()
        .cloned()
        .map(|mut vec| {
            vec.resize(pad_size, pad_value.clone());
            vec
        })
        .collect();

    padded_vec
}

impl<B: Backend, T, U: Eq + Hash> Puzzle<B, T, U> {
    pub fn load(puzzle_def: CompiledPuzzleDefinition<T, U>, device: B::Device) -> Self {
        let num_moves = puzzle_def.moves.len();
        let num_pieces = puzzle_def.solved_state.len();
        let num_dfa_states = puzzle_def.dfa_next_map.len();
        let max_states_len = puzzle_def
            .orbits
            .iter()
            .map(|orbit| orbit.states.len())
            .max()
            .unwrap_or(0) as usize;

        let padded_transforms = puzzle_def
            .compiled_moves
            .iter()
            .map(|move_| pad_nested_vec(move_, -1, max_states_len))
            .collect::<Vec<_>>();

        let moves_tensordata = TensorData::new(
            padded_transforms.into_iter().flatten().flatten().collect(),
            [num_moves, num_pieces, max_states_len],
        );
        let moves = Tensor::from_data(moves_tensordata, &device);

        let dfa_validity_mask_tensordata = TensorData::new(
            puzzle_def
                .dfa_validity_mask
                .iter()
                .cloned()
                .flatten()
                .collect(),
            [num_dfa_states, num_moves],
        );
        let dfa_validity_mask = Tensor::from_data(dfa_validity_mask_tensordata, &device);

        let dfa_next_map_tensordata = TensorData::new(
            puzzle_def.dfa_next_map.iter().cloned().flatten().collect(),
            [num_dfa_states, num_moves],
        );
        let dfa_next_map = Tensor::from_data(dfa_next_map_tensordata, &device);

        let piece_index_map = Tensor::from_data(puzzle_def.piece_index_map.as_slice(), &device);
        let solved_state = Tensor::from_data(puzzle_def.solved_state.as_slice(), &device);

        Self {
            device,
            definition: puzzle_def,
            moves,
            piece_index_map,
            solved_state,
            dfa_validity_mask,
            dfa_next_map,
        }
    }
}
