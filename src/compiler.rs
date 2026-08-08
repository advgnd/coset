use std::collections::{BTreeMap, BTreeSet};

use crate::core::{
    CompiledMoveDefinition, CompiledPieceState, CompiledPuzzleDefinition, MoveDefinition,
    OrbitDefinition, PieceState, PuzzleDefinition,
    TransformIndex::{PieceId, Property},
};

fn transform_state(piece_id: i32, piece_state: PieceState, move_: &MoveDefinition) -> PieceState {
    piece_state
        .iter()
        .map(|(property, value)| {
            let property_transform;

            if let Some(transform) = move_.transforms.get(property) {
                property_transform = transform;
            } else {
                return (property.clone(), *value);
            };

            let index = match &property_transform.index_type {
                PieceId => piece_id as usize,
                Property(name) => *piece_state.get(name).expect(&format!(
                    "Transformation of {} in move {} is indexed by {}, which is not present in the state of piece #{}",
                    property, move_.name, name, piece_id
                )) as usize,
            };

            (
                property.clone(),
                property_transform.value_map[(index, *value as usize)],
            )
        })
        .collect()
}

fn decode_compiled_state(
    compiled_piece_state: CompiledPieceState,
    orbit: &OrbitDefinition,
) -> PieceState {
    orbit
        .states
        .get(compiled_piece_state as usize)
        .expect(&format!(
            "Cannot decode invalid compiled piece state {}",
            compiled_piece_state
        ))
        .clone()
}

fn encode_compiled_state(piece_state: &PieceState, orbit: &OrbitDefinition) -> CompiledPieceState {
    orbit
        .states
        .iter()
        .position(|state| state == piece_state)
        .expect(&format!(
            "Cannot encode invalid piece state {:?}",
            piece_state
        )) as i32
}

fn compile_move(
    move_: MoveDefinition,
    orbits: &[OrbitDefinition],
    orbit_map: &[i32],
    index_piece_map: &[i32],
) -> CompiledMoveDefinition {
    let mut transform = vec![];

    for piece_id in index_piece_map.iter() {
        let mut row = vec![];
        let orbit = &orbits[orbit_map[*piece_id as usize] as usize];

        for compiled_piece_state in 0..orbit.states.len() {
            let piece_state = decode_compiled_state(compiled_piece_state as i32, orbit);
            let new_piece_state = transform_state(*piece_id, piece_state, &move_);

            row.push(encode_compiled_state(&new_piece_state, orbit));
        }

        transform.push(row);
    }

    CompiledMoveDefinition {
        name: move_.name,
        transform,
    }
}

fn find_orbits(state_map: &[Vec<String>], moves: &[MoveDefinition]) -> Vec<OrbitDefinition> {
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
                    .map(|state| transform_state(piece_id as i32, state.clone(), move_));

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

    orbit_definitions
}

impl From<PuzzleDefinition> for CompiledPuzzleDefinition {
    fn from(puzzle: PuzzleDefinition) -> Self {
        let orbits = find_orbits(&puzzle.states_map, &puzzle.moves);
        let mut orbit_map = vec![];

        for piece_id in 0..puzzle.states_map.len() {
            orbit_map.push(
                orbits
                    .iter()
                    .position(|orbit| orbit.pieces.contains(&(piece_id as i32)))
                    .expect(&format!("Piece ID {} not found in any orbits", piece_id))
                    as i32,
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
            .collect();

        CompiledPuzzleDefinition {
            moves: compiled_moves,
            orbits,
            orbit_map,
            piece_index_map,
        }
    }
}
