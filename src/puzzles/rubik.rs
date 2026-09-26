use std::collections::HashMap;

use strum::Display;

use crate::core::{Move, PuzzleDefinition};

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Hash)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3d {
    y: i32,
    z: i32,
    x: i32,
}

impl Vec3d {
    fn coord(&self, axis: Axis) -> i32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    fn rotate(self, axis: Axis, clockwise: bool) -> Self {
        match axis {
            Axis::X => {
                if clockwise {
                    Self {
                        x: self.x,
                        y: -self.z,
                        z: self.y,
                    }
                } else {
                    Self {
                        x: self.x,
                        y: self.z,
                        z: -self.y,
                    }
                }
            }
            Axis::Y => {
                if clockwise {
                    Self {
                        x: -self.z,
                        y: self.y,
                        z: self.x,
                    }
                } else {
                    Self {
                        x: self.z,
                        y: self.y,
                        z: -self.x,
                    }
                }
            }
            Axis::Z => {
                if clockwise {
                    Self {
                        x: -self.y,
                        y: self.x,
                        z: self.z,
                    }
                } else {
                    Self {
                        x: self.y,
                        y: -self.x,
                        z: self.z,
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cubie {
    position: Vec3d,
    orientation: Vec3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveRotations {
    Clockwise,
    Half,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveId {
    axis: Axis,
    layer: i32,
    rotation: MoveRotations,
}

#[derive(Debug, Clone, Default)]
pub struct DfaState {
    current_axis: Option<Axis>,
    turned_layers: Vec<i32>,
}

impl Cubie {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            position: Vec3d { x, y, z },
            orientation: Vec3d { x: 0, y: 0, z: 1 },
        }
    }

    fn outer_score(&self, size: usize) -> usize {
        let half_size = (size / 2) as i32;

        [self.position.x, self.position.y, self.position.z]
            .iter()
            .filter(|&x| *x == -half_size || *x == half_size as i32)
            .count()
    }

    fn rotate(&self, axis: Axis, clockwise: bool) -> Self {
        Self {
            position: self.position.rotate(axis, clockwise),
            orientation: self.orientation.rotate(axis, clockwise),
        }
    }
}

fn dfa_eval(dfa_state: &DfaState, move_id: &MoveId) -> Option<DfaState> {
    if dfa_state.current_axis == Some(move_id.axis) {
        if dfa_state.turned_layers.contains(&move_id.layer)
            || move_id.layer < *dfa_state.turned_layers.iter().max().unwrap_or(&-1)
        {
            None
        } else {
            let mut new_dfa_state = dfa_state.clone();
            new_dfa_state.turned_layers.push(move_id.layer);
            Some(new_dfa_state)
        }
    } else {
        Some(DfaState {
            current_axis: Some(move_id.axis),
            turned_layers: vec![move_id.layer],
        })
    }
}

pub fn rubik(size: usize) -> PuzzleDefinition<Cubie, MoveId, DfaState> {
    let half_size = (size / 2) as i32;
    let include_zero = size % 2 != 0;
    let dim_range = (-half_size..=half_size).filter(|n| include_zero || *n != 0);

    let mut solved_state = vec![];

    for z in dim_range.clone() {
        for y in dim_range.clone() {
            for x in dim_range.clone() {
                let cubie = Cubie::new(x as i32, y as i32, z as i32);
                let outer_score = cubie.outer_score(size);

                if outer_score > 0 {
                    solved_state.push(cubie);
                }
            }
        }
    }

    let mut moves: HashMap<MoveId, Box<Move<Cubie>>> = HashMap::new();

    for dimension in [Axis::X, Axis::Y, Axis::Z] {
        for dim_index in dim_range.clone() {
            for rotation in [
                MoveRotations::Clockwise,
                MoveRotations::Half,
                MoveRotations::CounterClockwise,
            ] {
                let move_ = move |pos: &Cubie| -> Cubie {
                    if pos.position.coord(dimension) == dim_index {
                        match rotation {
                            MoveRotations::Clockwise => pos.rotate(dimension, true),
                            MoveRotations::Half => {
                                pos.rotate(dimension, true).rotate(dimension, true)
                            }
                            MoveRotations::CounterClockwise => pos.rotate(dimension, false),
                        }
                    } else {
                        *pos
                    }
                };

                moves.insert(
                    MoveId {
                        axis: dimension,
                        layer: dim_index,
                        rotation,
                    },
                    Box::new(move_),
                );
            }
        }
    }

    PuzzleDefinition {
        moves,
        solved_state,
        dfa_eval: Box::new(dfa_eval),
    }
}
