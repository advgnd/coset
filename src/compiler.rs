use std::collections::{BTreeMap, BTreeSet};

use crate::core::{
    CompiledMoveDefinition, CompiledPieceState, CompiledPuzzleDefinition, MoveDefinition,
    OrbitDefinition, PieceState, PuzzleDefinition,
};

#[derive(thiserror::Error, Debug)]
pub enum CompilerError {
    #[error("cannot decode invalid compiled piece state {0}")]
    InvalidCompiledPieceState(i32),

    #[error("cannot encode invalid piece state {0:?}")]
    InvalidPieceState(PieceState),
}

type Result<T> = std::result::Result<T, CompilerError>;

fn transform_state(
    piece_state: PieceState,
    move_: &MoveDefinition,
) -> PieceState {
    let mut new_piece_state = piece_state.clone();

    for (from, to) in move_.transform.iter() {
        let state_matches = from
            .iter()
            .all(|(property, select_value)| piece_state.get(property).map_or(true, |value| value == select_value));

        if state_matches {
            new_piece_state.extend(to.clone());
        }
    }
    
    new_piece_state
}

pub fn decompile_state(
    compiled_piece_state: CompiledPieceState,
    orbit: &OrbitDefinition,
) -> Result<PieceState> {
    orbit
        .states
        .get(compiled_piece_state as usize)
        .ok_or(CompilerError::InvalidCompiledPieceState(
            compiled_piece_state,
        ))
        .cloned()
}

pub fn compile_state(
    piece_state: &PieceState,
    orbit: &OrbitDefinition,
) -> Result<CompiledPieceState> {
    orbit
        .states
        .iter()
        .position(|state| state == piece_state)
        .map(|index| index as i32)
        .ok_or_else(|| CompilerError::InvalidPieceState(piece_state.clone()))
}

fn compile_move(
    move_: MoveDefinition,
    orbits: &[OrbitDefinition],
    orbit_map: &[i32],
    index_piece_map: &[i32],
) -> Result<CompiledMoveDefinition> {
    let mut transform = vec![];

    for piece_id in index_piece_map.iter() {
        let mut row = vec![];
        let orbit = &orbits[orbit_map[*piece_id as usize] as usize];

        for compiled_piece_state in 0..orbit.states.len() {
            let piece_state = decompile_state(compiled_piece_state as i32, orbit)
                .expect("all compiled piece states from 0 to total states should be valid");
            let new_piece_state = transform_state(piece_state, &move_);

            row.push(compile_state(&new_piece_state, orbit)?);
        }

        transform.push(row);
    }

    Ok(CompiledMoveDefinition {
        name: move_.name,
        transform,
    })
}

fn find_orbits(
    solved_state: &[PieceState],
    moves: &[MoveDefinition],
) -> Result<Vec<OrbitDefinition>> {
    let mut orbit_map: BTreeMap<BTreeSet<PieceState>, BTreeSet<i32>> = BTreeMap::new();

    for piece_id in 0..solved_state.len() {
        let initial_piece_state = solved_state[piece_id].clone();

        let mut new_piece_states: BTreeSet<PieceState> =
            BTreeSet::from_iter(vec![initial_piece_state]);
        let mut visited: BTreeSet<PieceState> = BTreeSet::from_iter(vec![]);

        while !visited.is_superset(&new_piece_states) {
            let old_piece_states = new_piece_states;
            new_piece_states = BTreeSet::new();

            visited.extend(old_piece_states.iter().cloned());

            for move_ in moves.iter() {
                // For lack of a better variable name, I present you:
                let new_new_piece_states = old_piece_states
                    .iter()
                    .map(|state| transform_state(state.clone(), move_))
                    .collect::<Vec<PieceState>>();

                new_piece_states.extend(new_new_piece_states);
            }
        }

        orbit_map
            .entry(visited)
            .or_default()
            .insert(piece_id as i32);
    }

    let mut orbit_definitions = vec![];
    let mut beginning_index = 0;

    for (states, pieces) in orbit_map.into_iter() {
        let end_index = beginning_index + pieces.len() as i32;

        orbit_definitions.push(OrbitDefinition {
            slice: beginning_index..end_index,
            states: states.into_iter().collect(),
            pieces: pieces.into_iter().collect(),
        });

        beginning_index = end_index;
    }

    Ok(orbit_definitions)
}

impl TryFrom<PuzzleDefinition> for CompiledPuzzleDefinition {
    type Error = CompilerError;

    fn try_from(puzzle: PuzzleDefinition) -> Result<Self> {
        let states_map = puzzle
            .solved_state
            .iter()
            .map(|state| state.keys().cloned().collect())
            .collect::<Vec<_>>();

        let orbits = find_orbits(&puzzle.solved_state, &puzzle.moves)?;
        let mut orbit_map = vec![];

        for piece_id in 0..puzzle.solved_state.len() {
            orbit_map.push(
                orbits
                    .iter()
                    .position(|orbit| orbit.pieces.contains(&(piece_id as i32)))
                    .expect("all piece IDs should be found in an orbit") as i32,
            );
        }

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
            .map(|move_| compile_move(move_, &orbits, &orbit_map, &piece_index_map))
            .collect::<Result<Vec<CompiledMoveDefinition>>>()?;

        let compiled_solved_state = puzzle
            .solved_state
            .into_iter()
            .enumerate()
            .map(|(index, state)| compile_state(&state, &orbits[orbit_map[index] as usize]))
            .collect::<Result<Vec<CompiledPieceState>>>()?;

        Ok(CompiledPuzzleDefinition {
            moves: compiled_moves,
            orbits,
            orbit_map,
            piece_index_map,
            states_map,
            solved_state: compiled_solved_state,
        })
    }
}
