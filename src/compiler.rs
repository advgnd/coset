use disjoint::DisjointSet;

use crate::core::{Move, Orbit, PuzzleRepr};

fn find_orbits(puzzle_state_size: usize, moves: &[Move]) -> Vec<Orbit> {
    let mut orbits = DisjointSet::with_len(puzzle_state_size);
    for move_ in moves {
        for (i, piece_move) in move_.piece_map.iter().enumerate() {
            let current_state_size = piece_move.state_map.len();
            let new_state_size = move_.piece_map[piece_move.piece as usize].state_map.len();

            if current_state_size == new_state_size {
                orbits.join(i, piece_move.piece as usize);
            }
        }
    }

    orbits
        .sets()
        .iter()
        .map(|orbit| Orbit {
            piece_ids: orbit.iter().map(|i| *i as i32).collect(),
            state_size: moves[0].piece_map[orbit[0]].state_map.len(), // Assumes the data is formatted properly
        })
        .collect()
}

impl PuzzleRepr {
    pub fn from_moves(state_size: usize, moves: &[Move]) -> Self {
        let orbits = find_orbits(state_size, moves);
        Self {
            moves: moves.to_vec(),
            orbits,
            state_size,
        }
    }
}
