use std::time::Duration;

use bevy::{prelude::*, text::BreakLineOn};
use bevy_tweening::{EaseFunction, Tween, lens::TransformPositionLens, Animator};

use crate::{InputDelay, TurnState, components::{RealityAnchor, QueuedAction, Position, Cursor, LogIndex, CreatureDescription}, map::{is_in_bounds, WorldMap, xy_idx}, axiom::tup_i32_to_usize, soul::CurrentEntityInUI, text::{match_species_with_description, LORE, split_text}, species::Species};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputBindings{
            up: vec![KeyCode::W, KeyCode::Up],
            down: vec![KeyCode::S, KeyCode::Down],
            right: vec![KeyCode::D, KeyCode::Right],
            left: vec![KeyCode::A, KeyCode::Left],
            one: vec![KeyCode::Key1],
            two: vec![KeyCode::Key2],
            three: vec![KeyCode::Key3],
            four: vec![KeyCode::Key4],
            cursor: vec![KeyCode::Q],
        });
        app.add_systems(Update, await_input.run_if(in_state(TurnState::AwaitingInput)));
        app.add_systems(Update, move_cursor.run_if(in_state(TurnState::ExaminingCreatures)));
        app.add_systems(OnEnter(TurnState::ExaminingCreatures), reset_cursor);
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ActionType{
    Walk { momentum: (i32, i32)},
    SoulCast {slot: usize},
    Nothing,
}

#[derive(Resource, Clone)]
struct InputBindings{
    up: Vec<KeyCode>,
    down: Vec<KeyCode>,
    left: Vec<KeyCode>,
    right: Vec<KeyCode>,
    one: Vec<KeyCode>,
    two: Vec<KeyCode>,
    three: Vec<KeyCode>,
    four: Vec<KeyCode>,
    cursor: Vec<KeyCode>,
}

fn await_input(
    input: Res<Input<KeyCode>>,
    bindings: Res<InputBindings>,
    mut player: Query<&mut QueuedAction, With<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    let mut reset_queued = true;
    let action = if input.any_pressed(bindings.up.clone()){
        ActionType::Walk { momentum: (0,1)}
    }
    else if input.any_pressed(bindings.down.clone()){
        ActionType::Walk { momentum: (0,-1)}
    }
    else if input.any_pressed(bindings.left.clone()){
        ActionType::Walk { momentum: (-1, 0)}
    }
    else if input.any_pressed(bindings.right.clone()){
        ActionType::Walk { momentum: (1, 0)}
    }
    else if input.any_just_pressed(bindings.one.clone()){
        ActionType::SoulCast { slot: 0 }
    }
    else if input.any_just_pressed(bindings.two.clone()){
        ActionType::SoulCast { slot: 1 }
    }
    else if input.any_just_pressed(bindings.three.clone()){
        ActionType::SoulCast { slot: 2 }
    }
    else if input.any_just_pressed(bindings.four.clone()){
        ActionType::SoulCast { slot: 3 }
    }
    else if input.any_just_pressed(bindings.cursor.clone()){
        next_state.set(TurnState::ExaminingCreatures);
        return;
    }
    else { 
        reset_queued = false;
        ActionType::Nothing
    };
    if reset_queued {
        if let Ok(mut queued) = player.get_single_mut() {
            queued.action = action.clone();
            next_state.set(TurnState::CalculatingResponse);
        } else {
            panic!("There are zero or more than 1 players!")
        }            
    }        
}

fn reset_cursor(
    mut cursor: Query<(&mut Cursor, &mut Visibility, &mut Transform), Without<RealityAnchor>>,
    player: Query<(&Position, &Transform), With<RealityAnchor>>,
    mut log_messages: Query<&mut Visibility, (With<LogIndex>, Without<CreatureDescription>, Without<RealityAnchor>, Without<Cursor>)>,
    mut desc: Query<&mut Visibility, (With<CreatureDescription>, Without<LogIndex>, Without<RealityAnchor>, Without<Cursor>)>,
){
    let (pos, p_t) = player.get_single().unwrap();
    let mut desc_vis = desc.get_single_mut().unwrap();
    let (mut pointer, mut vis, mut trans) = cursor.get_single_mut().unwrap();
    (pointer.x, pointer.y) = (pos.x, pos.y);
    *vis = Visibility::Visible;
    trans.translation = p_t.translation;
    *desc_vis = Visibility::Visible;
    for mut log_vis in log_messages.iter_mut() {
        *log_vis = Visibility::Hidden;
    }
}

