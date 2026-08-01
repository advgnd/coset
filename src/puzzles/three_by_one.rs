use crate::core::{Move, PieceMove, PuzzleRepr};

pub fn three_by_one() -> PuzzleRepr {
    // Hypothetical 3x3x1 cube because I'm too lazy to hardcode an actual 3x3x3
    let moves = [
        Move {
            name: "F".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
            ],
        },
        Move {
            name: "R".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
            ],
        },
        Move {
            name: "B".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
            ],
        },
        Move {
            name: "L".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
            ],
        },
        Move {
            name: "M".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
            ],
        },
        Move {
            name: "E".to_string(),
            piece_map: vec![
                PieceMove {
                    piece: 0,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 1,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 2,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 7,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 4,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 5,
                    state_map: vec![0],
                },
                PieceMove {
                    piece: 6,
                    state_map: vec![1, 0],
                },
                PieceMove {
                    piece: 3,
                    state_map: vec![0],
                },
            ],
        },
    ];

    PuzzleRepr::from_moves(8, &moves)
}
