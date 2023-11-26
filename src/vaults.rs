use bevy::prelude::*;

use crate::species::Species;

#[derive(Component, Clone)]
pub enum Vault {
    FelidolGenerator,
}


const VAULTS: &[&str] = &[
// 0. Empty Template
"
#########
#.......#
#.......#
#.......#
#.......#
#.......#
#.......#
#.......#
#########
",
// 1. Felidol Generator
"
#########
#.......#
#.F...F.#
#.......#
#.......#
#.......#
#.F...F.#
#.......#
####.####
",
];

pub fn get_build_sequence( // I am so surprised this worked on the first try. Rust magic! 25th of November 2023
    vault: Vault,
    corner: (usize, usize)
) -> Vec<(Species, (usize, usize))>{
    let vault_idx = grab_vault(vault);
    let mut str_seq = VAULTS[vault_idx];
    let binding = str_seq.replace("\n", "");
    str_seq = &binding;
    let mut output = Vec::new();
    for x in 0..9{
        for y in 0..9{
            let chara = str_seq.as_bytes()[vault_xy_idx(x, 8-y)] as char; // "8-y" because this unfortunately needs to be flipped to match the vault strings.
            let species = get_species_from_char(chara);
            if species == Species::Void{ // Don't place down "floor" creatures.
                continue;
            }
            output.push((species, (x+corner.0, y+corner.1)));
        }
    }
    output
}

pub fn vault_xy_idx(x: usize, y: usize) -> usize {
    (y * 9) + x
}

fn grab_vault(
    vault: Vault
)-> usize{
    match vault{
        Vault::FelidolGenerator => 1,
    }
}

fn get_species_from_char(
    char: char,
) -> Species {
    match char{
        '.' => Species::Void,
        'F' => Species::Felidol,
        '#' => Species::Wall,
        _ => Species::BuggedSpecies
    }
}