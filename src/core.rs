use std::collections::BTreeMap;

use grid::Grid;
use serde::{Deserialize, Serialize};

pub type PropertyMaxes = BTreeMap<String, i32>; // i32 represents the max value of the property
pub type PieceState = BTreeMap<String, i32>; // i32 represents the current value of the property
pub type CompiledPieceState = i32; // compound state derived from the traditional piece state

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformIndex {
    PieceId,
    Property(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyTransform {
    pub index: TransformIndex,
    pub value_map: Grid<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub name: String,
    pub transforms: BTreeMap<String, PropertyTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orbit {
    pub pieces: Vec<i32>,
    pub max_composite_state: CompiledPieceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub moves: Vec<Move>,
    pub property_maxes: PropertyMaxes,
    pub states_map: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledMove {
    pub name: String,
    pub transform: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPuzzle {
    pub moves: Vec<CompiledMove>,
    pub orbits: Vec<Orbit>,
    pub property_max_map: Vec<PropertyMaxes>,
}
