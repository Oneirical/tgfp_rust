use std::{time::Duration, f32::consts::PI, cmp::Ordering};

use bevy::{prelude::*, utils::petgraph::visit::EdgeRef};
use bevy_tweening::{*, lens::{TransformPositionLens, TransformScaleLens, TransformRotationLens}};
use rand::seq::SliceRandom;

use crate::{components::{QueuedAction, RealityAnchor, Position, SoulBreath, AxiomEffects, EffectMarker, Faction}, input::ActionType, TurnState, map::{xy_idx, WorldMap, is_in_bounds, bresenham_line, get_neighbouring_entities, get_best_move, get_all_factions_except_one}, soul::{Soul, get_soul_rot_position, SoulRotationTimer, match_soul_with_display_index, match_soul_with_sprite, select_random_entities}, ui::CenterOfWheel, axiom::{grab_coords_from_form, CasterInfo, match_soul_with_axiom, Function, Form, match_axiom_with_soul}, species::{Species, match_faction_with_index}, ZoomInEffect, SpriteSheetHandle};

pub struct TurnPlugin;

#[derive(Debug, PartialEq, Clone)]
pub enum Animation{
    Passage,
    SoulDrain {source: Vec3, destination: Vec3, drained: Vec<Entity>},
    FormMark {coords: (usize, usize)},
    Soulless
}

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_actions.run_if(in_state(TurnState::CalculatingResponse)));
        app.add_systems(Update, execute_turn.run_if(in_state(TurnState::ExecutingTurn)));
        app.add_systems(Update, dispense_functions.run_if(in_state(TurnState::DispensingFunctions)));
        app.add_systems(Update, unpack_animations.run_if(in_state(TurnState::UnpackingAnimation)));
        app.add_systems(Update, fade_effects);
    }
}

fn choose_action (
    destination: (usize, usize),
    foes: Vec<Entity>,
    allies: Vec<Entity>,
    axioms: Vec<(Form, Function)>,
    polarity: Vec<i32>,
    available_souls: Vec<&Soul>,
    info: CasterInfo,
    world_map: &[Option<Entity>],
) -> ActionType {
    let mut scores = vec![0,0,0,0];
    for (i, (form, _function)) in axioms.iter().enumerate() {
        if !available_souls.contains(&&match_axiom_with_soul(i)) {
            scores[i] = -99;
            continue;
        }
        let results = grab_coords_from_form(&world_map, form.clone(), info.clone()); // It is checking all its combos even though not all of them might be available. Potential optimization?
        for target in results.entities {
            if foes.contains(&target) {scores[i] -= polarity[i]} else if allies.contains(&target) { scores[i] += polarity[i] };
        }
    }
    let (score_index, score) = scores.iter().enumerate().max_by_key(|&(_, x)| x).unwrap();
    let desired_soul = match_axiom_with_soul(score_index);
    if  score > &0  && available_souls.contains(&&desired_soul) {
        return ActionType::SoulCast { slot: available_souls.iter().position(|s| **s == desired_soul).unwrap()}
    }
    let possible_movements = get_neighbouring_entities(world_map, info.pos.0, info.pos.1);
    let momentum_pool = [(-1,0),(1,0),(0,1),(0,-1)];
    let mut choices = Vec::with_capacity(4);
    for (i, possi) in possible_movements.iter().enumerate(){
        if possi.is_none() { choices.push(momentum_pool[i]) };
    }
    let momentum = get_best_move(info.pos, destination, choices);
    if momentum.is_some(){  ActionType::Walk { momentum: momentum.unwrap() } } else { ActionType::Nothing }
}

