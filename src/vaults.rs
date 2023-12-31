use bevy::prelude::*;

use crate::species::Species;

#[derive(Component, Clone)]
pub enum Vault {
    EpicWow,
    Epsilon,
    Factory,
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
// 1. DCSS Plagiarized Vault
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
T................TT.....TT................T
T......TTTTTT....T.......T....TTTTTT......T
T.....TT....TTT.............TTT....TT.....T
T.....T$TT.......................TT$T.....T
T.....TPT......G..D..F..R..P......TPT.....T
T.....T$TT.......................TT$T.....T
T.....TT....TTT.............TTT....TT.....T
T......TTTTTT....T.......T....TTTTTT......T
T................TT.....TT................T
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
",
// 2. Round Epsilon
"
#####################...#####################
#####################...#####################
#####################...#####################
#####################...#####################
#####################...#####################
##################.........##################
#######...######.............######...#######
######.....###.................###.....######
######.....#..........E..........#.....######
######.................................######
#######...............................#######
#########...........................#########
########.............................########
########.............................########
#######...............................#######
#######...............................#######
######.................................######
######................T................######
#####...................................#####
#####...................................#####
#####...................................#####
.....................T.T.....................
.................T.........T.................
.....................T.T.....................
#####...................................#####
#####...................................#####
#####...................................#####
######................T................######
######.................................######
#######...............................#######
#######...............................#######
########.............................########
########.............................########
#########............................########
#######...............A...............#######
######.................................######
######.....#.........................*.######
######.....###.................###.....######
#######...#####..............######...#######
##################.........##################
#####################...#####################
#####################mfm#####################
#####################fmf#####################
#####################mfm#####################
#####################...#####################
",
];

pub fn get_build_sequence( // I am so surprised this worked on the first try. Rust magic! 25th of November 2023
    vault: Vault,
    corner: (usize, usize)
) -> Vec<(Species, (usize, usize))>{
    let vault_idx = grab_vault(vault);
    let mut str_seq = VAULTS[vault_idx];
    let width = str_seq.split('\n').nth(1).unwrap_or(&"").len();
    let height = str_seq.matches("\n").count()-1;
    let binding = str_seq.replace('\n', "");
    str_seq = &binding;
    //let length = (str_seq.len() as f32).sqrt() as usize;
    let mut output = Vec::with_capacity(str_seq.len());
    for x in 0..width{
        for y in 0..height{
            let chara = str_seq.as_bytes()[vault_xy_idx(x, height-1-y, width)] as char; // "length-1-y" because this unfortunately needs to be flipped to match the vault strings.
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
        Vault::EpicWow => 1,
        Vault::Epsilon => 2,
        Vault::Factory => 3,
    }
}

pub fn match_vault_with_spawn_loc(
    vault: Vault
) -> (usize,usize){
    match vault {
        Vault::EpicWow => (22,8),
        Vault::Epsilon => (22,8),
        Vault::Factory => (10,10),
    }
}

fn get_species_from_char(
    char: char,
) -> Species {
    match char{
        '.' => Species::Void,
        '>' => Species::Airlock { dir: 1 },
        'V' => Species::Airlock { dir: 0 },
        '<' => Species::Airlock { dir: 3 },
        '^' => Species::Airlock { dir: 2 },
        'F' => Species::Harmonizer,
        'G' => Species::GlamourCrate,
        'D' => Species::DisciplineCrate,
        'R' => Species::GraceCrate,
        '+' => Species::PrideCrate,
        '#' => Species::Wall,
        'T' => Species::TermiWall,
        'n' => Species::RiftBorder { dir: 0 },
        'e' => Species::RiftBorder { dir: 3 },
        's' => Species::RiftBorder { dir: 2 },
        'w' => Species::RiftBorder { dir: 1 },
        'E' => Species::EpsilonHead { len: 0 },
        'm' => Species::LunaMoth,
        'f' => Species::Felidol,
        'C' => Species::EpsilonTail {order: 0},
        'P' => Species::AxiomCrate,
        'A' => Species::ChromeNurse,
        '*' => Species::SegmentTransformer,
        _ => Species::BuggedSpecies
    }
}

pub fn extract_square(vault: Vault, x: usize, y: usize) -> Vec<Vec<Species>> {
    let range = 19;
    let map = string_to_2d_vec(VAULTS[grab_vault(vault)]);
    let mut square = Vec::new();
    for i in (y as i32-range)..=(y as i32+range) {
        let mut row = Vec::new();
        for j in (x as i32-range)..=(x as i32 +range) {
            if i < map.len() as i32 && i >= 0 {
                if j >= 0 && j < map[i as usize].len() as i32 {row.push(get_species_from_char(map[i as usize][j as usize]));}
            } else {
                row.push(get_species_from_char('.'));
            }
        }
        square.push(row);
    }
    square
 }
 
 fn string_to_2d_vec(s: &str) -> Vec<Vec<char>> {
    s.lines()
     .map(|line| line.chars().collect())
     .collect()
 }
 