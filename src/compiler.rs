use std::collections::{BTreeMap, BTreeSet};

use crate::core::{
    CompiledMoveDefinition, CompiledPieceState, CompiledPuzzleDefinition, OrbitDefinition,
    PieceState, PuzzleDefinition, PuzzleMove,
};

#[derive(thiserror::Error, Debug)]
pub enum CompilerError<T: PieceState> {
    #[error("cannot decode invalid compiled piece state {0}")]
    InvalidCompiledPieceState(i32),

    #[error("cannot encode invalid piece state {0:?}")]
    InvalidPieceState(T),
}

type Result<T, U> = std::result::Result<T, CompilerError<U>>;

pub fn decompile_state<T: PieceState>(
    compiled_piece_state: CompiledPieceState,
    orbit: &OrbitDefinition<T>,
) -> Result<T, T> {
    orbit
        .states
        .get(compiled_piece_state as usize)
        .ok_or(CompilerError::InvalidCompiledPieceState(
            compiled_piece_state,
        ))
        .cloned()
}

pub fn compile_state<T: PieceState>(
    piece_state: &T,
    orbit: &OrbitDefinition<T>,
) -> Result<CompiledPieceState, T> {
    orbit
        .states
        .iter()
        .position(|state| state == piece_state)
        .map(|index| index as i32)
        .ok_or_else(|| CompilerError::InvalidPieceState(piece_state.clone()))
}

fn bfs<T, S: Ord>(
    initial_state: T,
    transform: impl Fn(&T) -> Vec<T> + Send + Sync,
    key: impl Fn(&T) -> S,
) -> Vec<S> {
    let mut visited: BTreeSet<S> = BTreeSet::new();
    let mut queue: Vec<T> = vec![initial_state];

    while let Some(state) = queue.pop() {
        let new_states = transform(&state);

        for new_state in new_states {
            let new_state_key = key(&new_state);

            if visited.insert(new_state_key) {
                queue.push(new_state);
            }
        }
    }

    visited.into_iter().collect()
}

fn compile_move<T: PieceState>(
    name: String,
    move_: PuzzleMove<T>,
    orbits: &[OrbitDefinition<T>],
    orbit_map: &[i32],
    index_piece_map: &[i32],
) -> Result<CompiledMoveDefinition, T> {
    let mut transform = vec![];

    for piece_id in index_piece_map.iter() {
        let mut row = vec![];
        let orbit = &orbits[orbit_map[*piece_id as usize] as usize];

        for compiled_piece_state in 0..orbit.states.len() {
            let piece_state = decompile_state(compiled_piece_state as i32, orbit)
                .expect("all compiled piece states from 0 to total states should be valid");
            let new_piece_state = move_(&piece_state);

            row.push(compile_state(&new_piece_state, orbit)?);
        }

        transform.push(row);
    }

    Ok(CompiledMoveDefinition { name, transform })
}

fn find_orbits<T: PieceState>(
    solved_state: &[T],
    moves: &Vec<PuzzleMove<T>>,
) -> (Vec<OrbitDefinition<T>>, Vec<i32>) {
    let mut orbit_piece_map: BTreeMap<Vec<T>, Vec<i32>> = BTreeMap::new();

    for piece_id in 0..solved_state.len() {
        let visited = bfs(
            solved_state[piece_id].clone(),
            |state| moves.iter().map(|move_| move_(state)).collect(),
            |state| state.clone(),
        );

        orbit_piece_map
            .entry(visited)
            .or_default()
            .push(piece_id as i32);
    }

    let mut orbit_definitions = vec![];
    let mut piece_orbit_map = vec![0; solved_state.len()];
    let mut beginning_index = 0;

    for (orbit_index, (states, pieces)) in orbit_piece_map.into_iter().enumerate() {
        let end_index = beginning_index + pieces.len() as i32;

        for &piece_id in pieces.iter() {
            piece_orbit_map[piece_id as usize] = orbit_index as i32;
        }

        orbit_definitions.push(OrbitDefinition {
            slice: beginning_index..end_index,
            states: states,
            pieces: pieces,
        });

        beginning_index = end_index;
    }

    (orbit_definitions, piece_orbit_map)
}

impl<T: PieceState> TryFrom<PuzzleDefinition<T>> for CompiledPuzzleDefinition<T> {
    type Error = CompilerError<T>;

    fn try_from(puzzle: PuzzleDefinition<T>) -> Result<Self, T> {
        let (orbits, piece_orbit_map) = find_orbits(
            &puzzle.solved_state,
            &puzzle.moves.values().cloned().collect(),
        );

        let index_piece_map: Vec<i32> = orbits
            .iter()
            .flat_map(|orbit| orbit.pieces.iter().copied())
            .collect();
        let mut piece_index_map = vec![0; index_piece_map.len()];

        for (piece_id, &orbit_index) in index_piece_map.iter().enumerate() {
            piece_index_map[orbit_index as usize] = piece_id as i32;
        }

        let compiled_moves = puzzle
            .moves
            .into_iter()
            .map(|(name, move_)| {
                compile_move(name, move_, &orbits, &piece_orbit_map, &piece_index_map)
            })
            .collect::<Result<Vec<CompiledMoveDefinition>, T>>()?;

        let compiled_solved_state = puzzle
            .solved_state
            .into_iter()
            .enumerate()
            .map(|(index, state)| compile_state(&state, &orbits[piece_orbit_map[index] as usize]))
            .collect::<Result<Vec<CompiledPieceState>, T>>()?;

        Ok(CompiledPuzzleDefinition {
            moves: compiled_moves,
            orbits,
            piece_orbit_map,
            piece_index_map,
            solved_state: compiled_solved_state,
        })
    }
}
