use std::{time::Duration, f32::consts::PI};

use bevy::prelude::*;
use bevy_tweening::{*, lens::{TransformPositionLens, TransformScaleLens}};
use rand::seq::SliceRandom;

use crate::{components::{QueuedAction, RealityAnchor, Position, SoulBreath, UIElement}, input::ActionType, TurnState, map::{xy_idx, WorldMap, WORLD_WIDTH, WORLD_HEIGHT, is_in_bounds, bresenham_line}, soul::Soul, ui::CenterOfWheel, axiom::{grab_coords_from_form, CasterInfo, match_soul_with_axiom, Function}, species::Species};

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_actions.run_if(in_state(TurnState::CalculatingResponse)));
        app.add_systems(Update, execute_turn.run_if(in_state(TurnState::ExecutingTurn)));
        app.add_systems(Update, dispense_functions.run_if(in_state(TurnState::DispensingFunctions)));
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
    mut creatures: Query<(Entity, &QueuedAction, &Transform, &Species, &mut SoulBreath, &mut Animator<Transform>, &mut Position)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &mut UIElement, &Transform, &Soul), Without<Position>>,
    ui_center: Res<CenterOfWheel>,
){
    for (entity, queue, transform, species, mut breath, mut anim, mut pos) in creatures.iter_mut(){
        
        match queue.action{
            ActionType::SoulCast { slot } => {
                let soul = match breath.held.get(slot).cloned(){ // Check that we aren't picking an empty slot.
                    Some(soul) => soul,
                    None => continue
                };

                if let Ok((_anim, _ui, _transform, soul_id), ) = souls.get(soul) {
                    let axioms = breath.axioms.clone();
                    let (form, function) = axioms[match_soul_with_axiom(soul_id)].clone();
                    let info = CasterInfo{ pos: (pos.x,pos.y), species: species.clone(), momentum: pos.momentum};
                    let targets = grab_coords_from_form(&world_map.entities, form, info.clone());
                    for target in targets.entities {
                        world_map.targeted_axioms.push((target, function.clone(), info.clone()));
                    }
                } else {
                    panic!("The used Soul did not have a soul type!");
                }

                breath.discard.push(soul); // Move the soul to the discard.
                if breath.pile.is_empty() { // If empty, reshuffle.
                    breath.discard.shuffle(&mut rand::thread_rng());
                    let mut new_content = breath.discard.clone();
                    breath.pile.append(&mut new_content);
                    breath.discard.clear();
                }
                let replacement = breath.pile.pop();
                match replacement { // Replace the used soul.
                    Some(new_soul) => {
                        breath.held[slot] = new_soul;

                        let slot_coords = [ // TODO: Adjust these to the centerofwheel.
                            (22.689, 9.561),
                            (24.811, 9.561),
                            (22.689, 7.439),
                            (24.811, 7.439),
                        ];
                        if let Ok((mut anim, mut ui, transform, soul_id), ) = souls.get_mut(new_soul) { 

                            let tween_tr = Tween::new(
                                EaseFunction::QuadraticInOut,
                                Duration::from_millis(500),
                                TransformPositionLens {
                                    start: transform.translation,
                                    end: Vec3{ x: slot_coords[slot].0, y: slot_coords[slot].1, z: 0.5},
                                },
                            );
                            let tween_sc = Tween::new(
                                EaseFunction::QuadraticInOut,
                                Duration::from_millis(500),
                                TransformScaleLens {
                                    start: transform.scale,
                                    end: Vec3{ x: transform.scale.x*2.2, y: transform.scale.y*2.2, z: 0.},
                                },
                            );
                            let track = Tracks::new([tween_tr, tween_sc]);
                            anim.set_tweenable(track);
                            (ui.x, ui.y) = slot_coords[slot];
                        }
                    },
                    None => panic!("The Breath pile is still empty after reshuffling it!")
                }
            }
            ActionType::Walk { momentum } => {
                world_map.targeted_axioms.push((entity, Function::Dash { dist: 1 }, CasterInfo{pos: (pos.x, pos.y), species: species.clone(), momentum}));
            },
            ActionType::Nothing => ()
        };
    }
    next_state.set(TurnState::DispensingFunctions);
}

fn dispense_functions(
    mut creatures: Query<(&QueuedAction, &Transform, &Species, &mut SoulBreath, &mut Animator<Transform>, &mut Position)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &mut UIElement, &Transform, &Soul), Without<Position>>,
){
    for (i, (entity, function, info)) in world_map.targeted_axioms.clone().iter().enumerate(){
        if let Ok((queue, transform, species, mut breath, mut anim, mut pos)) = creatures.get_mut(entity.to_owned()) {
            let function = function.to_owned();
            match function {
                Function::Dash { dist } => {
                    let dest = (dist as i32 * info.momentum.0, dist as i32 * info.momentum.1);
                    let mut line = bresenham_line(pos.x as i32, pos.y as i32, pos.x as i32 + dest.0, pos.y as i32 + dest.1);
                    let old_idx = xy_idx(pos.x, pos.y);
                    line.remove(0); // remove the origin point
                    for (nx, ny) in line {
                        if is_in_bounds(nx, ny){
                            if world_map.entities[xy_idx(nx as usize, ny as usize)].is_some() {
                                // TODO Raise a collision event here
                                break;
                            }
                            else {(pos.x, pos.y) = (nx as usize, ny as usize)}
                        } else {
                            break;
                        }
                    }
                    let idx = xy_idx(pos.x, pos.y);
                    world_map.entities.swap(old_idx, idx);

                    let max = dest.0.abs().max(dest.1.abs());
                    pos.momentum = if max == dest.0.abs(){ // Reassign the new momentum.
                        (dest.0/dest.0.abs(), 0)
                    } else {
                        (0, dest.1/dest.1.abs())
                    };

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
                },
                Function::DiscardSlot { slot } => todo!(),
                Function::Empty => ()
            };
            world_map.targeted_axioms.remove(i);
        }
    }
    if world_map.targeted_axioms.is_empty() {
        next_state.set(TurnState::AwaitingInput);
    }
}

/*
                let mut direction = Vec2::new(dir.0, dir.1);
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
                 */