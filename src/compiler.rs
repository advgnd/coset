use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
};

use indexmap::IndexSet;

use crate::core::{
    CompiledMove, CompiledPieceState, CompiledPuzzleDefinition, DfaEvaluator, Move,
    OrbitDefinition, PuzzleDefinition,
};

#[derive(thiserror::Error, Debug)]
pub enum CompilerError<T> {
    #[error("cannot decode invalid compiled piece state {0}")]
    InvalidCompiledPieceState(i32),

    #[error("cannot encode invalid piece state {0:?}")]
    InvalidPieceState(T),
}

type Result<T, U> = std::result::Result<T, CompilerError<U>>;

pub fn decompile_state<T: Clone>(
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

pub fn compile_state<T: Clone + Eq>(
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

fn bfs<T: Clone + Hash + Eq>(initial_state: T, transform: impl Fn(&T) -> Vec<T>) -> IndexSet<T> {
    let mut visited = IndexSet::new();
    let mut queue = VecDeque::new();

    visited.insert(initial_state.clone());
    queue.push_back(initial_state);

    while let Some(state) = queue.pop_front() {
        let new_states = transform(&state);

        for new_state in new_states {
            if visited.insert(new_state.clone()) {
                queue.push_back(new_state);
            }
        }
    }

    visited
}

fn compile_move<T: Debug + Clone + Eq>(
    move_: &Move<T>,
    orbits: &[OrbitDefinition<T>],
    orbit_map: &[i32],
    index_piece_map: &[i32],
) -> Result<CompiledMove, T> {
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

    Ok(transform)
}

fn find_orbits<T: Clone + Eq + Hash>(
    solved_state: &[T],
    moves: &Vec<&Move<T>>,
) -> (Vec<OrbitDefinition<T>>, Vec<i32>) {
    let mut orbit_definitions = vec![];
    let mut visited_states = HashSet::new();
    let mut piece_orbit_map = vec![0; solved_state.len()];
    let mut beginning_index = 0;

    for piece_state in solved_state.iter() {
        if visited_states.contains(piece_state) {
            continue;
        }

        let visited = bfs(piece_state.clone(), |state| {
            moves.iter().map(|move_| move_(state)).collect()
        });
        let piece_ids: Vec<i32> = solved_state
            .iter()
            .enumerate()
            .filter_map(|(id, state)| {
                if visited.contains(state) {
                    Some(id as i32)
                } else {
                    None
                }
            })
            .collect();
        let end_index = beginning_index + piece_ids.len();

        for &piece_id in piece_ids.iter() {
            piece_orbit_map[piece_id as usize] = orbit_definitions.len() as i32;
        }

        visited_states.extend(visited.clone().into_iter());
        orbit_definitions.push(OrbitDefinition {
            slice: beginning_index..end_index,
            states: visited.into_iter().collect(),
            pieces: piece_ids,
        });
        beginning_index = end_index;
    }

    (orbit_definitions, piece_orbit_map)
}

fn find_dfa_masks<T: Clone + Default + Eq + Hash, U: Eq + Hash>(
    dfa_eval: &DfaEvaluator<T, U>,
    moves: &Vec<U>,
) -> (Vec<Vec<i32>>, Vec<Vec<i32>>) {
    let dfa_states = bfs(T::default(), |dfa_state| {
        moves
            .iter()
            .filter_map(|move_| dfa_eval(dfa_state, move_))
            .collect()
    })
    .into_iter()
    .collect::<IndexSet<_>>();

    let mut allowed_moves = vec![];
    let mut next_dfa_states = vec![];

    for dfa_state in dfa_states.iter() {
        let mut mask = vec![];
        let mut next_states = vec![];

        for (move_index, move_) in moves.iter().enumerate() {
            if let Some(next_dfa_state) = dfa_eval(dfa_state, move_) {
                mask.push(move_index as i32);
                next_states.push(dfa_states.get_index_of(&next_dfa_state).unwrap() as i32);
            }
        }

        allowed_moves.push(mask);
        next_dfa_states.push(next_states);
    }

    (allowed_moves, next_dfa_states)
}

impl<
    T: Debug + Clone + Eq + Hash + 'static,
    U: Clone + Eq + Hash + 'static,
    V: Clone + Default + Eq + Hash + 'static,
> TryFrom<PuzzleDefinition<T, U, V>> for CompiledPuzzleDefinition<T, U>
{
    type Error = CompilerError<T>;

    fn try_from(puzzle: PuzzleDefinition<T, U, V>) -> Result<Self, T> {
        let (orbits, piece_orbit_map) = find_orbits(
            &puzzle.solved_state,
            &puzzle.moves.values().map(|move_| move_.as_ref()).collect(),
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
            .iter()
            .map(|(_, move_)| compile_move(move_, &orbits, &piece_orbit_map, &index_piece_map))
            .collect::<Result<_, T>>()?;

        let compiled_solved_state = puzzle
            .solved_state
            .into_iter()
            .enumerate()
            .map(|(index, state)| compile_state(&state, &orbits[piece_orbit_map[index] as usize]))
            .collect::<Result<Vec<CompiledPieceState>, T>>()?;

        let reordered_compiled_solved_state = index_piece_map
            .iter()
            .map(|&index| compiled_solved_state[index as usize])
            .collect();

        let (allowed_moves, next_dfa_states) =
            find_dfa_masks(&puzzle.dfa_eval, &puzzle.moves.keys().cloned().collect());

        Ok(CompiledPuzzleDefinition {
            moves: puzzle.moves.keys().cloned().collect(),
            compiled_moves,
            orbits,
            piece_orbit_map,
            piece_index_map,
            solved_state: reordered_compiled_solved_state,
            allowed_moves,
            next_dfa_states,
        })
    }
}
