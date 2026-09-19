use std::{
    collections::BTreeMap,
    fmt::Debug,
    ops::Range,
    sync::Arc,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use trait_set::trait_set;

trait_set! {
    pub trait PieceState = Ord + Clone + Debug + Serialize + DeserializeOwned;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub struct PieceStateStub;
pub type CompiledPieceState = i32;

pub trait PuzzleMove<T>: Fn(&T) -> T + Debug {}

impl<F, T> PuzzleMove<T> for F where F: Fn(&T) -> T + Debug {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: PieceState")]
pub struct OrbitDefinition<T: PieceState> {
    pub slice: Range<i32>,
    pub pieces: Vec<i32>,
    pub states: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct PuzzleDefinition<T: PieceState> {
    pub moves: BTreeMap<String, Arc<dyn PuzzleMove<T> + Send + Sync>>,
    pub solved_state: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledMoveDefinition {
    pub name: String,
    pub transform: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: PieceState")]
pub struct CompiledPuzzleDefinition<T: PieceState> {
    pub moves: Vec<CompiledMoveDefinition>,
    pub orbits: Vec<OrbitDefinition<T>>,
    pub orbit_map: Vec<i32>,
    pub piece_index_map: Vec<i32>,
    pub solved_state: Vec<CompiledPieceState>,
}
