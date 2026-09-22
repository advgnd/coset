use std::{collections::HashMap, fmt::Debug, hash::Hash};

use burn::{
    Tensor,
    tensor::{DataError, Int, TensorData, backend::Backend},
};

use crate::{
    compiler::decompile_state,
    core::{CompiledPuzzleDefinition, OrbitDefinition},
    sim::SimError::MoveNotFound,
};

#[derive(thiserror::Error, Debug)]
pub enum SimError<U> {
    #[error("move not found: {0}")]
    MoveNotFound(U),
    #[error("tensor data error: {0}")]
    DataError(DataError),
}

type Result<T, U> = std::result::Result<T, SimError<U>>;

pub struct LoadedPuzzleDefinition<T, U, B: Backend> {
    device: B::Device,
    num_moves: usize,
    num_pieces: usize,
    moves: Tensor<B, 3, Int>,
    move_map: HashMap<U, i32>,
    orbits: Vec<OrbitDefinition<T>>,
    piece_orbit_map: Vec<i32>,
    piece_index_map: Tensor<B, 1, Int>,
    solved_state: Tensor<B, 1, Int>,
}

impl<T, U: Eq + Hash, B: Backend> LoadedPuzzleDefinition<T, U, B> {
    pub fn load(puzzle_def: CompiledPuzzleDefinition<T, U>, device: B::Device) -> Self {
        let num_moves = puzzle_def.moves.len();
        let num_pieces = puzzle_def.solved_state.len();
        let mut nested_transforms = vec![];
        let mut move_map = HashMap::new();
        let max_states_len = puzzle_def
            .orbits
            .iter()
            .map(|orbit| orbit.states.len())
            .max()
            .unwrap_or(0) as usize;

        for (i, (name, move_)) in puzzle_def.moves.into_iter().enumerate() {
            let padded_transform = move_
                .iter()
                .map(|state_map| {
                    let mut padded_state_map = state_map.clone();
                    padded_state_map.resize(max_states_len, 0);
                    padded_state_map
                })
                .collect::<Vec<_>>();

            nested_transforms.push(padded_transform);
            move_map.insert(name, i as i32);
        }

        let moves_tensordata = TensorData::new(
            nested_transforms.into_iter().flatten().flatten().collect(),
            [num_moves, num_pieces, max_states_len],
        );

        let moves = Tensor::from_data(moves_tensordata, &device);
        let piece_index_map = Tensor::from_data(puzzle_def.piece_index_map.as_slice(), &device);
        let solved_state = Tensor::from_data(puzzle_def.solved_state.as_slice(), &device);

        Self {
            device,
            num_moves,
            num_pieces,
            moves,
            move_map,
            orbits: puzzle_def.orbits,
            piece_orbit_map: puzzle_def.piece_orbit_map,
            piece_index_map,
            solved_state,
        }
    }
}

pub struct PuzzleStates<'a, T, U, B: Backend> {
    num_states: usize,
    state: Tensor<B, 2, Int>,
    loaded_puzzle: &'a LoadedPuzzleDefinition<T, U, B>,
}

pub struct PuzzleState<'a, T, U, B: Backend>(PuzzleStates<'a, T, U, B>);

impl<'a, T: Debug + Clone, U: Clone + Eq + Hash, B: Backend> PuzzleStates<'a, T, U, B> {
    pub fn new(num_states: usize, loaded_puzzle: &'a LoadedPuzzleDefinition<T, U, B>) -> Self {
        let state = loaded_puzzle
            .solved_state
            .clone()
            .unsqueeze::<2>()
            .expand([num_states as i32, -1]);

        Self {
            num_states,
            state,
            loaded_puzzle,
        }
    }

    #[inline(always)]
    fn shape_moves(&self, moves: Tensor<B, 3, Int>) -> Tensor<B, 4, Int> {
        moves
            .unsqueeze::<4>()
            .expand([self.num_states as i32, -1, -1, -1])
    }

    #[inline(always)]
    fn select_moves(&self, move_indices: Tensor<B, 1, Int>) -> Tensor<B, 3, Int> {
        self.loaded_puzzle.moves.clone().select(0, move_indices)
    }

    #[inline(always)]
    fn shape_state(&self, num_moves: Option<usize>) -> Tensor<B, 4, Int> {
        let shaped_state = self.state.clone().unsqueeze_dims::<4>(&[1, 3]);

        if let Some(num_moves) = num_moves {
            shaped_state.expand([-1, num_moves as i32, -1, -1])
        } else {
            shaped_state
        }
    }

