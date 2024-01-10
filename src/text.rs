pub const LORE: &[&str] = &[ 
"The head of a gigantic mechanical snake, its blazing red eyes burning away the retinas of organics whom would dare stare too long. Its gold and chrome frills act as an attestation of the superiority of metal over muscle.\n\n[r]MELTDOWN[w] - Each turn, if this [y]Creature[w] is adjacent to 4 [y]Creatures[w], it gains one [l]Meltdown[w]. Upon reaching 5 [l]Meltdown[w], it immediately [r]Concedes[w].",

"Cyan Floods Wash Away Scorn - If possessed, Inject 1 Serene Soul into each Targeted Creature. Targeted Creatures become Charmed for Pride x 10 turns.",
"Steps Aligned, Minds United - Each Targeted Creature becomes Synchronized with the Caster for Grace x 10 turns.",
"One's Self, Hollow As A Costume - If the Caster possesses the Reality Anchor, it is given to the first Targeted Creature. After Glamour x 10 turns, it is given back to the Caster.",
"Imitate the Glorious, So They May Be Crushed - The Caster changes its Species to match that of the last Targeted Creature. After Discipline x 10 turns, it changes back to its old form.",
"Focused Thought Pierces the Veil - Form\nThe Caster shoots a linear beam in the direction of its Momentum, stopping at the first Creature hit. All Tiles touched, including the contacted Creature, are Targeted.",
];

pub fn match_species_with_description(
    species: &Species
) -> usize {
    match species {
        Species::EpsilonHead { len: _ } => 0,
        _ => 0,
    }
}

use bevy::{text::TextStyle, asset::AssetServer, ecs::system::Res, render::color::Color, log::info};
use regex::Regex;

use crate::species::Species;

pub fn split_text(
    text: &str,
    asset_server: &Res<AssetServer>,
) -> Vec<(String, TextStyle)> {
    let re = Regex::new(r"\[([^\]]+)\]").unwrap();

    let mut split_text = Vec::new();
    let mut colors = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text) {
        let start = cap.get(0).unwrap().start();
        let end = cap.get(0).unwrap().end();
        let tag = cap.get(1).unwrap().as_str().chars().next();
        colors.push(match_char_code_with_color(tag));
        split_text.push(&text[last_end..start]);
        last_end = end;
    }
    split_text.push(&text[last_end..]);
    
    let font = asset_server.load("Play-Regular.ttf");
    let mut output = Vec::new();

    for i in 0..split_text.len() {
        let color = if i == 0 {
            Color::WHITE
        } else {
            colors[i-1]
        };
        output.push((
            split_text[i].to_owned(),
            TextStyle {
                font: font.clone(),
                font_size: 20.,
                color,
            }
        ));
    }
    output
}

fn match_char_code_with_color(
    some_char: Option<char>
) -> Color {
    match some_char{
        Some(char) => {
            match char {
                'p' => Color::VIOLET,
                'r' => Color::ORANGE_RED,
                'y' => Color::YELLOW,
                'w' => Color::WHITE,
                'l' => Color::LIME_GREEN,
                'c' => Color::CYAN,
                _ => {
                    info!("Warning, an invalid color tag was used.");
                    Color::WHITE
                }
            }
        },
        None => panic!("There was no character in the text split!")
    }
}