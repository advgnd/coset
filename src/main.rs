use std::{fs::File, io::Write};

use coset::{core::CompiledPuzzleDefinition, puzzles::rubik};

fn main() {
    let puzzle = CompiledPuzzleDefinition::try_from(rubik(4)).unwrap();

    println!("Compiled Rubik's Cube puzzle successfully");

    let bytes = serde_json::to_vec(&puzzle).unwrap();

    println!("Serialized Rubik's Cube puzzle successfully");

    let mut file = File::create("puzzle.json").unwrap();
    file.write_all(&bytes).unwrap();

    println!("Compiled Rubik's Cube puzzle successfully written to puzzle.json");
}
