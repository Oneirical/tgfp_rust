use bevy::prelude::*;

use crate::species::Species;

#[derive(Component, Clone)]
pub enum Vault {
    FelidolGenerator,
    EpicWow,
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
// 2. DCSS Plagiarized Vault
"
TTTT.....TTT......TTTTTTT......TTT.....TTTT
TTTTTTTTTT......TTT.....TTT......TTTTTTTTTT
.T...TTT......TTT.........TTT......TTT...T.
.T...T......TTT.............TTT......T...T.
.T...T....TTT.................TTT....T...T.
TTTTTTTTTTT.....................TTTTTTTTTTT
TT...T.............TTTTT.............T...TT
T....T............TT$P$TT............T....T
...TTT............T.TTT.T............TTT...
..TT..............T.TTT.T..............TT..
.TT...............T.....T...............TT.
TT.........TT.....T.....T.....TT.........TT
T.........TTTT....TT...TT....TTTT.........T
T.........TTTT.....T...T.....TTTT.........T
T..........TT......T...T......TT..........T
T.........................................T
T.........................................T
T.........................................T
T......TTTTTT.....TT...TT.....TTTTTT......T
T.....TT....TTT...T.nnn.T...TTT....TT.....T
T.....T$TT.........w...e.........TT$T.....T
T.....TPTT.........w...e.........TTPT.....T
T.....T$TT.........w...e.........TT$T.....T
T.....TT....TTT...T.sss.T...TTT....TT.....T
T......TTTTTT.....TT...TT.....TTTTTT......T
T.........................................T
T.........................................T
T.........................................T
T..................T...T..................T
T..........TT......T...T......TT..........T
T.........TTTT....TT...TT....TTTT.........T
TT........TTTT....T.....T....TTTT........TT
.TT........TT.....T.....T.....TT........TT.
..TT..............T.T.T.T..............TT..
...TTT............T.T.T.T............TTT...
T....T............TTT.TTT............T....T
TT...T.............TTTTT.............T...TT
TTTTTTTTTTT.....................TTTTTTTTTTT
.T...T....TTT.................TTT....T...T.
.T...T......TTT.............TTT......T...T.
.T...TTT......TTT.........TTT......TTT...T.
TTTTTTTTTT......TTT.....TTT......TTTTTTTTTT
TTTT.....TTT......TTTTTTT......TTT.....TTTT
"
];

pub fn get_build_sequence( // I am so surprised this worked on the first try. Rust magic! 25th of November 2023
    vault: Vault,
    corner: (usize, usize)
) -> Vec<(Species, (usize, usize))>{
    let vault_idx = grab_vault(vault);
    let mut str_seq = VAULTS[vault_idx];
    let binding = str_seq.replace('\n', "");
    str_seq = &binding;
    let length = (str_seq.len() as f32).sqrt() as usize;
    let mut output = Vec::with_capacity(str_seq.len());
    for x in 0..length{
        for y in 0..length{
            let chara = str_seq.as_bytes()[vault_xy_idx(x, length-1-y, length)] as char; // "length-1-y" because this unfortunately needs to be flipped to match the vault strings.
            let species = get_species_from_char(chara);
            if species == Species::Void{ // Don't place down "floor" creatures.
                continue;
            }
            output.push((species, (x+corner.0, y+corner.1)));
        }
    }
    output
}

pub fn vault_xy_idx(x: usize, y: usize, size: usize) -> usize {
    (y * size) + x
}

fn grab_vault(
    vault: Vault
)-> usize{
    match vault{
        Vault::FelidolGenerator => 1,
        Vault::EpicWow => 2,
    }
}

fn get_species_from_char(
    char: char,
) -> Species {
    match char{
        '.' => Species::Void,
        'F' => Species::Felidol,
        '#' => Species::Wall,
        'T' => Species::TermiWall,
        'n' => Species::HypnoWell { dir: 0 },
        'e' => Species::HypnoWell { dir: 3 },
        's' => Species::HypnoWell { dir: 2 },
        'w' => Species::HypnoWell { dir: 1 },
        _ => Species::BuggedSpecies
    }
}