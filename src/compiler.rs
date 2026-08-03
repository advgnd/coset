use std::collections::BTreeMap;

use disjoint::DisjointSet;

use crate::core::{
    CompiledMove, CompiledPuzzle, Move, Orbit, PieceState, PropertyMaxes, Puzzle, TransformIndex,
};

fn compute_compound_totals(property_maxes: &PropertyMaxes) -> BTreeMap<String, i32> {
    let mut totals = BTreeMap::new();
    let mut total = 1;

    for (property, max) in property_maxes.iter().rev() {
        totals.insert(property.clone(), total);
        total *= max;
    }

    totals
}

fn decode_compound_state(mut state: i32, property_maxes: &PropertyMaxes) -> PieceState {
    let mut piece_state = PieceState::new();

    for (property, total) in compute_compound_totals(property_maxes).iter() {
        piece_state.insert(property.clone(), state / total);
        state %= total;
    }

    piece_state
}

fn encode_compound_state(piece_state: &PieceState, property_maxes: &PropertyMaxes) -> i32 {
    let mut state = 0;

    for (property, total) in compute_compound_totals(property_maxes).iter() {
        state += piece_state[property] * total;
    }

    state
}

fn compile_move(move_: Move, property_maxes: &PropertyMaxes, state: &[PieceState]) -> CompiledMove {
    let mut transform = vec![];

    for piece_state in state.iter() {
        let mut row = vec![];
        let piece_properties = piece_state.keys().collect::<Vec<_>>();
        let piece_property_maxes = property_maxes
            .iter()
            .filter(|(property, _)| piece_properties.contains(property))
            .map(|(key, value)| (key.clone(), *value)) // Dereference because filter is referencing for whatever reason
            .collect::<BTreeMap<_, _>>();

        for piece_id in 0..piece_property_maxes.values().product() {
            let piece_state = decode_compound_state(piece_id as i32, &piece_property_maxes);
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

            row.push(encode_compound_state(&piece_state, property_maxes));
        }

        transform.push(row);
    }

    CompiledMove {
        name: move_.name,
        transform,
    }
}
