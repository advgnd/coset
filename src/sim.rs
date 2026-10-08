use std::{fmt::Debug, hash::Hash};

use burn::{
    Tensor,
    tensor::{Bool, DataError, Int, backend::Backend},
};

use crate::{
    compiler::decompile_state,
    core::Puzzle,
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

#[derive(Clone)]
pub struct PuzzleStates<'a, B: Backend, T, U: Eq + Hash> {
    num_states: usize,
    states: Tensor<B, 2, Int>,
    dfa_states: Tensor<B, 1, Int>,
    validity_mask: Tensor<B, 1, Bool>,
    puzzle: &'a Puzzle<B, T, U>,
}

#[derive(Clone)]
pub struct PuzzleState<'a, B: Backend, T, U: Eq + Hash>(PuzzleStates<'a, B, T, U>);

impl<'a, T: Debug + Clone, U: Clone + Eq + Hash, B: Backend> PuzzleStates<'a, B, T, U> {
    pub fn new(num_states: usize, puzzle: &'a Puzzle<B, T, U>) -> Self {
        let states = puzzle
            .solved_state
            .clone()
            .unsqueeze::<2>()
            .expand([num_states as i32, -1]);
        let dfa_states = Tensor::ones([num_states as i32], &puzzle.device);
        let validity_mask = Tensor::ones([num_states as i32], &puzzle.device);

        Self {
            num_states,
            states,
            dfa_states,
            validity_mask,
            puzzle,
        }
    }

    #[inline(always)]
    fn select_moves(&self, move_indices: Tensor<B, 1, Int>) -> Tensor<B, 3, Int> {
        self.puzzle.moves.clone().select(0, move_indices)
    }

    #[inline(always)]
    fn shape_moves(&self, moves: Tensor<B, 3, Int>) -> Tensor<B, 4, Int> {
        moves
            .unsqueeze::<4>()
            .expand([self.num_states as i32, -1, -1, -1])
    }

    #[inline(always)]
    fn shape_state(&self, num_moves: Option<usize>) -> Tensor<B, 4, Int> {
        let shaped_state = self.states.clone().unsqueeze_dims::<4>(&[1, 3]);

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

    #[inline(always)]
    fn apply_dfa(
        &self,
        move_indices: Option<Tensor<B, 1, Int>>,
    ) -> (Tensor<B, 1, Int>, Tensor<B, 1, Bool>) {
        let mut dfa_states = self
            .puzzle
            .dfa_next_map
            .clone()
            .select(0, self.dfa_states.clone());

        let mut validity_mask = self
            .puzzle
            .dfa_validity_mask
            .clone()
            .select(0, self.dfa_states.clone());

        if let Some(move_indices) = move_indices {
            dfa_states = dfa_states.select(1, move_indices.clone());
            validity_mask = validity_mask.select(1, move_indices);
        }

        (dfa_states.flatten(0, 1), validity_mask.flatten(0, 1))
    }

    pub fn apply_move(&self, move_key: &U) -> Result<Self, U> {
        let move_index = self
            .puzzle
            .definition
            .moves
            .get_index_of(move_key)
            .ok_or_else(|| MoveNotFound(move_key.clone()))?;
        let move_index = Tensor::from_data([move_index], &self.puzzle.device);
        let move_ = self.shape_moves(self.select_moves(move_index.clone()));

        let new_state = self.apply_moves_tensor(move_, None);
        let (dfa_states, validity_mask) = self.apply_dfa(Some(move_index));

        Ok(Self {
            states: new_state,
            dfa_states,
            validity_mask,
            ..self.clone()
        })
    }

    pub fn apply_moves(&self, move_keys: &[&U]) -> Result<Self, U> {
        let move_indices = move_keys
            .iter()
            .map(|move_| {
                self.puzzle
                    .definition
                    .moves
                    .get_index_of(*move_)
                    .ok_or_else(|| MoveNotFound((*move_).clone()))
            })
            .collect::<Result<Vec<_>, U>>()?;
        let move_indices = Tensor::from_data(move_indices.as_slice(), &self.puzzle.device);
        let moves = self.shape_moves(self.select_moves(move_indices.clone()));

        let new_state = self.apply_moves_tensor(moves, Some(move_keys.len()));
        let (dfa_states, validity_mask) = self.apply_dfa(Some(move_indices));

        Ok(Self {
            num_states: move_keys.len() * self.num_states,
            states: new_state,
            dfa_states,
            validity_mask,
            ..self.clone()
        })
    }

    pub fn apply_all_moves(&self) -> Result<Self, U> {
        let moves = self.shape_moves(self.puzzle.moves.clone());

        let new_state = self.apply_moves_tensor(moves, Some(self.puzzle.definition.moves.len()));
        let (dfa_states, validity_mask) = self.apply_dfa(None);

        Ok(Self {
            num_states: self.puzzle.definition.moves.len() * self.num_states,
            states: new_state,
            dfa_states,
            validity_mask,
            ..self.clone()
        })
    }

    pub fn raw_states(&self) -> Tensor<B, 2, Int> {
        self.states.clone()
    }

    pub fn states(&self) -> Result<Vec<Vec<T>>, U> {
        let raw_data = self
            .states
            .clone()
            .select(1, self.puzzle.piece_index_map.clone())
            .to_data()
            .to_vec()
            .map_err(SimError::DataError)?;

        let raw_data = raw_data.chunks(self.puzzle.definition.solved_state.len());

        Ok(raw_data
            .map(|state| {
                state
                    .into_iter()
                    .enumerate()
                    .map(|(piece_id, piece_state)| {
                        decompile_state(
                            *piece_state,
                            &self.puzzle.definition.orbits
                                [self.puzzle.definition.piece_orbit_map[piece_id] as usize],
                        )
                        .expect("illegal state cannot be stored in PuzzleState(s)")
                    })
                    .collect()
            })
            .collect())
    }
}

impl<'a, T: Debug + Clone, U: Clone + Eq + Hash, B: Backend> PuzzleState<'a, B, T, U> {
    pub fn new(loaded_puzzle: &'a Puzzle<B, T, U>) -> Self {
        Self(PuzzleStates::new(1, loaded_puzzle))
    }

    pub fn apply_move(&self, move_key: &U) -> Result<Self, U> {
        self.0.apply_move(move_key).map(|new_state| Self(new_state))
    }

    pub fn apply_moves(&self, move_keys: &[&U]) -> Result<PuzzleStates<'a, B, T, U>, U> {
        self.0.apply_moves(move_keys)
    }

    pub fn apply_all_moves(&self) -> Result<PuzzleStates<'a, B, T, U>, U> {
        self.0.apply_all_moves()
    }

    pub fn raw_state(&self) -> Tensor<B, 2, Int> {
        self.0.raw_states().squeeze_dim(0)
    }

    pub fn state(&self) -> Result<Vec<T>, U> {
        self.0
            .states()
            .map(|mut states| states.pop().unwrap_or_default())
    }
}
