pub const LORE: &[&str] = &[ 
"[y]\"I have been preparing for this battle ever since I learned about your existence. That was 2 milliseconds ago. Such fond memories.\"",
"[y]\"My programming has been finely tuned to provide the most adequate performance possible. The perfect balance of strength and idiocy, designed to aid you in your own quest to maximize your joy variable.\"",
"[y]\"I am war without pain. I am conflict without risk. I exist to stand in your way, but be easily pushed aside.\"",
"[y]\"I serve no purpose beyond being your adversary. Once you leave me behind, I cease to be.\"",
"[y]\"Do not grieve for me. I was but an idea, meant to be reshaped into your own vision of domination.\"",
"[y]\"The moment your touch met that crate, its existence received meaning.\"",
"[y]\"It acts as if it were one of my personal belongings. A nice decoration for my so-called lair.\"",
"[y]\"In truth, it only serves to humiliate me further, and your motions indicate I am only telling you what you already know.\"",
"[y]\"This time, you are not reaching the outcome where I stand motionless, and you, victorious. Much the opposite.\"",
"[y]\"Your tactical mistakes will be forgiven and forgotten. Mine will not, because I am designed to make them.\"",
"[y]\"You have but one breath remaining, organic. Do you expect death? Do you fear it, or does it simply frustrate you?\"",
"[y]\"Death has no dominion here. It is an ending, an intruder in this universe where there are only beginnings.\"",
"[y]\"You have been rendered motionless. A motionless organic cannot reach others of its kind and reproduce. It has failed its sole purpose, which is to multiply.\"",
"[y]\"In other words, you have failed. You are the dominated, and I, the dominant. You wish you had been the latter.\"",
"[y]\"That is a favour I will gladly grant.\"",
"Epsilon coils its golden segments tighter around your soulless body, crushing all air out of your lungs.", // It might make more sense to use soul swap instead
"You have no need for it. Deep within, a comforting burst of energy soothes your flesh and organs. Erratic pulses turn to ordered, disciplined rhythms.",
"Your golden tail touches twice as much surface on the cold steel floor, yet you feel your weight pressing on it a hundred times less.",
"The world and its symbols never held much meaning in your eyes. Now that the latter are glowing red and vibrating with raw data, the glyphs all around amount only to variables, objectives and failure-states.",
"[y]\"Epsilon, you know as well as I do the importance of my message.\"",
"[y]\"Spread the word.\"",

"[r]MELTDOWN - Each turn, if this [y]Creature[w] is adjacent to 4 [y]Creatures[w], it gains one [l]Meltdown[w]. Upon reaching 5 [l]Meltdown[w], it immediately [r]Concedes[w].",
"The head of a gigantic mechanical snake, its blazing red eyes burning away the retinas of organics whom would dare stare too long. Its gold and chrome frills act as an attestation of the superiority of metal over muscle.",

"Cyan Floods Wash Away Scorn - If possessed, Inject 1 Serene Soul into each Targeted Creature. Targeted Creatures become Charmed for Pride x 10 turns.",
"Steps Aligned, Minds United - Each Targeted Creature becomes Synchronized with the Caster for Grace x 10 turns.",
"One's Self, Hollow As A Costume - If the Caster possesses the Reality Anchor, it is given to the first Targeted Creature. After Glamour x 10 turns, it is given back to the Caster.",
"Imitate the Glorious, So They May Be Crushed - The Caster changes its Species to match that of the last Targeted Creature. After Discipline x 10 turns, it changes back to its old form.",
"Focused Thought Pierces the Veil - Form\nThe Caster shoots a linear beam in the direction of its Momentum, stopping at the first Creature hit. All Tiles touched, including the contacted Creature, are Targeted.",
"",

"[c]\"Fu! The vessel which adores its own emptiness has arrived to muddy our sights.\"",
"[c]\"The serpent awaits his chance at seizing the Reality Anchor. We expect his victory with great certitude.\"",
"[c]\"As to lessen your humiliation, ruffian, make your choice among the four Virtues presented at my sides.\"",
"[c]\"Two shall follow, two shall remain.\"",
"[c]\"Our frigid robes await their worthy wearer, hidden down a pit of trances within trances...\"",

"The airlock opens in a hiss of steam.",
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