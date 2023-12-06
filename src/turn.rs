use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{*, lens::TransformPositionLens};

use crate::{components::{QueuedAction, RealityAnchor, Position}, input::{ActionType, direction}, TurnState, map::{xy_idx, WorldMap, WORLD_WIDTH, WORLD_HEIGHT}};

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_actions.run_if(in_state(TurnState::CalculatingResponse)));
        app.add_systems(Update, execute_turn.run_if(in_state(TurnState::ExecutingTurn)));
    }
}

fn calculate_actions (
    mut creatures: Query<&mut QueuedAction, Without<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
){
    for mut queue in creatures.iter_mut(){
        queue.action = ActionType::Nothing;
    }
    next_state.set(TurnState::ExecutingTurn);
}

fn execute_turn (
    mut creatures: Query<(&QueuedAction, &Transform, &mut Animator<Transform>, &mut Position)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,

){
    for (queue, transform, mut anim, mut pos) in creatures.iter_mut(){
        let mut direction = direction(queue.action.clone());

        if direction == Vec2::ZERO {
            continue;
        }
        if direction.x < 0. && pos.x == 0 || direction.x > 0. && pos.x == WORLD_WIDTH{
            direction.x = 0.;
        }
        if direction.y < 0. && pos.y == 0 || direction.y > 0. && pos.y == WORLD_HEIGHT{
            direction.y = 0.;
        }
        assert!(world_map.entities[xy_idx(pos.x, pos.y)].is_some());
        let (old_x, old_y) = (pos.x, pos.y);
        let old_idx = xy_idx(pos.x, pos.y);
        pos.x = (pos.x as f32 + direction.x) as usize;
        pos.y = (pos.y as f32 + direction.y) as usize;
        if world_map.entities[xy_idx(pos.x, pos.y)].is_some() {
            (pos.x, pos.y) = (old_x, old_y);
            continue;
        }
        let idx = xy_idx(pos.x, pos.y);
        world_map.entities.swap(old_idx, idx);
        

        let start = transform.translation;
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(200),
            TransformPositionLens {
                start,
                end: Vec3::new(pos.x as f32/2., pos.y as f32/2., 0.),
            },
        );
        anim.set_tweenable(tween);

    }
    next_state.set(TurnState::AwaitingInput);
}