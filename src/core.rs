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
pub struct PropertyTransformDefinition {
    pub index: TransformIndex,
    pub value_map: Grid<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveDefinition {
    pub name: String,
    pub transforms: BTreeMap<String, PropertyTransformDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition {
    pub slice: [i32; 2],
    pub pieces: Vec<i32>,
    pub states: Vec<CompiledPieceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleDefinition {
    pub moves: Vec<MoveDefinition>,
    pub property_maxes: PropertyMaxes,
    pub states_map: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledMoveDefinition {
    pub name: String,
    pub transform: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPuzzleDefinition {
    pub moves: Vec<CompiledMoveDefinition>,
    pub orbits: Vec<OrbitDefinition>,
    pub piece_index_map: Vec<i32>,
    pub property_max_map: Vec<PropertyMaxes>,
}
