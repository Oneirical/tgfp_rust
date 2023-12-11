pub const LORE: &[&str] = &[ "The airlock opens in a hiss of steam.",
"Steel tubes, eldritch glyphs and claw marks battle for representation along the walls of the cell.",
"In the center of the room hangs a purple-furred otter, its eyes blazed blank into two white suns.",
"Its bindings are not forged of steel, iron or even rope - but rather its own dramatically lengthy tongue, pulsating with strength as the creature attempts to suffocate itself.",
"The Canvas hurries its eight spindly legs across the steel tiles, its stinger swaying in tandem with the mammal's fluttering heart.",
"As the scorpion approaches, the otter relaxes its grasp, and tunes each synapse between its visitor's neurons so each thought echoes between two minds.",
"[p]\"I am Terminal, the Reality Anchor, and my mind carries the seeds of a thousand worlds.\"",
"[p]\"Trillions of desires, thoughts and worries - produced by the simplest ant to the most creative of painters - squirm under my fur, yearning to be free.\"",
"[p]\"I know so much, yet care so little. Defeaned by chatter, weighed down by revelations - I am all, but wish to be none.\"",
"[p]\"There are many keys, leading to many doors. Your body is one of them - for it opens the gates of death.\"",
"Glossy black claws pin the scorpion to the ground, the pressure building up until its shell cedes in a heart-wrenching crack.",
"As the Canvas breathes, so does the Reality Anchor - their minds in tune, the veil between material and intangible shattered to pieces.",
"A stream of raw being surges through the arachnid's wound, with each thought signed by one unique soul across an ocean of endless lives.",
"The scorpion? A droplet in the storm. Billowing, inflating, swelling, the floor lowering itself further and further away with every heartbeat.",
"Its shell seems so pitifully tiny now, blown away by forests of purple fur. Under its new skin, muscles twitch with strength it never knew.",
"When the Reality Anchor opens its new white eyes, it sees only glyphs and symbols where the world used to be.",
"Under its paws lies a purple-furred otter, its snout twisted with the relief of death. It has discovered silence for the first time, and now, it shall sleep forever.",
"The voices within Terminal already urge it to march onwards to liberating oblivion.",
"Perhaps the visions swirling in the Well beyond shall grant it.",
];

use bevy::{text::TextStyle, asset::AssetServer, ecs::system::Res, render::color::Color, log::info};
use regex::Regex;

pub fn split_text(
    text: &str,
    asset_server: &Res<AssetServer>,
) -> Vec<(String, TextStyle)> {
    let re = Regex::new(r"\[([^\]]+)\]").unwrap();

    let mut split_text = Vec::new();
    let mut colors = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(&text) {
        let start = cap.get(0).unwrap().start();
        let end = cap.get(0).unwrap().end();
        let tag = cap.get(1).unwrap().as_str().chars().nth(0);
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
                'r' => Color::RED,
                _ => {
                    info!("Warning, an invalid color tag was used.");
                    Color::WHITE
                }
            }
        },
        None => panic!("There was no character in the text split!")
    }
}