fn move_cursor(
    mut cursor: Query<(&mut Cursor, &mut Animator<Transform>, &mut Visibility, &Transform), Without<RealityAnchor>>,
    player: Query<&Position, With<RealityAnchor>>,
    read_species: Query<&Species>,
    mut delay: ResMut<InputDelay>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    bindings: Res<InputBindings>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut inspected: ResMut<CurrentEntityInUI>,
    world_map: Res<WorldMap>,
    asset_server: Res<AssetServer>,

    mut log_messages: Query<&mut Visibility, (With<LogIndex>, Without<CreatureDescription>, Without<RealityAnchor>, Without<Cursor>)>,
    mut desc: Query<(&mut Text, &mut Visibility), (With<CreatureDescription>, Without<LogIndex>, Without<RealityAnchor>, Without<Cursor>)>,
) {
    let (mut pointer, mut anim, mut vis, trans) = cursor.get_single_mut().unwrap();
    let (mut text, mut desc_vis) = desc.get_single_mut().unwrap();
    let pos= player.get_single().unwrap();
    if input.any_just_pressed(bindings.cursor.clone()){
        next_state.set(TurnState::AwaitingInput);
        *vis = Visibility::Hidden;
        if let Some(crea) = world_map.entities[xy_idx(pos.x, pos.y)] { inspected.entity = crea } else {panic!("Where did the player go?")};

        for mut log_vis in log_messages.iter_mut() {
            *log_vis = Visibility::Visible;
        }
        *desc_vis = Visibility::Hidden;

        let mut text_sections = Vec::new();
        let crea_of_interest = read_species.get(inspected.entity).unwrap();
        let chosen_text = match LORE.get(match_species_with_description(crea_of_interest)) {
            Some(lore) => *lore,
            None => "\"Hi, Onei here. That text index doesn't exist, so here I am instead. Please report this bug.\"",
        };
        let split_text = split_text(chosen_text, &asset_server);
        for (snippet, style) in split_text {
            text_sections.push(TextSection::new(snippet, style));
        }
        let set_text = Text {
            sections: text_sections,
            alignment: TextAlignment::Left,
            linebreak_behavior: BreakLineOn::WordBoundary
        };
        *text = set_text;
        return;
    }
    delay.time.tick(time.delta());
    if !delay.time.finished() {
        return;
    }
    let mut action = if input.any_pressed(bindings.up.clone()){
        (0.,1.)
    }
    else if input.any_pressed(bindings.down.clone()){
        (0.,-1.)
    }
    else if input.any_pressed(bindings.left.clone()){
        (-1., 0.)
    }
    else if input.any_pressed(bindings.right.clone()){
        (1., 0.) 
    } else {
        (0., 0.)
    };
    let new_pos = (pointer.x as i32 + action.0 as i32, pointer.y as i32 + action.1 as i32);
    if is_in_bounds(new_pos.0, new_pos.1) && (pos.x as i32 - new_pos.0).abs() < 16 && (pos.y as i32 - new_pos.1).abs() < 16{
        (pointer.x, pointer.y) = tup_i32_to_usize(new_pos);
    } else {
        action = (0., 0.);
    }
    if action != (0., 0.) {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(50),
            TransformPositionLens {
                start: trans.translation,
                end: Vec3::new((trans.translation.x *2. + action.0).round()/2., (trans.translation.y *2. + action.1).round()/2., 10.)
            },
        );
        if let Some(crea) = world_map.entities[xy_idx(pointer.x, pointer.y)] { inspected.entity = crea } else {};
        anim.set_tweenable(tween);
        delay.time.reset();

        let mut text_sections = Vec::new();
        let crea_of_interest = read_species.get(inspected.entity).unwrap();
        let chosen_text = match LORE.get(match_species_with_description(crea_of_interest)) {
            Some(lore) => *lore,
            None => "\"Hi, Onei here. That text index doesn't exist, so here I am instead. Please report this bug.\"",
        };
        let split_text = split_text(chosen_text, &asset_server);
        for (snippet, style) in split_text {
            text_sections.push(TextSection::new(snippet, style));
        }
        let set_text = Text {
            sections: text_sections,
            alignment: TextAlignment::Left,
            linebreak_behavior: BreakLineOn::WordBoundary
        };
        *text = set_text;
    }
}