use bevy::prelude::*;

use crate::{InputDelay, TurnState, components::{RealityAnchor, QueuedAction}};

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
        });
        app.add_systems(Update, await_input.run_if(in_state(TurnState::AwaitingInput)));
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
}

fn await_input(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut delay: ResMut<InputDelay>,
    bindings: Res<InputBindings>,
    mut player: Query<&mut QueuedAction, With<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    if !delay.time.finished() {
        delay.time.tick(time.delta());
    }
    if delay.time.finished() {
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
        else if input.any_pressed(bindings.one.clone()){
            ActionType::SoulCast { slot: 0 }
        }
        else if input.any_pressed(bindings.two.clone()){
            ActionType::SoulCast { slot: 1 }
        }
        else if input.any_pressed(bindings.three.clone()){
            ActionType::SoulCast { slot: 2 }
        }
        else if input.any_pressed(bindings.four.clone()){
            ActionType::SoulCast { slot: 3 }
        }
        else { 
            reset_queued = false;
            ActionType::Nothing
        };
        if reset_queued {
            if let Ok(mut queued) = player.get_single_mut() {
                queued.action = action.clone();
                next_state.set(TurnState::CalculatingResponse);
                delay.time.reset();
            } else {
                panic!("There are zero or more than 1 players!")
            }            
        }        
    }

}