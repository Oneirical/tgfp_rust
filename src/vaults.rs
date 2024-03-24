use bevy::prelude::*;
use rand::{Rng, seq::SliceRandom};

use crate::{species::Species, axiom::{Form, Function, tup_i32_to_usize, tup_usize_to_i32}};

#[derive(Component, Clone)]
pub enum Vault {
    WorldSeed,
    EpicWow,
    Epsilon,
    Factory,
    EviePlants,
}

pub enum Structure {
    Platform,
    Crucible,
    Vault,
    SnakeEgg,
}


const VAULTS: &[&str] = &[
// 0. Empty Template
"
####^####
#.......#
#.......#
#.......#
<.......>
#.......#
#.......#
#.......#
####V####
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
"
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
........#.................................................................................
........#.................................................................................
........#.................................................................................
........#.................................................................................
..#####...................................................................................
..........................................................................................
..........................................................................................
..........E...............................................................................
..........C...............................................................................
..........C...............................................................................
..........C...............................................................................
..........................................................................................
.............#####..................#####.................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
........................#...#.............................................................
........................#...#.............................................................
........................#####.............................................................
..........................................................................................
..........................................................................................
...........####...........................................................................
...........#..#...........................................................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#####...............................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
........#.................................................................................
........#.................................................................................
........#.................................................................................
........#.................................................................................
..#####...................................................................................
..........................................................................................
..........................................................................................
..........E...............................................................................
..........C...............................................................................
..........C...............................................................................
..........C...............................................................................
..........................................................................................
.............#####..................#####.................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
..........................................................................................
........................#...#.............................................................
........................#...#.............................................................
........................#####.............................................................
..........................................................................................
..........................................................................................
...........####...........................................................................
...........#..#...........................................................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#...#...............................................
......................................#####...............................................
..........................................................................................
",
"
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.............................................
.....................&&&.....................
......................&......................
......................&......................
#############################################
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

pub fn build_spire(

) -> Vec<(Species, (usize, usize))>
{
  let mut structures = Vec::new();
  let mut rng = rand::thread_rng();
  let start_point = rng.gen_range(0..10);
  let mut current_center = (Species::Platform, (start_point, 1));
  let mut summit = 1;
  while summit < 40 {
    let mut platform = Vec::new();
    platform.push(current_center.clone());
    let platform_length = rng.gen_range(4..9);
    for _i in 0..platform_length {
        let x_pos = 44.min(current_center.1.0+1);
        current_center = (Species::Platform,(x_pos, summit));
        platform.push(current_center.clone());
    }
    let mut ladder = Vec::new();
    let ladder_length = rng.gen_range(3..6);
    let ladder_pos = rng.gen_range(0..platform.len()-1);
    let ladder_pos = platform[ladder_pos].1.0;
    for _i in 0..ladder_length {
        summit += 1;
        //ladder.push((Species::Ladder, (ladder_pos, summit)));
    }
    summit+= 1;
    current_center = ((Species::Platform), (ladder_pos, summit));
    //ladder.push((Species::EpsilonTail { order: -1 }, (ladder_pos, summit-1)));
    structures.append(&mut platform);
    structures.append(&mut ladder);
  }
  //structures.push((Species::EpsilonHead { len: 0 }, (2,2)));
  structures

}

pub fn build_pit() -> Vec<(Species, (usize, usize))> {
    let zones = select_struct_coords();
    let mut rng = rand::thread_rng();
    let mut output = Vec::new();
    let mut placed_blocks = Vec::new();
    let structures = [Structure::Crucible];
    for (x,y) in zones {
        let chosen_struct = structures.choose(&mut rng).unwrap();
        match chosen_struct {
            Structure::Crucible => {
                let num_sides = rng.gen_range(1..=4);
                let mut len_sides = Vec::new();
                let mut current_brush = tup_usize_to_i32((x,y));
                for _i in 0..num_sides {
                    len_sides.push(rng.gen_range(1..=9));
                }
                let mut stretch = [(0,-1),(0,1),(1,0),(-1,0)];
                stretch.shuffle(&mut rng);
                for (i, length) in len_sides.iter().enumerate() {
                    for _j in 0..*length {
                        if !placed_blocks.contains(&current_brush) {
                            output.push((Species::Wall, (tup_i32_to_usize(current_brush))));
                            placed_blocks.push(current_brush);
                        }
                        current_brush = (current_brush.0 + stretch[i].0, current_brush.1 + stretch[i].1);
                    }
                }
            }
            _ => ()
        }
    }
    output
}

pub fn select_struct_coords() -> Vec<(usize,usize)> {
    let mut rng = rand::thread_rng();
    let mut tuples = Vec::new();
 
    // Generate the first tuple
    let first_tuple = (rng.gen_range(10..=80), rng.gen_range(10..=80));
    tuples.push(first_tuple);
 
    // Generate the rest of the tuples
    for _ in 0..9 {
        loop {
            let new_tuple = (rng.gen_range(10..=80), rng.gen_range(10..=80));
 
            let mut is_valid = true;
            for existing_tuple in &tuples {
                let distance = ((new_tuple.0 as i32 - existing_tuple.0 as i32).abs() as f64)
                   .powi(2)
                   + ((new_tuple.1 as i32 - existing_tuple.1 as i32).abs() as f64).powi(2);
                if distance <= 9.0 * 9.0 {
                   is_valid = false;
                   break;
                }
            }
 
            if is_valid {
                tuples.push(new_tuple);
                break;
            }
        }
    }
 
    let mut output = Vec::new();

    for tuple in tuples {
        output.push(tup_i32_to_usize(tuple));
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
        Vault::WorldSeed => 0,
        Vault::EpicWow => 1,
        Vault::Epsilon => 2,
        Vault::Factory => 3,
        Vault::EviePlants => 4,
    }
}

pub fn match_vault_with_spawn_loc(
    vault: Vault
) -> (usize,usize){
    match vault {
        Vault::WorldSeed => (0,0),
        Vault::EpicWow => (22,8),
        Vault::Epsilon => (22,8),
        Vault::Factory => (10,10),
        Vault::EviePlants => (10,10),
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
        'C' => Species::EpsilonTail {order: -1},
        'P' => Species::AxiomCrate,
        'A' => Species::ChromeNurse,
        '*' => Species::SegmentTransformer,
        '0' => Species::CrateActivator { caste: 0 },
        '1' => Species::CrateActivator { caste: 1 },
        '2' => Species::CrateActivator { caste: 2 },
        '3' => Species::CrateActivator { caste: 3 },
        '!' => Species::FormCrate { form: Form::Ego },
        '@' => Species::FormCrate { form: Form::MomentumBeam },
        '$' => Species::FunctionCrate { function: Box::new(Function::MomentumDash) },
        '%' => Species::FunctionCrate { function: Box::new(Function::MomentumReverseDash) },
        '&' => Species::PlantSegment,
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
 