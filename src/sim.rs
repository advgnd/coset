use std::collections::BTreeMap;

use burn::{
    Tensor,
    tensor::{TensorData, backend::Backend},
};

use crate::core::{CompiledPuzzleDefinition, PuzzleDefinition};

struct Puzzle<B: Backend> {
    state: Tensor<B, 1>,
    moves: Tensor<B, 2>,
    move_map: BTreeMap<String, i32>,
    piece_index_map: Vec<i32>,
}

impl<B: Backend> Puzzle<B> {
    fn new(device: B::Device, puzzle_definition: CompiledPuzzleDefinition) -> Self {
        let state = Tensor::zeros([puzzle_definition.piece_index_map.len()], &device);

        let num_moves = puzzle_definition.moves.len();
        let mut nested_transforms = vec![];
        let mut move_map = BTreeMap::new();
        let max_state_map_len = puzzle_definition
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

        for (i, move_) in puzzle_definition.moves.into_iter().enumerate() {
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
            [
                num_moves,
                puzzle_definition.piece_index_map.len(),
                max_state_map_len,
            ],
        );

        let moves = Tensor::from_data(moves_tensordata, &device);

        Self {
            state,
            moves,
            move_map,
            piece_index_map: puzzle_definition.piece_index_map,
        }
    }
}