    #[inline(always)]
    fn apply_moves_tensor(
        &self,
        moves: Tensor<B, 4, Int>,
        num_moves: Option<usize>,
    ) -> Tensor<B, 2, Int> {
        let shaped_state = self.shape_state(num_moves);
        let new_state = moves.gather(3, shaped_state);

        if let Some(_) = num_moves {
            new_state.squeeze_dim::<3>(3).flatten(0, 1)
        } else {
            new_state.squeeze_dims(&[1, 3])
        }
    }

    pub fn apply_move(&self, move_key: &U) -> Result<Self, U> {
        let move_index = self
            .loaded_puzzle
            .move_map
            .get(move_key)
            .ok_or_else(|| MoveNotFound(move_key.clone()))?;
        let move_index = Tensor::from_data([*move_index], &self.loaded_puzzle.device);
        let move_ = self.shape_moves(self.select_moves(move_index));

        let new_state = self.apply_moves_tensor(move_, None);

        Ok(Self {
            num_states: self.num_states,
            state: new_state,
            loaded_puzzle: self.loaded_puzzle,
        })
    }

    pub fn apply_moves(&self, move_keys: &[&U]) -> Result<Self, U> {
        let move_indices = move_keys
            .iter()
            .map(|move_| {
                self.loaded_puzzle
                    .move_map
                    .get(move_)
                    .ok_or_else(|| MoveNotFound((*move_).clone()))
                    .copied()
            })
            .collect::<Result<Vec<_>, U>>()?;
        let move_indices = Tensor::from_data(move_indices.as_slice(), &self.loaded_puzzle.device);
        let moves = self.shape_moves(self.select_moves(move_indices));

        let new_state = self.apply_moves_tensor(moves, Some(move_keys.len()));

        Ok(Self {
            num_states: move_keys.len() * self.num_states,
            state: new_state,
            loaded_puzzle: self.loaded_puzzle,
        })
    }

    pub fn apply_all_moves(&self) -> Result<Self, U> {
        let moves = self.shape_moves(self.loaded_puzzle.moves.clone());

        let new_state = self.apply_moves_tensor(moves, Some(self.loaded_puzzle.num_moves));

        Ok(Self {
            num_states: self.loaded_puzzle.num_moves * self.num_states,
            state: new_state,
            loaded_puzzle: self.loaded_puzzle,
        })
    }

    pub fn raw_states(&self) -> Tensor<B, 2, Int> {
        self.state.clone()
    }

    pub fn states(&self) -> Result<Vec<Vec<T>>, U> {
        let raw_data = self
            .state
            .clone()
            .select(1, self.loaded_puzzle.piece_index_map.clone())
            .to_data()
            .to_vec()
            .map_err(SimError::DataError)?;
        
        let raw_data = raw_data.chunks(self.loaded_puzzle.num_pieces);

        Ok(raw_data
            .map(|state|
                state.into_iter()
            .enumerate()
            .map(|(piece_id, piece_state)| {
                decompile_state(
                    *piece_state,
                    &self.loaded_puzzle.orbits[self.loaded_puzzle.piece_orbit_map[piece_id] as usize],
                )
                .expect("illegal state cannot be stored in PuzzleState(s)")
            })
            .collect()
            ).collect())
    }
}

impl<'a, T: Debug + Clone, U: Clone + Eq + Hash, B: Backend> PuzzleState<'a, T, U, B> {
    pub fn new(loaded_puzzle: &'a LoadedPuzzleDefinition<T, U, B>) -> Self {
        Self(PuzzleStates::new(1, loaded_puzzle))
    }

    pub fn apply_move(&self, move_key: &U) -> Result<Self, U> {
        self.0.apply_move(move_key).map(|new_state| Self(new_state))
    }

    pub fn apply_moves(&self, move_keys: &[&U]) -> Result<PuzzleStates<'a, T, U, B>, U> {
        self.0.apply_moves(move_keys)
    }

    pub fn apply_all_moves(&self) -> Result<PuzzleStates<'a, T, U, B>, U> {
        self.0.apply_all_moves()
    }

    pub fn raw_state(&self) -> Tensor<B, 2, Int> {
        self.0.raw_states().squeeze_dim(0)
    }

    pub fn state(&self) -> Result<Vec<T>, U> {
        self.0.states().map(|mut states| states.pop().unwrap_or_default())
    }
}
