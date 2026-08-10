use std::collections::BTreeMap;

use burn::{
    Tensor,
    tensor::{Int, TensorData, backend::Backend},
};

use crate::{core::CompiledPuzzleDefinition, sim::SimError::MoveNotFound};

#[derive(thiserror::Error, Debug)]
pub enum SimError {
    #[error("move not found: {0}")]
    MoveNotFound(String),
}

type Result<T> = std::result::Result<T, SimError>;

#[derive(Clone)]
pub struct LoadedPuzzleDefinition<B: Backend> {
    device: B::Device,
    moves: Tensor<B, 3, Int>,
    move_map: BTreeMap<String, i32>,
    piece_index_map: Vec<i32>,
    state_len: usize,
}

impl<B: Backend> LoadedPuzzleDefinition<B> {
    pub fn load(puzzle_def: CompiledPuzzleDefinition, device: B::Device) -> Self {
        let num_moves = puzzle_def.moves.len();
        let mut nested_transforms = vec![];
        let mut move_map = BTreeMap::new();
        let max_state_map_len = puzzle_def
            .moves
            .iter()
            .map(|move_| {
                move_
                    .transform
                    .iter()
                    .map(|state_map| state_map.len())
                    .max()
                    .unwrap_or(0)
            })
            .max() // This max is technically unnecessary because all elements of the list should have the same value but wtvr
            .unwrap_or(0);

        for (i, move_) in puzzle_def.moves.into_iter().enumerate() {
            let padded_transform = move_
                .transform
                .iter()
                .map(|state_map| {
                    let mut padded_state_map = state_map.clone();
                    padded_state_map.resize(max_state_map_len, 0);
                    padded_state_map
                })
                .collect::<Vec<_>>();

            nested_transforms.push(padded_transform);
            move_map.insert(move_.name, i as i32);
        }

        let moves_tensordata = TensorData::new(
            nested_transforms.into_iter().flatten().flatten().collect(),
            [num_moves, puzzle_def.state_len, max_state_map_len],
        );

        let moves = Tensor::from_data(moves_tensordata, &device);

        Self {
            device,
            moves,
            move_map,
            piece_index_map: puzzle_def.piece_index_map,
            state_len: puzzle_def.state_len,
        }
    }
}

#[derive(Clone)]
pub struct Puzzle<'a, B: Backend> {
    state: Tensor<B, 1, Int>,
    loaded_puzzle: &'a LoadedPuzzleDefinition<B>,
}

impl<'a, B: Backend> Puzzle<'a, B> {
    pub fn new(loaded_puzzle: &'a LoadedPuzzleDefinition<B>, device: B::Device) -> Self {
        let state = Tensor::zeros([loaded_puzzle.state_len], &device);

        Self {
            state,
            loaded_puzzle,
        }
    }

    pub fn apply_move(&self, move_name: &str) -> Result<Self> {
        let move_index = self
            .loaded_puzzle
            .move_map
            .get(move_name)
            .ok_or_else(|| MoveNotFound(move_name.to_string()))?;
        let move_index = Tensor::from_data([*move_index], &self.loaded_puzzle.device);

        let move_ = self.loaded_puzzle.moves.clone().select(0, move_index);
        let shaped_state = self.state.clone().unsqueeze_dim(1);

        let new_state = move_.gather(1, shaped_state).squeeze();

        Ok(Self {
            state: new_state,
            ..self.clone()
        })
    }
}
