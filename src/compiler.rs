use std::collections::{BTreeMap, BTreeSet};

use crate::core::{
    CompiledMoveDefinition, CompiledPieceState, CompiledPuzzleDefinition, MoveDefinition,
    OrbitDefinition, PieceState, PuzzleDefinition,
    TransformIndex::{PieceId, Property},
};

#[derive(thiserror::Error, Debug)]
pub enum CompilerError {
    #[error(
        "transformation of {property} in move '{move_name}' is indexed by {name}, which is not present in the state of piece #{piece_id}"
    )]
    TransformationIndexNotFound {
        property: String,
        move_name: String,
        name: String,
        piece_id: i32,
    },

    #[error("cannot decode invalid compiled piece state {0}")]
    InvalidCompiledPieceState(i32),

    #[error("cannot encode invalid piece state {0:?}")]
    InvalidPieceState(PieceState),
}

type Result<T> = std::result::Result<T, CompilerError>;

fn transform_state(
    piece_id: i32,
    piece_state: PieceState,
    move_: &MoveDefinition,
) -> Result<PieceState> {
    let new_state = piece_state
        .iter()
        .map(|(property, value)| {
            let property_transform;

            if let Some(transform) = move_.transforms.get(property) {
                property_transform = transform;
            } else {
                return Ok((property.clone(), *value));
            };

            let index = match &property_transform.index_type {
                PieceId => piece_id as usize,
                Property(name) => *piece_state.get(name).ok_or_else(|| {
                    CompilerError::TransformationIndexNotFound {
                        property: property.clone(),
                        move_name: move_.name.clone(),
                        name: name.clone(),
                        piece_id,
                    }
                })? as usize,
            };

            Ok((
                property.clone(),
                property_transform.value_map[(index, *value as usize)],
            ))
        })
        .collect();

    new_state
}

fn decode_compiled_state(
    compiled_piece_state: CompiledPieceState,
    orbit: &OrbitDefinition,
) -> Result<PieceState> {
    orbit
        .states
        .get(compiled_piece_state as usize)
        .ok_or_else(|| CompilerError::InvalidCompiledPieceState(compiled_piece_state))
        .cloned()
}

fn encode_compiled_state(
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
            let piece_state = decode_compiled_state(compiled_piece_state as i32, orbit)
                .expect("all compiled piece states from 0 to total states should be valid");
            let new_piece_state = transform_state(*piece_id, piece_state, &move_)?;

            row.push(encode_compiled_state(&new_piece_state, orbit)?);
        }

        transform.push(row);
    }

    Ok(CompiledMoveDefinition {
        name: move_.name,
        transform,
    })
}

fn find_orbits(
    state_map: &[Vec<String>],
    moves: &[MoveDefinition],
) -> Result<Vec<OrbitDefinition>> {
    let mut orbit_map: BTreeMap<BTreeSet<PieceState>, BTreeSet<i32>> = BTreeMap::new();

    for piece_id in 0..state_map.len() {
        let mut initial_piece_state = PieceState::default();

        for property in state_map[piece_id].iter() {
            initial_piece_state.insert(property.clone(), 0);
        }

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
                    .map(|state| transform_state(piece_id as i32, state.clone(), move_))
                    .collect::<Result<Vec<PieceState>>>()?;

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
        let orbits = find_orbits(&puzzle.states_map, &puzzle.moves)?;
        let mut orbit_map = vec![];

        for piece_id in 0..puzzle.states_map.len() {
            orbit_map.push(
                orbits
                    .iter()
                    .position(|orbit| orbit.pieces.contains(&(piece_id as i32)))
                    .expect("all piece IDs should be found in an orbit") as i32,
            );
        }

        let index_piece_map: Vec<i32> = orbits
            .iter()
            .map(|orbit| orbit.pieces.iter().copied())
            .flatten()
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

        Ok(CompiledPuzzleDefinition {
            moves: compiled_moves,
            orbits,
            orbit_map,
            piece_index_map,
        })
    }
}
