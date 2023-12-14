use std::{time::Duration, f32::consts::PI};

use bevy::prelude::*;
use bevy_tweening::{*, lens::{TransformPositionLens, TransformScaleLens}};
use rand::seq::SliceRandom;

use crate::{components::{QueuedAction, RealityAnchor, Position, SoulBreath}, input::ActionType, TurnState, map::{xy_idx, WorldMap, is_in_bounds, bresenham_line, PlanePassage}, soul::{Soul, get_soul_rot_position, SoulRotationTimer, match_soul_with_display_index}, ui::CenterOfWheel, axiom::{grab_coords_from_form, CasterInfo, match_soul_with_axiom, Function}, species::Species, world::Plane, ZoomInEffect};

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
    mut creatures: Query<(Entity, &QueuedAction, &Species, &mut SoulBreath, &mut Position)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    souls: Query<(&mut Animator<Transform>, &Transform, &Soul), Without<Position>>,
){
    for (entity, queue, species, breath, pos) in creatures.iter_mut(){
        
        match queue.action{
            ActionType::SoulCast { slot } => {
                let soul = match breath.held.get(slot).cloned(){ // Check that we aren't picking an empty slot.
                    Some(soul) => soul,
                    None => continue
                };
                let info = CasterInfo{ pos: (pos.x,pos.y), species: species.clone(), momentum: pos.momentum};
                if let Ok((_anim, _transform, soul_id), ) = souls.get(soul) {
                    let axioms = breath.axioms.clone();
                    let (form, function) = axioms[match_soul_with_axiom(soul_id)].clone();
                    let targets = grab_coords_from_form(&world_map.entities, form, info.clone());
                    for target in targets.entities {
                        world_map.targeted_axioms.push((target, function.clone(), info.clone()));
                    }
                } else {
                    panic!("The used Soul did not have a soul type!");
                }

                world_map.targeted_axioms.push((entity, Function::DiscardSoul { soul, slot }, info.clone()));
            }
            ActionType::Walk { momentum } => {
                world_map.targeted_axioms.push((entity, Function::LinearDash { dist: 1 }, CasterInfo{pos: (pos.x, pos.y), species: species.clone(), momentum}));
            },
            ActionType::Nothing => ()
        };
    }
    next_state.set(TurnState::DispensingFunctions);
}

