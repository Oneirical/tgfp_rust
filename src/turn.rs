use std::{time::Duration, f32::consts::PI, cmp::Ordering};

use bevy::prelude::*;
use bevy_tweening::{*, lens::{TransformPositionLens, TransformScaleLens}};
use rand::seq::SliceRandom;

use crate::{components::{QueuedAction, RealityAnchor, Position, SoulBreath}, input::ActionType, TurnState, map::{xy_idx, WorldMap, is_in_bounds, bresenham_line, get_neighbouring_entities}, soul::{Soul, get_soul_rot_position, SoulRotationTimer, match_soul_with_display_index}, ui::CenterOfWheel, axiom::{grab_coords_from_form, CasterInfo, match_soul_with_axiom, Function}, species::Species, ZoomInEffect};

pub struct TurnPlugin;

#[derive(Debug)]
pub enum Animation{
}

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_actions.run_if(in_state(TurnState::CalculatingResponse)));
        app.add_systems(Update, execute_turn.run_if(in_state(TurnState::ExecutingTurn)));
        app.add_systems(Update, dispense_functions.run_if(in_state(TurnState::DispensingFunctions)));
        app.add_systems(Update, unpack_animations.run_if(in_state(TurnState::UnpackingAnimation)));
    }
}

fn calculate_actions (
    mut creatures: Query<(&mut QueuedAction, &Position, &Species), Without<RealityAnchor>>,
    read_species: Query<&Species>,
    read_position: Query<&Position>,
    mut next_state: ResMut<NextState<TurnState>>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
){
    for (mut queue, pos, species) in creatures.iter_mut(){
        queue.action = match species {
            Species::EpsilonHead => {
                let neigh = get_neighbouring_entities(&world_map.entities, pos.x, pos.y);
                for detected in neigh{
                    match detected {
                        Some(creature) => {
                            match read_species.get(creature).unwrap() {
                                Species::EpsilonTail { order } => {
                                    if order.eq(&0) {
                                        let crea_pos = read_position.get(creature).unwrap();
                                        let momentum = (pos.x as i32-crea_pos.x as i32, pos.y as i32-crea_pos.y as i32);
                                        commands.entity(creature).insert(QueuedAction{action: ActionType::Walk { momentum }});
                                    } else {continue;}
                                },
                                _ => continue,
                            };
                        }
                        None => continue,
                    };
                }
                let mut rng = rand::thread_rng();
                let mom = [(0,1),(0,-1),(1,0),(-1,0)].choose(&mut rng).unwrap();
                ActionType::Walk { momentum: *mom }
            },
            Species::EpsilonTail { order } => {
                let neigh = get_neighbouring_entities(&world_map.entities, pos.x, pos.y);
                let target_order = order+1;
                for detected in neigh{
                    match detected {
                        Some(creature) => {
                            match read_species.get(creature).unwrap() {
                                Species::EpsilonTail { order } => {
                                    if order.eq(&target_order) {
                                        let crea_pos = read_position.get(creature).unwrap();
                                        let momentum = (pos.x as i32-crea_pos.x as i32, pos.y as i32-crea_pos.y as i32);
                                        commands.entity(creature).insert(QueuedAction{action: ActionType::Walk { momentum }});
                                    } else {continue;}
                                },
                                _ => continue,
                            };
                        }
                        None => continue,
                    };
                }
                ActionType::Nothing
            },
            _ => ActionType::Nothing,
        };
    }
    next_state.set(TurnState::ExecutingTurn);
}

fn execute_turn (
    mut creatures: Query<(Entity, &QueuedAction, &Species, &mut SoulBreath, &mut Position, Has<RealityAnchor>)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    souls: Query<(&mut Animator<Transform>, &Transform, &Soul), Without<Position>>,
){
    for (entity, queue, species, breath, pos, is_player) in creatures.iter_mut(){
        
        match queue.action{
            ActionType::SoulCast { slot } => {
                let soul = match breath.held.get(slot).cloned(){ // Check that we aren't picking an empty slot.
                    Some(soul) => soul,
                    None => continue
                };
                let info = CasterInfo{ pos: (pos.x,pos.y), species: species.clone(), momentum: pos.momentum, is_player};
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
                world_map.targeted_axioms.push((entity, Function::LinearDash { dist: 1 }, CasterInfo{pos: (pos.x, pos.y), species: species.clone(), momentum, is_player}));
            },
            ActionType::Nothing => ()
        };
    }
    world_map.targeted_axioms.sort_by(|a, b| { // 
        match (a.2.is_player, b.2.is_player) {
            (true, true) | (false, false) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        }
    });
    world_map.targeted_axioms.reverse();
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
        if let Ok((_transform, _species, mut breath, _anim, mut pos, is_player)) = creatures.get_mut(entity.to_owned()) {
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

                    //if anim.tweenable().progress() != 1.0 { continue; }
                    if !is_player {
                        continue;
                    }
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
                            if false && world_map.entities[xy_idx(nx as usize, ny as usize)].is_some() {
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
        world_map.anim_queue.reverse(); // I will probably forget about this and rage later
        next_state.set(TurnState::UnpackingAnimation);
    }
}

fn unpack_animations(
    mut creatures: Query<(&mut Transform, &mut Animator<Transform>, &Position, Has<RealityAnchor>), With<Position>>,
    player: Query<&Position, With<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    time: Res<Time>,
){
    world_map.animation_timer.tick(time.delta());
    if !world_map.animation_timer.just_finished() {
        return;
    }
    let (player_pos, player_trans) = if let Ok(pos) = player.get_single() {
        ((pos.x,pos.y),Vec2::new(11., 4.)) // These hardcoded values might be dangerous
    } else {panic!("0 or 2 players!")};
    let (entity, anim_choice) = match world_map.anim_queue.pop() { // The fact that this is pop and not a loop might cause "fake" lag with a lot of queued animations
        Some(element) => element,
        None => {
            for (trans_crea, mut anim_crea, fini, is_player) in creatures.iter_mut(){
                let end = Vec3::new(player_trans.x + (fini.x as f32 -player_pos.0 as f32)/2., player_trans.y + (fini.y as f32 -player_pos.1 as f32)/2., 0.);
                if is_player {continue;}
                let tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(150), // must be the same as input delay to avoid offset
                    TransformPositionLens {
                        start: trans_crea.translation,
                        end
                    },
                );
                anim_crea.set_tweenable(tween);
            }
            world_map.animation_timer.set_duration(Duration::from_millis(1));
            next_state.set(TurnState::AwaitingInput);
            return;
        }
    };
    if let Ok((transform, mut anim, fin, _is_player)) = creatures.get_mut(entity.to_owned()) {
        match anim_choice {

        }
    }
}