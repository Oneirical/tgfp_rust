use bevy::prelude::*;

use crate::{InputDelay, TurnState, components::{RealityAnchor, QueuedAction, Position, Cursor}};

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

fn move_cursor(
    mut cursor: Query<&Cursor>,
    mut delay: ResMut<InputDelay>,
    input: Res<Input<KeyCode>>,
    bindings: Res<InputBindings>,
) {
    let pointer = cursor.get_single_mut();
    let action = if input.any_pressed(bindings.up.clone()){
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

}