fn dispense_functions(
    mut creatures: Query<(&Transform, &Species, &mut SoulBreath, &mut Animator<Transform>, &mut Position, Has<RealityAnchor>)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &Transform, &Soul), Without<Position>>,
    ui_center: Res<CenterOfWheel>,
    time: Res<SoulRotationTimer>,
    mut zoom: ResMut<ZoomInEffect>,
){
    let mut next_axioms = Vec::new();
    for (entity, function, info) in world_map.targeted_axioms.clone().iter(){
        if let Ok((_transform, _species, mut breath, mut anim, mut pos, is_player)) = creatures.get_mut(entity.to_owned()) {
            let function = function.to_owned();
            match function {
                Function::Teleport { x, y } => {
                    if world_map.entities[xy_idx(x, y)].is_some() { // Cancel teleport if target is occupied
                        continue;
                    }
                    let old_pos = (pos.x, pos.y);
                    let old_idx = xy_idx(pos.x, pos.y);
                    (pos.x, pos.y) = (x, y);
                    let new_pos = (x, y);
                    let dest = (pos.x as i32 -old_pos.0 as i32, pos.y as i32-old_pos.1 as i32);
                    let idx = xy_idx(pos.x, pos.y);
                    world_map.entities.swap(old_idx, idx);
                    let max = dest.0.abs().max(dest.1.abs());
                    assert!(!(dest.0 == 0 && dest.1 == 0));
                    pos.momentum = if max == dest.0.abs(){ // Reassign the new momentum.
                        (dest.0/dest.0.abs(), 0)
                    } else {
                        (0, dest.1/dest.1.abs())
                    };
                    let tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(150),
                        TransformPositionLens {
                            start: Vec3::new(old_pos.0 as f32/2., old_pos.1 as f32/2., 0.),
                            end: Vec3::new(pos.x as f32/2., pos.y as f32/2., 0.),
                        },
                    );
                    //if anim.tweenable().progress() != 1.0 { continue; }
                    if !is_player {anim.set_tweenable(tween)}
                    else {
                        let mut passage_detected = false;
                        let mut player_coords = (0., 0.);
                        for (transform, _species, _breath, mut anim, posi, is_player) in creatures.iter_mut(){
                            if is_player {
                                for (passage_coords, destination) in &world_map.warp_zones{
                                    if new_pos == *passage_coords {
                                        passage_detected = true;
                                        player_coords = (posi.x as f32, posi.y as f32);
                                        zoom.destination = destination.clone();
                                    }
                                }

                                continue;
                            }
                            let tween = Tween::new(
                                EaseFunction::QuadraticInOut,
                                Duration::from_millis(150), // must be the same as input delay to avoid offset
                                TransformPositionLens {
                                    start: transform.translation,
                                    end: Vec3::new(transform.translation.x + (old_pos.0 as f32-new_pos.0 as f32)/2., transform.translation.y + (old_pos.1 as f32-new_pos.1 as f32)/2., 0.),
                                },
                            );
                            anim.set_tweenable(tween);
                        }
                        if passage_detected{
                            zoom.timer.unpause();
                            for (transform, _species, _breath, mut anim, posi, is_player) in creatures.iter_mut(){
                                if is_player {continue;}
                                let tween_sc = Tween::new(
                                    EaseFunction::QuadraticInOut,
                                    Duration::from_millis(500), // must be the same as input delay to avoid offset
                                    TransformScaleLens {
                                        start: transform.scale,
                                        end: Vec3::new(8., 8., 1.),
                                    },
                                );
                                let tween_tr = Tween::new(
                                    EaseFunction::QuadraticInOut,
                                    Duration::from_millis(500), // must be the same as input delay to avoid offset
                                    TransformPositionLens {
                                        start: transform.translation,
                                        end: Vec3::new(posi.x as f32/2.*8.-player_coords.0/2.*8.+11., posi.y as f32/2.*8.-player_coords.1/2.*8.+3.5, transform.translation.z),
                                    },
                                );
                                let track = Tracks::new([tween_sc,tween_tr]);
                                anim.set_tweenable(track);
                            }
                        }
                    }
                }
                Function::Dash { dx, dy } => {
                    let dest = (dx, dy);
                    let mut line = bresenham_line(pos.x as i32, pos.y as i32, pos.x as i32 + dest.0, pos.y as i32 + dest.1);
                    line.remove(0); // remove the origin point
                    let (mut fx, mut fy) = (pos.x, pos.y);
                    for (nx, ny) in line {
                        if is_in_bounds(nx, ny){
                            if world_map.entities[xy_idx(nx as usize, ny as usize)].is_some() {
                                // TODO Raise a collision event here
                                break;
                            }
                            else {(fx, fy) = (nx as usize, ny as usize)}
                        } else {
                            break;
                        }
                    }
                    next_axioms.push((*entity, Function::Teleport { x: fx, y: fy }, info.clone()));
                },
                Function::LinearDash { dist } => {
                    let dest = (dist as i32 * info.momentum.0, dist as i32 * info.momentum.1);
                    next_axioms.push((*entity, Function::Dash { dx: dest.0, dy: dest.1 }, info.clone()));
                }
                Function::DiscardSoul { soul, slot } => {
                    if let Ok((mut anim, transform, soul_id), ) = souls.get_mut(soul) { 
                        // Move the soul to the discard.
                        breath.discard[match_soul_with_display_index(soul_id)].push(soul);
                        let index = breath.discard[match_soul_with_display_index(soul_id)].iter().position(|&ent| ent == soul);
                        let final_pos = get_soul_rot_position(soul_id, (ui_center.x, ui_center.y), true, time.timer.elapsed_secs()+0.5, index.unwrap());
                        let tween_tr = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_millis(500),
                            TransformPositionLens {
                                start: transform.translation,
                                end: Vec3{ x: final_pos.0, y: final_pos.1, z: 0.5},
                            },
                        );
                        let tween_sc = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_millis(500),
                            TransformScaleLens {
                                start: transform.scale,
                                end: Vec3{ x: transform.scale.x/3., y: transform.scale.y/3., z: 0.},
                            },
                        );
                        let track = Tracks::new([tween_tr, tween_sc]);
                        anim.set_tweenable(track);
                    }
                    let mut rng = rand::thread_rng();
                    let mut possible_indices = Vec::new();
                    for (i, vec) in breath.pile.iter().enumerate() { if !vec.is_empty() {possible_indices.push(i)}}
                    if possible_indices.is_empty() {
                        for j in breath.discard.iter() { // Reshuffle if no souls are left!
                            for i in j.iter(){
                                if let Ok((mut anim, transform, soul_id), ) = souls.get_mut(*i) { 
                                    let index = breath.discard[match_soul_with_display_index(soul_id)].iter().position(|&ent| ent == *i);
                                    let final_pos = get_soul_rot_position(soul_id, (ui_center.x, ui_center.y), false, time.timer.elapsed_secs()+0.5, index.unwrap());
                                    let tween_tr = Tween::new(
                                        EaseFunction::QuadraticInOut,
                                        Duration::from_millis(500),
                                        TransformPositionLens {
                                            start: transform.translation,
                                            end: Vec3{ x: final_pos.0, y: final_pos.1, z: 0.5},
                                        },
                                    );
                                    let tween_sc = Tween::new(
                                        EaseFunction::QuadraticInOut,
                                        Duration::from_millis(500),
                                        TransformScaleLens {
                                            start: transform.scale,
                                            end: Vec3{ x: 1., y: 1., z: 0.},
                                        },
                                    );
                                    let track = Tracks::new([tween_tr, tween_sc]);
                                    anim.set_tweenable(track);
                                }
                                else{ panic!("A soul in the draw pile has no UIElement component!")};
                            }
                        }
                        breath.pile = breath.discard.clone();
                        for vec in breath.discard.iter_mut() {
                            vec.clear();
                        }
                        for (i, vec) in breath.pile.iter().enumerate() { if !vec.is_empty() {possible_indices.push(i)}}
                    }
                    let index = possible_indices.choose(&mut rng);
                    let replacement = breath.pile[index.unwrap().to_owned()].pop();
                    match replacement { // Replace the used soul.
                        Some(new_soul) => {
                            breath.held[slot] = new_soul;
    
                            let slot_coords_ui = [
                                ((3.*PI/4.).cos() * 1.5 +ui_center.x, (3.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((1.*PI/4.).cos() * 1.5 +ui_center.x, (1.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((5.*PI/4.).cos() * 1.5 +ui_center.x, (5.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((7.*PI/4.).cos() * 1.5 +ui_center.x, (7.*PI/4.).sin() * 1.5 +ui_center.y)
                            ];
                            if let Ok((mut anim, transform, _soul_id), ) = souls.get_mut(new_soul) { 
                                let tween_tr = Tween::new(
                                    EaseFunction::QuadraticInOut,
                                    Duration::from_millis(500),
                                    TransformPositionLens {
                                        start: transform.translation,
                                        end: Vec3{ x: slot_coords_ui[slot].0, y: slot_coords_ui[slot].1, z: 0.5},
                                    },
                                );
                                let tween_sc = Tween::new(
                                    EaseFunction::QuadraticInOut,
                                    Duration::from_millis(500),
                                    TransformScaleLens {
                                        start: transform.scale,
                                        end: Vec3{ x: 3., y: 3., z: 0.},
                                    },
                                );
                                let track = Tracks::new([tween_tr, tween_sc]);
                                anim.set_tweenable(track);
                            }
                        },
                        None => panic!("The chosen Soul category had nothing left!")
                    }
                },
                Function::Empty => ()
            };
        }
    }
    world_map.targeted_axioms.clear();
    world_map.targeted_axioms.append(&mut next_axioms);
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