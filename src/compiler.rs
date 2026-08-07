use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::{
    CompiledMoveDefinition, CompiledPieceState, CompiledPuzzleDefinition, MoveDefinition,
    OrbitDefinition, PieceState, PropertyMaxes, PuzzleDefinition, TransformIndex,
};

fn compute_compiled_totals(property_maxes: &PropertyMaxes) -> BTreeMap<String, i32> {
    let mut totals = BTreeMap::new();
    let mut total = 1;

    for (property, max) in property_maxes.iter().rev() {
        totals.insert(property.clone(), total);
        total *= max;
    }

    totals
}

fn decode_compiled_state(
    mut state: CompiledPieceState,
    property_maxes: &PropertyMaxes,
) -> PieceState {
    let mut piece_state = PieceState::new();

    for (property, total) in compute_compiled_totals(property_maxes).iter() {
        piece_state.insert(property.clone(), state / total);
        state %= total;
    }

    piece_state
}

fn encode_compiled_state(
    piece_state: &PieceState,
    property_maxes: &PropertyMaxes,
) -> CompiledPieceState {
    let mut state = 0;

    for (property, total) in compute_compiled_totals(property_maxes).iter() {
        state += piece_state[property] * total;
    }

    state
}

fn filter_property_maxes(
    property_maxes: &PropertyMaxes,
    piece_properties: &[String],
) -> PropertyMaxes {
    property_maxes
        .iter()
        .filter(|(property, _)| piece_properties.contains(property))
        .map(|(key, value)| (key.clone(), *value)) // Dereference because filter is referencing for whatever reason
        .collect::<BTreeMap<_, _>>()
}

fn compile_move(
    move_: MoveDefinition,
    property_maxes: &PropertyMaxes,
    states_map: &[Vec<String>],
) -> CompiledMoveDefinition {
    let mut transform = vec![];

    for piece_properties in states_map {
        let mut row = vec![];
        let piece_property_maxes = filter_property_maxes(property_maxes, piece_properties);

        for piece_id in 0..piece_property_maxes.values().product() {
            let piece_state = decode_compiled_state(piece_id as i32, &piece_property_maxes);
            let piece_state = piece_state
                .iter()
                .map(|(property, value)| {
                    let property_transform = &move_.transforms[property];
                    match &property_transform.index {
                        TransformIndex::PieceId => (
                            property.clone(),
                            property_transform.value_map[(piece_id as usize, *value as usize)],
                        ),
                        TransformIndex::Property(indexed_property) => (
                            property.clone(),
                            property_transform.value_map
                                [(piece_state[indexed_property] as usize, *value as usize)],
                        ),
                    }
                })
                .collect::<PieceState>();

            row.push(encode_compiled_state(&piece_state, property_maxes));
        }

        transform.push(row);
    }

    CompiledMoveDefinition {
        name: move_.name,
        transform,
    }
}

fn find_orbits(
    state_len: usize,
    moves: &[CompiledMoveDefinition],
) -> BTreeMap<BTreeSet<i32>, BTreeSet<i32>> {
    let mut orbit_map: BTreeMap<BTreeSet<i32>, BTreeSet<i32>> = BTreeMap::new();

    for piece_id in 0..state_len {
        let mut new_states: BTreeSet<i32> = BTreeSet::from_iter(vec![0]);
        let mut visited: BTreeSet<i32> = BTreeSet::from_iter(vec![]);

        while !visited.is_superset(&new_states) {
            visited.extend(new_states.iter());

            for move_ in moves.iter() {
                new_states = new_states
                    .into_iter()
                    .map(|state| move_.transform[piece_id][state as usize])
                    .collect::<BTreeSet<i32>>();
            }
        }

        orbit_map
            .entry(visited)
            .or_default()
            .insert(piece_id as i32);
    }

    orbit_map
}

impl From<PuzzleDefinition> for CompiledPuzzleDefinition {
    fn from(puzzle: PuzzleDefinition) -> Self {
        let compiled_moves: Vec<CompiledMoveDefinition> = puzzle
            .moves
            .into_iter()
            .map(|move_| compile_move(move_, &puzzle.property_maxes, &puzzle.states_map))
            .collect();
        let orbits = find_orbits(puzzle.states_map.len(), &compiled_moves);
        let index_piece_map: Vec<i32> = orbits.values().flatten().copied().collect();

        let compiled_moves = compiled_moves
            .into_iter()
            .map(|move_| CompiledMoveDefinition {
                name: move_.name,
                transform: move_
                    .transform
                    .iter()
                    .map(|transformation| {
                        index_piece_map
                            .iter()
                            .map(|&piece_id| &transformation[piece_id as usize])
                            .copied()
                            .collect()
                    })
                    .collect(),
            })
            .collect();

        let mut beginning_index = 0;
        let mut orbit_definitions: Vec<OrbitDefinition> = Vec::with_capacity(orbits.len());

        for (orbit, pieces) in orbits.into_iter() {
            let ending_index = beginning_index + pieces.len();

            orbit_definitions.push(OrbitDefinition {
                slice: [beginning_index as i32, ending_index as i32],
                pieces: pieces.into_iter().collect(),
                states: orbit.into_iter().collect(),
            });

            beginning_index = ending_index;
        }

        CompiledPuzzleDefinition {
            moves: compiled_moves,
            orbits: orbit_definitions,
            piece_index_map: index_piece_map,
            property_max_map: puzzle
                .states_map
                .into_iter()
                .map(|piece_properties| {
                    filter_property_maxes(&puzzle.property_maxes, &piece_properties)
                })
                .collect(),
        }
    }
}
