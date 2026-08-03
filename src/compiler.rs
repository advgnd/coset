use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::{
    CompiledMove, CompiledPieceState, CompiledPuzzle, Move, Orbit, PieceState, PropertyMaxes,
    Puzzle, TransformIndex,
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
    move_: Move,
    property_maxes: &PropertyMaxes,
    states_map: &[Vec<String>],
) -> CompiledMove {
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

    CompiledMove {
        name: move_.name,
        transform,
    }
}

fn find_orbits(state_len: usize, moves: &[CompiledMove]) -> Vec<Orbit> {
    let mut piece_state_map: HashMap<BTreeSet<i32>, Vec<i32>> = HashMap::new();

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

        piece_state_map
            .entry(visited)
            .or_default()
            .push(piece_id as i32);
    }

    piece_state_map
        .into_iter()
        .map(|(visited, pieces)| Orbit {
            pieces,
            max_composite_state: visited.into_iter().max().unwrap_or(0),
        })
        .collect()
}

impl From<Puzzle> for CompiledPuzzle {
    fn from(puzzle: Puzzle) -> Self {
        let compiled_moves: Vec<CompiledMove> = puzzle
            .moves
            .into_iter()
            .map(|move_| compile_move(move_, &puzzle.property_maxes, &puzzle.states_map))
            .collect();
        let orbits = find_orbits(puzzle.states_map.len(), &compiled_moves);

        CompiledPuzzle {
            moves: compiled_moves,
            orbits,
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
