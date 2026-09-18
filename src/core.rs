use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Deref, DerefMut, Range},
};

use grid::Grid;
use serde::{Deserialize, Serialize};

type InnerPieceState = BTreeMap<String, i32>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct PieceState(InnerPieceState);

impl Deref for PieceState {
    type Target = InnerPieceState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PieceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for PieceState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PieceState {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.values().sum::<i32>().cmp(&other.values().sum::<i32>());

        if result == Ordering::Equal {
            self.0.cmp(&other.0)
        } else {
            result
        }
    }
}

impl<T> FromIterator<T> for PieceState
where
    InnerPieceState: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(value: I) -> Self {
        PieceState(value.into_iter().collect())
    }
}

impl<T, U> IntoIterator for PieceState
where
    U: Iterator<Item = T>,
    InnerPieceState: IntoIterator<Item = T, IntoIter = U>,
{
    type Item = T;
    type IntoIter = U;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub type CompiledPieceState = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveDefinition {
    pub name: String,
    pub transform: BTreeMap<PieceState, PieceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitDefinition {
    pub slice: Range<i32>,
    pub pieces: Vec<i32>,
    pub states: Vec<PieceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleDefinition {
    pub moves: Vec<MoveDefinition>,
    pub solved_state: Vec<PieceState>,
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
    pub orbit_map: Vec<i32>,
    pub piece_index_map: Vec<i32>,
    pub states_map: Vec<Vec<String>>,
    pub solved_state: Vec<CompiledPieceState>,
}