fn calculate_actions (
    mut creatures: Query<(Entity, &mut QueuedAction, &AxiomEffects, &SoulBreath, &Position, &Species, &Faction), Without<RealityAnchor>>,
    souls: Query<&Soul>,
    read_species: Query<&Species>,
    read_position: Query<&Position>,
    mut next_state: ResMut<NextState<TurnState>>,
    player: Query<(Entity,&Position), With<RealityAnchor>>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
){
    let (_play_ent, play_pos) = if let Ok(play_ent) = player.get_single() { (play_ent.0, play_ent.1) } else { panic!("0 or 2+ players!")};
    let mut contestants = Vec::new();
    for _i in 0..5 {
        contestants.push(Vec::new());
    }
    for (entity, _queue, _ax, brea, _pos, _species, faction) in creatures.iter_mut(){
        let index = match_faction_with_index(faction);
        if index.is_some() && !brea.soulless { contestants[index.unwrap()].push(entity); } else { continue;} // Gather the pool of fighters by faction.
    }
    for (entity, mut queue, ax, brea, pos, species, faction) in creatures.iter_mut(){
        if brea.soulless {
            queue.action = ActionType::Nothing;
            continue;
        }
        let mut foes = Vec::new();
        let mut allies = Vec::new();
        let fac_index = match_faction_with_index(faction);
        if fac_index.is_some() {
            allies = contestants[fac_index.unwrap()].clone(); // Gather foes and allies of this creature's faction.
            foes = get_all_factions_except_one(&mut contestants.clone(), fac_index.unwrap());
        }
        let destination = {
            let target = foes.get(0); // Get the first foe available as a walking destination.
            if target.is_none() {
                (pos.x, pos.y)
            } else {
                let tar_pos = read_position.get(*target.unwrap());
                let tar_pos = tar_pos.unwrap();
                (tar_pos.x, tar_pos.y)
            }
        };
        let info = CasterInfo{entity, pos: (pos.x, pos.y), species: species.clone(), momentum: pos.momentum, is_player: false};
        let mut available_souls = Vec::with_capacity(4);
        for av in &brea.held {
            if let Ok(soul_type) = souls.get(*av) { available_souls.push(soul_type) };
        }
        queue.action = match species {
            Species::LunaMoth => {
                choose_action(destination, foes, allies, ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
            }
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
                choose_action(destination, foes, allies, ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
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
    mut creatures: Query<(Entity, &mut QueuedAction, &Species, &AxiomEffects, &mut SoulBreath, &mut Position, Has<RealityAnchor>)>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    souls: Query<(&mut Animator<Transform>, &Transform, &Soul), Without<Position>>,
){
    for (entity, mut queue, species, effects, breath, mut pos, is_player) in creatures.iter_mut(){
        (pos.ox, pos.oy) = (pos.x, pos.y); // To reset for the form mark animations
        if breath.soulless {queue.action = ActionType::Nothing;}
        match queue.action{
            ActionType::SoulCast { slot } => {
                let soul = match breath.held.get(slot).cloned(){ // Check that we aren't picking an empty slot.
                    Some(soul) => soul,
                    None => continue
                };
                let info = CasterInfo{ entity, pos: (pos.x,pos.y), species: species.clone(), momentum: pos.momentum, is_player};
                if let Ok((_anim, _transform, soul_id), ) = souls.get(soul) {
                    let axioms = effects.axioms.clone();
                    let (form, function) = axioms[match_soul_with_axiom(soul_id)].clone();
                    let targets = grab_coords_from_form(&world_map.entities, form, info.clone());
                    for target in targets.entities {
                        world_map.targeted_axioms.push((target, function.clone(), info.clone()));
                    }
                    for square in targets.coords{
                        world_map.anim_queue.push((entity, Animation::FormMark { coords: square }));
                    }
                } else {
                    panic!("The used Soul did not have a soul type!");
                }

                world_map.targeted_axioms.push((entity, Function::DiscardSoul { soul, slot }, info.clone()));
            }
            ActionType::Walk { momentum } => {
                world_map.targeted_axioms.push((entity, Function::Teleport { x: (pos.x as i32 + momentum.0) as usize, y: (pos.y as i32 + momentum.1) as usize }, CasterInfo{entity, pos: (pos.x, pos.y), species: species.clone(), momentum, is_player}));
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
    next_state.set(TurnState::DispensingFunctions);
}



fn dispense_functions(
    mut creatures: ParamSet<(
        Query<(&Transform, &Species, &mut SoulBreath, &mut Animator<Transform>, &mut Position, Has<RealityAnchor>)>,
        Query<&Position>,
        Query<&Species>,
    )>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &Transform, &mut TextureAtlasSprite, &mut Soul), Without<Position>>,
    ui_center: Res<CenterOfWheel>,
    time: Res<SoulRotationTimer>,
    mut zoom: ResMut<ZoomInEffect>,
){
    let mut anti_infinite_loop = 0;
    while !world_map.targeted_axioms.is_empty() {
        anti_infinite_loop += 1;
        if anti_infinite_loop > 500 { panic!("Infinite loop detected in axiom queue!") }
        let (entity, function, info) = world_map.targeted_axioms.pop().unwrap();
        if let Ok((transform_source, _species, mut breath, _anim, mut pos, is_player)) = creatures.p0().get_mut(entity.to_owned()) {
            let transform_source_trans = transform_source.translation;
            let function = function.to_owned();
            match function {
                Function::Teleport { x, y } => {
                    if !is_in_bounds(x as i32, y as i32) {continue;}
                    else if world_map.entities[xy_idx(x, y)].is_some() { // Cancel teleport if target is occupied
                        let collider = world_map.entities[xy_idx(x, y)].unwrap();
                        world_map.targeted_axioms.push((entity, Function::Collide { with: collider }, info.clone()));
                        continue;
                    }
                    let old_pos = (pos.x, pos.y);
                    let old_idx = xy_idx(pos.x, pos.y);
                    (pos.x, pos.y) = (x, y);
                    (pos.ox, pos.oy) = old_pos;
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
                        for (transform, _species, _breath, mut anim, posi, is_player) in creatures.p0().iter_mut(){
                            if is_player {
                                for (passage_coords, destination) in &world_map.warp_zones{
                                    if new_pos == *passage_coords {
                                        passage_detected = true;
                                        zoom.destination = destination.clone();
                                    }
                                }
                                continue;
                            }
                        }
                        if passage_detected{
                            world_map.anim_queue.push((entity, Animation::Passage));
                            zoom.timer.unpause();
                        }
                    }
                },
                Function::Collide { with } => {
                    let coll_species = creatures.p2().get(with).unwrap().clone();
                    let coll_pos = match creatures.p1().get(with) {
                        Ok(coll_pos_full) => (coll_pos_full.x, coll_pos_full.y),
                        Err(_) => panic!("Impossible.")
                    };
                    match coll_species {
                        Species::AxiomCrate => {
                            
                            if world_map.entities[xy_idx((coll_pos.0 as i32 + info.momentum.0) as usize, (coll_pos.1 as i32 + info.momentum.1) as usize)].is_some() {continue;}
                            world_map.targeted_axioms.push((entity, Function::MomentumDash { dist: 1 }, info.clone()));
                            world_map.targeted_axioms.push((with, Function::MomentumDash { dist: 1 }, info.clone()));
                        }
                        _ => ()
                    }
                },
                Function::StealSouls { dam } => {
                    let mut rng = rand::thread_rng();
                    let mut payload = select_random_entities(&mut breath.discard, dam, &mut rng);
                    if payload.len() < dam {
                        payload.append(&mut select_random_entities(&mut breath.pile, dam, &mut rng));
                    }
                    if payload.len() < dam {
                        while payload.len() < dam && !breath.held.is_empty(){
                            payload.push(breath.held.pop().unwrap());
                            if breath.held.is_empty() {
                                breath.soulless = true;
                                world_map.anim_queue.push((entity, Animation::Soulless));
                            }
                        }
                    }
                    
                    if let Ok((transform_culprit, _species, mut breath_culprit, _anim, _pos, _is_player)) = creatures.p0().get_mut(info.entity.to_owned()) {
                        let mut anim_output = Vec::new();
                        for soul in payload{
                            let slot = if let Ok((_anim, _transform, _sprite, soul_id), ) = souls.get(soul) { match_soul_with_display_index(soul_id) } else { panic!("A stolen soul does not exist!")};
                            breath_culprit.discard[slot].push(soul);
                            breath_culprit.soulless = false;
                            anim_output.push(soul);
                        }
                        world_map.anim_queue.push((entity, Animation::SoulDrain { source: transform_source_trans, destination: transform_culprit.translation, drained: anim_output }));
                    }

                },
                Function::RedirectSouls { dam, dest } => {
                    let new_info = if let Ok((_transform_source, species, _breath, _anim, pos, is_player)) = creatures.p0().get(dest) {
                        CasterInfo{entity: dest, pos: (pos.x, pos.y), species: species.clone(), momentum: pos.momentum, is_player}
                    } else { panic!("The RedirectSouls's destination entity does not exist!")};
                    world_map.targeted_axioms.push((entity, Function::StealSouls { dam }, new_info));
                },
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
                    world_map.targeted_axioms.push((entity, Function::Teleport { x: fx, y: fy }, info.clone()));
                },
                Function::MomentumDash { dist } => {
                    let dest = (dist as i32 * info.momentum.0, dist as i32 * info.momentum.1);
                    world_map.targeted_axioms.push((entity, Function::Dash { dx: dest.0, dy: dest.1 }, info.clone()));
                },
                Function::MomentumReverseDash { dist } => {
                    let dest = (dist as i32 * -info.momentum.0, dist as i32 * -info.momentum.1);
                    world_map.targeted_axioms.push((entity, Function::Dash { dx: dest.0, dy: dest.1 }, info.clone()));
                },
                Function::DiscardSoul { soul, slot } => {
                    if let Ok((mut anim, transform, _sprite, soul_id), ) = souls.get_mut(soul) { 
                        // Move the soul to the discard.
                        breath.discard[match_soul_with_display_index(&soul_id)].push(soul);
                        let index = breath.discard[match_soul_with_display_index(&soul_id)].iter().position(|&ent| ent == soul);
                        let final_pos = get_soul_rot_position(&soul_id, (ui_center.x, ui_center.y), true, time.timer.elapsed_secs()+0.5, index.unwrap());
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
                        let mut found_convert = None;
                        let mut harmonized = Some([Soul::Vile,Soul::Feral,Soul::Saintly,Soul::Ordered].choose(&mut rng).unwrap()); // TODO it should target slots with remaining souls only?
                        if breath.discard[0].is_empty() {harmonized = None;}
                        for j in breath.discard.iter() { // Reshuffle if no souls are left!
                            for i in j.iter(){
                                if let Ok((mut anim, transform, mut sprite, mut soul_id), ) = souls.get_mut(*i) {
                                    let index = breath.discard[match_soul_with_display_index(&soul_id)].iter().position(|&ent| ent == *i);
                                    if harmonized.is_some_and(|har| har.clone() == soul_id.clone()) {
                                        found_convert = Some((i,soul_id.clone()));
                                        *soul_id = Soul::Serene;
                                        sprite.index = match_soul_with_sprite(&soul_id);
                                        harmonized = None;
                                    }
                                    let final_pos = get_soul_rot_position(&soul_id, (ui_center.x, ui_center.y), false, time.timer.elapsed_secs()+0.5, index.unwrap());
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
                        let mut harmony_deck = breath.discard.clone();
                        if found_convert.is_some(){
                            let found_convert = found_convert.unwrap();
                            if let Some(index) = harmony_deck[match_soul_with_display_index(&found_convert.1)].iter().position(|entity| *entity == *found_convert.0) {
                                harmony_deck[match_soul_with_display_index(&found_convert.1)].remove(index);
                            }
                            harmony_deck[0].push(*found_convert.0);
                        }
                        breath.pile = harmony_deck;
                        for vec in breath.discard.iter_mut() {
                            vec.clear();
                        }
                        for (i, vec) in breath.pile.iter().enumerate() { if !vec.is_empty() {possible_indices.push(i)}}
                    }
                    let index = possible_indices.choose(&mut rng);
                    let replacement = breath.pile[index.unwrap().to_owned()].pop();
                    match replacement { // Replace the used soul.
                        Some(new_soul) => {
                            if breath.held.len() < 4 {
                                breath.held.push(new_soul);
                            }
                            else { breath.held[slot] = new_soul; }
    
                            let slot_coords_ui = [
                                ((3.*PI/4.).cos() * 1.5 +ui_center.x, (3.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((1.*PI/4.).cos() * 1.5 +ui_center.x, (1.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((5.*PI/4.).cos() * 1.5 +ui_center.x, (5.*PI/4.).sin() * 1.5 +ui_center.y),
                                ((7.*PI/4.).cos() * 1.5 +ui_center.x, (7.*PI/4.).sin() * 1.5 +ui_center.y)
                            ];
                            if let Ok((mut anim, transform, _sprite, _soul_id), ) = souls.get_mut(new_soul) { 
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
                                        end: Vec3{ x: 2., y: 2., z: 0.},
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
    world_map.anim_queue.reverse(); // I will probably forget about this and rage later
    next_state.set(TurnState::UnpackingAnimation);
}

fn unpack_animations(
    mut creatures: Query<(&SoulBreath, &mut Transform, &mut Animator<Transform>, &Position, Has<RealityAnchor>), With<Position>>,
    mut souls: Query<(&mut Animator<Transform>, &mut Visibility), (With<Soul>,Without<Position>)>,
    player: Query<&Position, With<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    time: Res<Time>,
    mut commands: Commands,
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    world_map.animation_timer.tick(time.delta());
    if !world_map.animation_timer.just_finished() {
        return;
    }
    let (player_pos, player_opos, player_trans) = if let Ok(pos) = player.get_single() {
        ((pos.x,pos.y),(pos.ox, pos.oy),Vec2::new(11., 4.)) // These hardcoded values might be dangerous
    } else {panic!("0 or 2 players!")};
    let (entity, anim_choice) = match world_map.anim_queue.pop() { // The fact that this is pop and not a loop might cause "fake" lag with a lot of queued animations
        Some(element) => element,
        None => {
            for (_breath, trans_crea, mut anim_crea, fini, is_player) in creatures.iter_mut(){
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
    if let Ok((breath, transform, mut anim, _fin, _is_player)) = creatures.get_mut(entity.to_owned()) {
        match anim_choice {
            Animation::Soulless => {
                if !breath.soulless {return;}
                let tween_rot = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(500),
                    TransformRotationLens {
                        start: transform.rotation,
                        end: Quat::from_rotation_z(PI),
                    },
                );
                anim.set_tweenable(tween_rot);
                world_map.animation_timer.set_duration(Duration::from_millis(500));
            },
            Animation::Passage => {
                for (_breath, transform, mut anim, posi, is_player) in creatures.iter_mut(){
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
                            end: Vec3::new(posi.x as f32/2.*8.-player_pos.0 as f32/2.*8.+11., posi.y as f32/2.*8.-player_pos.1 as f32/2.*8.+3.5, transform.translation.z),
                        },
                    );
                    let track = Tracks::new([tween_sc,tween_tr]);
                    anim.set_tweenable(track);
                }
                world_map.animation_timer.set_duration(Duration::from_millis(500));
            },
            Animation::SoulDrain { source, destination, mut drained } => {
                let soul = drained.pop();
                let soul = if soul.is_some() { soul.unwrap() } else { return; };
                if let Ok((mut anim, mut vis) ) = souls.get_mut(soul) { 
                    *vis = Visibility::Visible;
                    let tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(500),
                        TransformPositionLens {
                            start: source,
                            end: destination,
                        },
                    );
                    anim.set_tweenable(tween);
                }
                world_map.anim_queue.push((entity, Animation::SoulDrain { source, destination, drained: drained.clone() }));
                world_map.animation_timer.set_duration(Duration::from_millis(25));
            },
            Animation::FormMark { coords } => {
                let diff = if player_opos == player_pos {player_pos} else {player_opos};
                commands.spawn((SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: 14_usize,
                        color: Color::RED,
                        custom_size: Some(Vec2 { x: 1., y: 1. }),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(player_trans.x + (coords.0 as f32 -diff.0 as f32)/2., player_trans.y + (coords.1 as f32 -diff.1 as f32)/2., 0.),
                        scale: Vec3 { x: 0.5, y: 0.5, z: 1. },
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle.handle.clone(),
                    ..default()
                }, EffectMarker));
                world_map.animation_timer.set_duration(Duration::from_millis(25));
            }
        }
    }
}

fn fade_effects(
    mut effects: Query<(Entity, &mut TextureAtlasSprite), With<EffectMarker>>,
    mut commands: Commands,
) {
    for (ent, mut eff) in effects.iter_mut() {
        let alpha = eff.color.a()-0.1;
        if alpha <= 0.01 {
            commands.entity(ent).despawn();
        } else {
            eff.color.set_a(alpha);
        }
    }
}