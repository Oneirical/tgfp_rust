use std::{time::Duration, f32::consts::PI, mem::swap};

use bevy::prelude::*;
use bevy_tweening::{*, lens::{TransformPositionLens, TransformScaleLens, TransformRotationLens}};
use rand::seq::SliceRandom;

use crate::{components::{QueuedAction, RealityAnchor, Position, SoulBreath, AxiomEffects, EffectMarker, Faction, Wounded, Thought, Segmentified, DoorAnimation}, input::ActionType, TurnState, map::{xy_idx, WorldMap, is_in_bounds, bresenham_line, get_neighbouring_entities, get_best_move, get_all_factions_except_one, get_astar_best_move, manhattan_distance, pathfind_to_location}, soul::{Soul, get_soul_rot_position, SoulRotationTimer, match_soul_with_display_index, match_soul_with_sprite, select_random_entities, CurrentEntityInUI}, ui::{CenterOfWheel, LogMessage}, axiom::{grab_coords_from_form, CasterInfo, match_soul_with_axiom, Function, Form, match_axiom_with_soul, Effect, EffectType, match_effect_with_decay, TriggerType, reduce_down_to, match_effect_with_minimum, match_effect_with_gain, tup_usize_to_i32, tup_i32_to_usize}, species::{Species, match_faction_with_index, match_species_with_priority, match_species_with_sprite, is_pushable, is_openable, CreatureBundle, is_grab_point, is_intangible}, ZoomInEffect, SpriteSheetHandle, ai::has_effect, vaults::{Vault, get_build_sequence}};

pub struct TurnPlugin;

#[derive(Debug, PartialEq, Clone)]
pub enum Animation{
    Passage,
    SoulDrain {source: Vec3, destination: Vec3, drained: Vec<Entity>},
    FormMark {coords: (usize, usize)},
    Soulless,
    MessagePrint,
    SoulSwap,
    Polymorph {new_species: Species},
    RevealCreature,
    UseDoor {orient: usize, closing: bool},
    RemoveDoorAnims {closed: bool},
    MinimumDelay,
}

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_actions.run_if(in_state(TurnState::CalculatingResponse)));
        app.add_systems(Update, execute_turn.run_if(in_state(TurnState::ExecutingTurn)));
        app.add_systems(Update, dispense_functions.run_if(in_state(TurnState::DispensingFunctions)));
        app.add_systems(Update, unpack_animations.run_if(in_state(TurnState::UnpackingAnimation)));
        app.add_systems(Update, fade_effects);
        app.insert_resource(TurnCount{turns: 0});
    }
}

#[derive(Resource)]
pub struct TurnCount {
    pub turns: usize,
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
    let mut scores = [0,0,0,0];
    for (i, (form, _function)) in axioms.iter().enumerate() {
        if !available_souls.contains(&&match_axiom_with_soul(i)) {
            scores[i] = -99;
            continue;
        }
        let results = grab_coords_from_form(world_map, form.clone(), info.clone()); // It is checking all its combos even though not all of them might be available. Potential optimization?
        for target in results.entities {
            if foes.contains(&target) {scores[i] -= polarity[i]} else if allies.contains(&target) { scores[i] += polarity[i] };
        }
    }
    match info.species {
        Species::EpsilonHead { len: _ } => {
            if has_effect(&info.effects, EffectType::Meltdown).is_some() {
                scores[3] = 99;
            }
        }
        _ => ()
    }
    let (score_index, score) = scores.iter().enumerate().max_by_key(|&(_, x)| x).unwrap();
    let desired_soul = match_axiom_with_soul(score_index);
    if  score > &0  && available_souls.contains(&&desired_soul) {
        return ActionType::SoulCast { slot: available_souls.iter().position(|s| **s == desired_soul).unwrap()}
    }
    let momentum = get_astar_best_move(info.pos, destination, world_map);
    if let Some(momentum) = momentum {  ActionType::Walk { momentum } } else { ActionType::Nothing }
}

fn calculate_actions (
    mut creatures: Query<(Entity, &mut QueuedAction, &AxiomEffects, &SoulBreath, &Position, &Species, &Faction, Has<RealityAnchor>)>,
    read_species: Query<&Species>,
    read_position: Query<&Position>,
    read_thought: Query<&Thought>,
    locate_wounded: Query<(Entity, &Position), With<Wounded>>,
    locate_segments: Query<Entity, With<Segmentified>>,
    souls: Query<&Soul>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
    mut turn_count: ResMut<TurnCount>,
){
    turn_count.turns += 1;
    let mut contestants = Vec::new();
    for _i in 0..5 {
        contestants.push(Vec::new());
    }
    for (entity, _queue, _ax, brea, _pos, _species, faction, _is_player) in creatures.iter_mut(){
        let index = match_faction_with_index(faction);
        if index.is_some() && !brea.soulless { contestants[index.unwrap()].push(entity); } else { continue;} // Gather the pool of fighters by faction.
    }
    for (entity, mut queue, ax, brea, pos, species, faction, is_player) in creatures.iter_mut(){
        if brea.soulless && !matches!(species, &Species::EpsilonTail { .. }){ // this caused a weird bug with the segments
            queue.action = ActionType::Nothing;
        }
        let mut foes = Vec::new();
        let mut allies = Vec::new();
        let fac_index = match_faction_with_index(faction);
        if fac_index.is_some() {
            allies = contestants[fac_index.unwrap()].clone(); // Gather foes and allies of this creature's faction.
            foes = get_all_factions_except_one(&mut contestants.clone(), fac_index.unwrap());
        }
        let destination = {
            let target = if foes.is_empty() { None } else {Some(foes[0])}; // Get the first foe available as a walking destination.
            let target = if matches!(species, Species::EpsilonHead { .. }) && target.is_none() {
                let mut located_seg = Vec::new();
                for segment in locate_segments.iter() {
                    located_seg.push(segment);
                }
                let mut tail_detect = None;
                for tail in located_seg {
                    if matches!(read_species.get(tail).unwrap(), Species::EpsilonTail { order: -1 }) {tail_detect = Some(tail); break;} else {continue;}
                }
                tail_detect
            } else {target};
            if target.is_none() {
                (22, 22)
            } else {
                let tar_pos = read_position.get(target.unwrap());
                let tar_pos = tar_pos.unwrap();
                (tar_pos.x, tar_pos.y)
            }
        };
        let (glamour, discipline, grace, pride) = (ax.status[0].stacks, ax.status[1].stacks,ax.status[2].stacks,ax.status[3].stacks);
        let info = CasterInfo{entity, pos: (pos.x, pos.y), species: species.clone(), momentum: pos.momentum, is_player, glamour, grace, discipline, pride, effects: ax.status.clone() };
        let mut available_souls = Vec::with_capacity(4);
        for av in &brea.held {
            if let Ok(soul_type) = souls.get(*av) { available_souls.push(soul_type) };
        }
        let saved_play_action = if is_player { queue.action.clone() } else { ActionType::Nothing };
        queue.action = match species {
            Species::SegmentTransformer => {
                let mut located_wounded = Vec::new();
                for wound in locate_wounded.iter() {
                    located_wounded.push(wound);
                }
                located_wounded.sort_by(|a, b| {
                    let dist_a = manhattan_distance(tup_usize_to_i32((a.1.x, a.1.y)), tup_usize_to_i32(info.pos));
                    let dist_b = manhattan_distance(tup_usize_to_i32((b.1.x, b.1.y)), tup_usize_to_i32(info.pos));
                    dist_a.cmp(&dist_b)
                });
                let grab_this = located_wounded.get(0);
                if let Some(grab_this) = grab_this {  
                    choose_action(info.pos, vec![grab_this.0], Vec::new(), ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
                } else {ActionType::Nothing}            }
            Species::ChromeNurse => {
                let mut patient = None;
                for search in &info.effects {
                    match search.effect_type {
                        EffectType::AssignedPatient { link } => patient = Some(link),
                        _ => continue
                    }
                }
                match patient {
                    Some(found) => {
                        let patient_pos = read_position.get(found).unwrap();
                        let mut patient_next_dest = match &read_thought.get(found).unwrap().stored_path {
                            Some(seq) => {
                                seq.0.clone().pop()
                            }
                            None => {
                                None
                            }
                        };
                        let path_accepted = match patient_next_dest {
                            Some(can) => {
                                world_map.entities[xy_idx(can.0 as usize, can.1 as usize)].is_none()
                            }
                            None => false
                        };

                        if !path_accepted {
                            let mut new_path = pathfind_to_location((patient_pos.x, patient_pos.y), (36, 8), &world_map.entities);
                            if let Some(mut rev) = new_path { 
                                rev.0.reverse();
                                rev.0.pop();
                                patient_next_dest = if !rev.0.is_empty() {
                                    Some(rev.0[rev.0.len()-1]) } else { None };
                                new_path = Some((rev.0, rev.1));} else { new_path = None};
                            commands.entity(found).insert(Thought{stored_path: new_path});
                        }


                        let patient_next_move = match patient_next_dest {
                            None => None,
                            Some(dest) => {
                                if tup_i32_to_usize(dest) == info.pos {
                                    None
                                } else {
                                    Some((dest.0-patient_pos.x as i32, dest.1-patient_pos.y as i32))
                                }
                            }
                        };
                        match patient_next_move {
                            Some(momen) => {
                                let push_zone = (patient_pos.x as i32 - momen.0, patient_pos.y as i32 - momen.1);
                                if tup_usize_to_i32(info.pos) == push_zone {
                                    ActionType::Walk { momentum: momen }
                                } else {
                                    let momentum = get_astar_best_move(info.pos, tup_i32_to_usize(push_zone), &world_map.entities);
                                    if let Some(momentum) = momentum {  ActionType::Walk { momentum } } else { ActionType::Nothing }
                                }
                            }
                            None => ActionType::Nothing
                        }
                    }
                    None => {
                        let mut located_wounded = Vec::new();
                        for wound in locate_wounded.iter() {
                            located_wounded.push(wound);
                        }
                        located_wounded.sort_by(|a, b| {
                            let dist_a = manhattan_distance(tup_usize_to_i32((a.1.x, a.1.y)), tup_usize_to_i32(info.pos));
                            let dist_b = manhattan_distance(tup_usize_to_i32((b.1.x, b.1.y)), tup_usize_to_i32(info.pos));
                            dist_a.cmp(&dist_b)
                        });
                        let grab_this = located_wounded.get(0);
                        if let Some(grab_this) = grab_this {  
                            choose_action((grab_this.1.x, grab_this.1.y), foes, vec![grab_this.0], ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
                        } else {ActionType::Nothing}
                    }
                }
            }
            Species::LunaMoth => {
                choose_action(destination, foes, allies, ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
            }
            Species::EpsilonHead { len } => {
                let mut current_order = 0;
                let mut self_pos = (pos.x, pos.y);
                let mut self_mom = pos.momentum;
                let mut detected_tails = Vec::new();
                let mut num_nei = 0;
                loop {
                    let neigh = get_neighbouring_entities(&world_map.entities, self_pos.0, self_pos.1);
                    if self_pos == (pos.x, pos.y) {
                        for i in &neigh {
                            if i.is_some() {
                                num_nei += 1;
                            }
                        }
                    }
                    let mut found_segment = false;
                    for detected in neigh{
                        match detected {
                            Some(creature) => {
                                match read_species.get(creature).unwrap() {
                                    Species::EpsilonTail {order: _} => {
                                        let crea_pos = read_position.get(creature).unwrap();
                                        let momentum = (self_pos.0 as i32-crea_pos.x as i32, self_pos.1 as i32-crea_pos.y as i32);

                                        if (momentum == self_mom || current_order >= *len ) && !detected_tails.contains(&creature){
                                            self_pos = (crea_pos.x, crea_pos.y);
                                            self_mom = crea_pos.momentum;
                                            commands.entity(creature).insert(QueuedAction{action: ActionType::Walk { momentum }});
                                            found_segment = true;
                                            detected_tails.push(creature);
                                            commands.entity(creature).insert(Species::EpsilonTail { order: current_order as i32 });
                                            current_order += 1;
                                            break;
                                        } else {continue;}
                                    },
                                    _ => continue,
                                };
                            }
                            None => continue,
                        };
                    }
                    if !found_segment {break;}
                }
                if num_nei >= 4 {
                    world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: 2, effect_type: EffectType::Meltdown}}, info.clone()));
                }
                commands.entity(entity).insert(Species::EpsilonHead { len: current_order });
                choose_action(destination, foes, allies, ax.axioms.clone(), ax.polarity.clone(), available_souls, info, &world_map.entities)
            },
            Species::EpsilonTail {order: _} => {
                queue.action.clone()
            },
            _ => ActionType::Nothing,
        };
        if is_player { queue.action = saved_play_action; }
    }

    next_state.set(TurnState::ExecutingTurn);
}

fn execute_turn (
    mut creatures: Query<(Entity, &QueuedAction, &Species, &mut AxiomEffects, &mut SoulBreath, &mut Position, Has<RealityAnchor>)>,
    read_action: Query<&QueuedAction>,
    read_species: Query<&Species>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    souls: Query<(&mut Animator<Transform>, &Transform, &Soul), Without<Position>>,
    turn_count: Res<TurnCount>,
){
    if turn_count.turns%10 == 1 {
        //world_map.targeted_axioms.push((play_ent, Function::MessageLog { message_id: turn_count.turns/10 }, CasterInfo::placeholder()));
    }
    for (entity, queue, species, mut effects, breath, mut pos, is_player) in creatures.iter_mut(){
        if is_player {
            world_map.anim_reality_anchor = entity;
        }
        (pos.ox, pos.oy) = (pos.x, pos.y); // To reset for the form mark animations
        let mut chosen_action = queue.action.clone();
        if breath.soulless && !matches!(species, &Species::EpsilonTail { .. }) {chosen_action = ActionType::Nothing;}
        let (glamour, discipline, grace, pride) = (effects.status[0].stacks, effects.status[1].stacks,effects.status[2].stacks,effects.status[3].stacks);
        let mut info = CasterInfo{ entity, pos: (pos.x,pos.y), species: species.clone(), momentum: pos.momentum, is_player, glamour, grace, discipline, pride, effects: effects.status.clone()};
        for eff in effects.status.iter() {
            match eff.effect_type {
                EffectType::Sync { link } => {
                    let overwrite = read_action.get(link).map(|e| (e.action.clone())).unwrap();
                    chosen_action = overwrite.clone();
                    break;
                }
                _ => ()
            }
        }
        let adj = get_neighbouring_entities(&world_map.entities, pos.x, pos.y);
        let mut supported = false;
        for pot in adj {
            if let Some(tile) = pot { 
                let sp = read_species.get(tile).unwrap();
                if !is_intangible(sp) {supported = true;}
            }
        }
        match chosen_action{
            ActionType::SoulCast { slot } => {
                let soul = match breath.held.get(slot).cloned(){ // Check that we aren't picking an empty slot.
                    Some(soul) => soul,
                    None => continue
                };
                if let Ok((_anim, _transform, soul_id), ) = souls.get(soul) {
                    let axioms = effects.axioms.clone();
                    let (form, function) = axioms[match_soul_with_axiom(soul_id)].clone();
                    let targets = grab_coords_from_form(&world_map.entities, form, info.clone());
                    for target in targets.entities {
                        world_map.targeted_axioms.push((target, function.clone(), info.clone()));
                    }
                    for square in targets.coords{
                        world_map.anim_queue.push((entity, Animation::FormMark { coords: square }));
                        world_map.floor_axioms.push((square, function.clone(), info.clone()));
                    }
                } else {
                    panic!("The used Soul did not have a soul type!");
                }

                world_map.targeted_axioms.push((entity, Function::DiscardSoul { soul, slot }, info.clone()));

                // CASTING
                // ++Glamour
                // --Grace
                world_map.targeted_axioms.push((entity, Function::TriggerEffect { trig: TriggerType::CastSoul }, info.clone()));
            }
            ActionType::Walk { momentum } => {
                if supported || species != &Species::Terminal{
                    world_map.targeted_axioms.push((entity, Function::Dash {dx: momentum.0, dy: momentum.1}, info.clone()));
                    if species == &Species::Terminal {
                        world_map.targeted_axioms.push((entity, Function::AlterMomentum {alter: momentum}, info.clone()));
                        if (momentum.1).signum() + (info.momentum.1).signum() == 0{
                            world_map.targeted_axioms.push((entity, Function::ResetVertical, info.clone()));
                        }
                        if (momentum.0).signum() + (info.momentum.0).signum() == 0{
                            world_map.targeted_axioms.push((entity, Function::ResetHorizontal, info.clone()));
                        }
                    }
                    else {pos.momentum = momentum;}
                }
            },
            ActionType::Nothing => ()
        };
        if !supported && species == &Species::Terminal{
            world_map.targeted_axioms.push((entity, Function::Dash {dx: info.momentum.0, dy: info.momentum.1}, info.clone()));
            world_map.targeted_axioms.push((entity, Function::AlterMomentum {alter: (0,-1)}, info.clone()));
        }
        if effects.status.len() > 4 {
            for eff in effects.status.iter_mut() {
                if match_effect_with_decay(&eff.effect_type) == TriggerType::EachTurn || match_effect_with_gain(&eff.effect_type) == TriggerType::EachTurn { // If at least one turn-decay effect, tick them
                    world_map.targeted_axioms.push((entity, Function::TriggerEffect { trig: TriggerType::EachTurn }, info.clone()));
                    break;
                }
            }
        }
    }
    next_state.set(TurnState::DispensingFunctions);
}



fn dispense_functions(
    mut creatures: ParamSet<(
        Query<(&Transform, &mut Species, &mut SoulBreath, &mut AxiomEffects, &mut Animator<Transform>, &mut Position, Has<RealityAnchor>)>,
        Query<&Position>,
        Query<&Species>,
        Query<&SoulBreath>,
        Query<(&Position, &Transform), With<RealityAnchor>>,
    )>,
    faction: Query<&Faction>,
    check_wound: Query<Entity, With<Wounded>>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &Transform, &mut TextureAtlasSprite, &mut Soul), Without<Position>>,
    ui_center: Res<CenterOfWheel>,
    time: Res<SoulRotationTimer>,
    mut events: EventWriter<LogMessage>,
    mut zoom: ResMut<ZoomInEffect>,
    mut commands: Commands,
    mut current_crea_display: ResMut<CurrentEntityInUI>,
    texture_atlas_handle: Res<SpriteSheetHandle>,

){
    let mut anti_infinite_loop = 0;
    /*world_map.targeted_axioms.sort_by(|a, b| { // 
        match (a.2.is_player, b.2.is_player) {
            (true, true) | (false, false) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        }
    }); // The player acts first */
    world_map.floor_axioms.sort_unstable_by(|a, b| match_species_with_priority(&b.2.species).cmp(&match_species_with_priority(&a.2.species)));
    world_map.targeted_axioms.sort_unstable_by(|a, b| match_species_with_priority(&b.2.species).cmp(&match_species_with_priority(&a.2.species)));
    // Then we grant special priorities

    while !world_map.floor_axioms.is_empty() {
        anti_infinite_loop += 1;
        if anti_infinite_loop > 500 { panic!("Infinite loop detected in axiom queue!") }
        let (coords, function, mut info) = world_map.floor_axioms.pop().unwrap();
        match function {
            Function::SummonCreature { species } => {
                let mut player_pos = (0.,0.);
                let mut player_trans = (0., 0.);
                if let Ok((pos, tra)) = creatures.p4().get_single() {
                    player_pos = ((22. - pos.x as f32)/2., (8. - pos.y as f32)/2.);
                    player_trans = (tra.translation.x, tra.translation.y);
                }
                if world_map.entities[xy_idx(coords.0, coords.1)].is_some() {continue;}
                let new_creature = CreatureBundle::new(&texture_atlas_handle)
                .with_data(coords.0, coords.1, player_pos, Some(player_trans), species.clone());
                let entity_id = commands.spawn(new_creature).id();
                //commands.entity(entity_id).insert(Visibility::Hidden);
                //world_map.anim_queue.push((entity_id, Animation::RevealCreature));
            }
            _ => ()
        }
    }

    while !world_map.targeted_axioms.is_empty() {
        anti_infinite_loop += 1;
        if anti_infinite_loop > 500 { panic!("Infinite loop detected in axiom queue!") }
        let (entity, function, mut info) = world_map.targeted_axioms.pop().unwrap();
        if let Ok((transform_source, mut species, mut breath, mut effects, _anim, mut pos, is_player)) = creatures.p0().get_mut(entity.to_owned()) {
            //let (glamour, discipline, grace, pride) = (effects.status[0].stacks, effects.status[1].stacks,effects.status[2].stacks,effects.status[3].stacks);
            assert_eq!(effects.status[0].effect_type, EffectType::Glamour);
            assert_eq!(effects.status[1].effect_type, EffectType::Discipline);
            assert_eq!(effects.status[2].effect_type, EffectType::Grace);
            assert_eq!(effects.status[3].effect_type, EffectType::Pride);
            let transform_source_trans = transform_source.translation;
            let function = function.to_owned();
            match function {
                Function::Teleport { x, y } => {
                    if !is_in_bounds(x as i32, y as i32) || world_map.entities[xy_idx(x, y)].is_some() {continue;}
                    //else if world_map.entities[xy_idx(x, y)].is_some() { // Cancel teleport if target is occupied
                        //let collider = world_map.entities[xy_idx(x, y)].unwrap();
                        //world_map.targeted_axioms.push((entity, Function::Collide { with: collider }, info.clone()));
                        //continue;
                    //}
                    let old_pos = (pos.x, pos.y);
                    let old_idx = xy_idx(pos.x, pos.y);
                    (pos.x, pos.y) = (x, y);
                    (pos.ox, pos.oy) = old_pos;
                    let new_pos = (x, y);
                    let dest = (pos.x as i32 -old_pos.0 as i32, pos.y as i32-old_pos.1 as i32);
                    let idx = xy_idx(pos.x, pos.y);
                    world_map.entities.swap(old_idx, idx);

                    // MOVING
                    // ++Grace
                    // --Discipline
                    world_map.targeted_axioms.push((entity, Function::TriggerEffect { trig: TriggerType::Move }, info.clone()));

                    let max = dest.0.abs().max(dest.1.abs());
                    assert!(!(dest.0 == 0 && dest.1 == 0));
                    /*pos.momentum = if max == dest.0.abs(){ // Reassign the new momentum.
                        (dest.0/dest.0.abs(), 0)
                    } else {
                        (0, dest.1/dest.1.abs())
                    };*/

                    //if anim.tweenable().progress() != 1.0 { continue; }
                    if !is_player {
                        continue;
                    }
                    else {
                        let mut triggered = false;
                        if is_player {
                            for (passage_coords, destination) in &world_map.warp_zones{
                                if new_pos == *passage_coords {
                                    zoom.timer.unpause();
                                    zoom.destination = destination.clone();
                                    triggered = true;
                                    break;
                                }
                            }
                        }
                        if triggered {world_map.anim_queue.push((entity, Animation::Passage))};

                    }
                },
                Function::AlterMomentum { alter } => {
                    pos.momentum.0 += alter.0;
                    pos.momentum.1 += alter.1;
                }
                Function::ResetHorizontal => {
                    pos.momentum.0 = 0;
                }
                Function::ResetVertical => {
                    pos.momentum.1 = 0;
                }
                Function::FlatStealSouls { dam } => {
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
                                commands.entity(entity).insert(Wounded);
                                world_map.anim_queue.push((entity, Animation::Soulless));
                            }
                        }
                    }

                    // TAKING DAMAGE
                    // ++Discipline
                    // --Pride
                    world_map.targeted_axioms.push((entity, Function::TriggerEffect { trig: TriggerType::TakeDamage }, info.clone()));

                    // DEALING DAMAGE
                    // ++Pride
                    // --Glamour
                    world_map.targeted_axioms.push((info.entity, Function::TriggerEffect { trig: TriggerType::DealDamage }, info.clone()));

                    
                    if let Ok((transform_culprit, _species, mut breath_culprit, _ax, _anim, _pos, _is_player)) = creatures.p0().get_mut(info.entity.to_owned()) {
                        let mut anim_output = Vec::new();
                        for soul in payload{
                            let slot = if let Ok((_anim, _transform, _sprite, soul_id), ) = souls.get(soul) { match_soul_with_display_index(soul_id) } else { panic!("A stolen soul does not exist!")};
                            breath_culprit.discard[slot].push(soul);
                            breath_culprit.soulless = false;
                            commands.entity(info.entity).remove::<Wounded>();
                            anim_output.push(soul);
                        }
                        world_map.anim_queue.push((entity, Animation::SoulDrain { source: transform_source_trans, destination: transform_culprit.translation, drained: anim_output }));
                    }

                },
                Function::TriggerEffect { trig } => {
                    let mut remove_these_effects = Vec::new();
                    for (i, eff) in effects.status.iter_mut().enumerate() {
                        if match_effect_with_decay(&eff.effect_type) == trig {
                            eff.stacks = reduce_down_to(match_effect_with_minimum(&eff.effect_type), eff.stacks, 1);
                        }
                        if match_effect_with_gain(&eff.effect_type) == trig {
                            eff.stacks += 1;
                        }
                        if eff.stacks == 0 {
                            match &eff.effect_type {
                                EffectType::Possession { link } => {
                                    world_map.targeted_axioms.push((*link, Function::SwapAnchor, info.clone()));
                                },
                                EffectType::Polymorph { original } => {
                                    world_map.targeted_axioms.push((entity, Function::PolymorphNow { new_species: original.clone() }, info.clone()));
                                }
                                EffectType::Charm { original } => {
                                    commands.entity(entity).insert(original.clone());
                                }
                                EffectType::OpenDoor => {
                                    match world_map.entities[xy_idx(pos.x, pos.y)] {
                                        Some(_) => {
                                            world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: 1, effect_type: EffectType::OpenDoor}}, info.clone()));
                                        }
                                        None => {
                                            world_map.targeted_axioms.push((entity, Function::BecomeTangible, info.clone()));
                                            match *species {
                                                Species::Airlock { dir } => world_map.anim_queue.push((entity, Animation::UseDoor { orient: dir, closing: true })),
                                                _ => ()
                                            }
                                        }
                                    }

                                }
                                _ => (),
                            }
                            remove_these_effects.push(i);
                        }
                        if eff.effect_type == EffectType::Meltdown && eff.stacks > 9 {
                            world_map.targeted_axioms.push((entity, Function::BlinkOuter, info.clone()));
                            remove_these_effects.push(i);
                        }
                    }
                    for i in remove_these_effects{
                        effects.status.remove(i);
                    }
                }
                Function::ApplyEffect { effect } => {
                    let mut found_same_effect = false;
                    for eff in effects.status.iter_mut() {
                        if eff.effect_type == effect.effect_type {
                            eff.stacks += effect.stacks;
                            found_same_effect = true;
                            break;
                        }
                    }
                    if !found_same_effect {
                        effects.status.push(effect);
                    }
                },
                Function::StealSouls => {
                    world_map.targeted_axioms.push((entity, Function::FlatStealSouls { dam: info.pride }, info.clone()));
                }
                Function::PossessCreature => {
                    let duration = 999;//info.glamour;
                    world_map.targeted_axioms.push((entity, Function::SwapAnchor, info.clone()));
                    world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: duration, effect_type: EffectType::Possession { link: info.entity }}}, info.clone()));
                }
                Function::Synchronize => {
                    let duration = 10;//info.grace;
                    world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: duration, effect_type: EffectType::Sync { link: info.entity }}}, info.clone()));
                }
                Function::Charm {dur}=> {
                    let fac = faction.get(entity).unwrap();
                    let new_fac = faction.get(info.entity).unwrap();
                    world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: dur, effect_type: EffectType::Charm { original: fac.clone()}}}, info.clone()));
                    commands.entity(entity).insert(new_fac.clone());
                }
                Function::MarkPatient => {
                    world_map.targeted_axioms.push((entity, Function::MomentumReverseDash, info.clone()));
                    world_map.targeted_axioms.push((info.entity, Function::ApplyEffect { effect: Effect { stacks: 99, effect_type: EffectType::AssignedPatient { link: entity } } }, info.clone()));
                }
                Function::InjectCaste {num, caste} => {
                    let mut payload = Vec::with_capacity(num);
                    let slot = match_soul_with_display_index(&caste);
                    let mut origin_translation = Vec3::ZERO;

                    if let Ok((transform_culprit, _species, mut breath_culprit, _ax, _anim, _pos, _is_player)) = creatures.p0().get_mut(info.entity.to_owned()) {
                        origin_translation = transform_culprit.translation;
                        while payload.len() < num {
                            if !breath_culprit.discard[slot].is_empty() {
                                let soul = breath_culprit.discard[slot].pop().unwrap();
                                payload.push(soul);
                            } else if !breath_culprit.pile[slot].is_empty() {
                                let soul = breath_culprit.pile[slot].pop().unwrap();
                                payload.push(soul);
                            } else if !breath_culprit.held.is_empty() {
                                let soul = breath_culprit.held.pop().unwrap();
                                let soul_type = if let Ok((_anim, _transform, _sprite, soul_id), ) = souls.get(soul) {soul_id } else { panic!("A stolen soul does not exist!")};
                                if soul_type == &caste {
                                    payload.push(soul);
                                } else {
                                    breath_culprit.held.push(soul);
                                }
                            } else {
                                breath_culprit.soulless = true;
                                commands.entity(info.entity).insert(Wounded);
                                break;
                            }
                        }
                    }
                    if let Ok((transform_receiver, _species, mut breath_receiver, _ax, _anim, _pos, _is_player)) = creatures.p0().get_mut(entity) {
                        let mut anim_output = Vec::new();
                        for i in payload {
                            breath_receiver.discard[slot].push(i);
                            anim_output.push(i);
                            breath_receiver.soulless = false;
                            commands.entity(entity).remove::<Wounded>();
                        }
                        world_map.anim_queue.push((entity, Animation::SoulDrain { source: origin_translation, destination: transform_receiver.translation, drained: anim_output }));
                    }

                    // TAKING DAMAGE
                    // ++Discipline
                    // --Pride
                    world_map.targeted_axioms.push((info.entity, Function::TriggerEffect { trig: TriggerType::TakeDamage }, info.clone()));

                    // DEALING DAMAGE
                    // ++Pride
                    // --Glamour
                    world_map.targeted_axioms.push((entity, Function::TriggerEffect { trig: TriggerType::DealDamage }, info.clone()));

                }
                Function::CyanCharm => {
                    let dur = 10;//info.pride;
                    world_map.targeted_axioms.push((entity, Function::InjectCaste {num: 1, caste: Soul::Serene}, info.clone()));
                    world_map.targeted_axioms.push((entity, Function::Charm {dur}, info.clone()));
                }
                Function::Segmentize => {
                    world_map.targeted_axioms.push((entity, Function::PolymorphNow { new_species: Species::EpsilonTail { order: -1 } }, info.clone()));
                    commands.entity(entity).remove::<Wounded>();
                    commands.entity(entity).insert(Segmentified);
                }
                Function::ImitateSpecies => {
                    let duration = info.grace;
                    world_map.targeted_axioms.push((info.entity, Function::PolymorphNow { new_species: species.clone() }, info.clone()));
                    world_map.targeted_axioms.push((info.entity, Function::ApplyEffect { effect: Effect {stacks: duration, effect_type: EffectType::Polymorph { original: info.species.clone() }}}, info.clone()));
                }
                Function::SwapSpecies => {
                    let duration = info.grace;
                    world_map.targeted_axioms.push((info.entity, Function::PolymorphNow { new_species: species.clone() }, info.clone()));
                    world_map.targeted_axioms.push((info.entity, Function::ApplyEffect { effect: Effect {stacks: duration, effect_type: EffectType::Polymorph { original: info.species.clone() }}}, info.clone()));
                    world_map.targeted_axioms.push((entity, Function::PolymorphNow { new_species: info.species.clone() }, info.clone()));
                    world_map.targeted_axioms.push((entity, Function::ApplyEffect { effect: Effect {stacks: duration, effect_type: EffectType::Polymorph { original: species.clone() }}}, info.clone()));
                }
                Function::PolymorphNow { new_species } => {
                    *species = new_species.clone();
                    world_map.anim_queue.push((entity, Animation::Polymorph {new_species}));
                }
                Function::BlinkOuter => {
                    let dests = grab_coords_from_form(&world_map.entities, Form::BigOuter, info.clone());
                    for target in dests.coords {
                        if world_map.entities[xy_idx(target.0, target.1)].is_none() {
                            world_map.targeted_axioms.push((entity, Function::Teleport { x: target.0, y: target.1 }, info.clone()));
                            break;
                        }
                    }
                }
                Function::MomentumDash => {
                    world_map.targeted_axioms.push((entity, Function::FlatMomentumDash { dist: info.grace }, info.clone()));
                }
                Function::Collide { with } => { // with is the entity you hit with your move
                    let coll_species = creatures.p2().get(with).unwrap().clone();
                    let coll_pos = creatures.p1().get(with).map(|e| (e.x, e.y)).unwrap();
                    let wound = check_wound.get(with);
                    if is_pushable(&coll_species) || wound.is_ok() {
                        if world_map.entities[xy_idx((coll_pos.0 as i32 + info.momentum.0) as usize, (coll_pos.1 as i32 + info.momentum.1) as usize)].is_some() {continue;}
                        world_map.targeted_axioms.push((entity, Function::FlatMomentumDash { dist: 1 }, info.clone()));
                        world_map.targeted_axioms.push((with, Function::FlatMomentumDash { dist: 1 }, info.clone()));
                    }
                    if is_openable(&coll_species) {
                        world_map.targeted_axioms.push((with, Function::BecomeIntangible, info.clone()));
                        world_map.targeted_axioms.push((with, Function::ApplyEffect { effect: Effect {stacks: 3, effect_type: EffectType::OpenDoor}}, info.clone()));
                        match &coll_species {
                            Species::Airlock { dir } => {
                                world_map.anim_queue.push((with, Animation::UseDoor { orient: *dir, closing: false }));
                                let corner = {
                                    let curr = ((coll_pos.0/9)*9, (coll_pos.1/9)*9);
                                    let dix = [(0,-9),(9,0),(0,9),(-9,0)];
                                    tup_i32_to_usize(((curr.0 as i32 + dix[*dir].0), (curr.1 as i32+ dix[*dir].1)))
                                };
                                let builds = get_build_sequence(Vault::WorldSeed, corner);
                                for bui in builds {
                                    world_map.floor_axioms.push((bui.1, Function::SummonCreature { species: bui.0 }, info.clone()));
                                }
                            },
                            _ => ()
                        }
                    }
                },
                Function::BecomeIntangible => {
                    let idx = xy_idx(pos.x, pos.y);
                    world_map.entities[idx] = None;
                }
                Function::BecomeTangible => {
                    let idx = xy_idx(pos.x, pos.y);
                    world_map.entities[idx] = Some(entity);
                }
                Function::MessageLog { message_id } => {
                    events.send(LogMessage(message_id));
                    world_map.anim_queue.push((entity, Animation::MessagePrint));
                }
                Function::SwapAnchor => {
                    if !is_player {
                        if let Ok((_transform_culprit, _species, _breath_culprit, _ax,  _anim, _pos, is_player_cul)) = creatures.p0().get_mut(info.entity.to_owned()) {
                            if is_player_cul{
                                commands.entity(info.entity).remove::<RealityAnchor>();
                                commands.entity(entity).insert(RealityAnchor{player_id: 0});
                                world_map.anim_queue.push((entity, Animation::SoulSwap));
                                current_crea_display.entity = entity;
                            }
                        }
                    } else {
                        commands.entity(entity).remove::<RealityAnchor>();
                        commands.entity(info.entity).insert(RealityAnchor{player_id: 0});
                        world_map.anim_queue.push((entity, Animation::SoulSwap));
                        current_crea_display.entity = info.entity;
                    }
                }
                Function::Coil => {
                    let atk_pos = creatures.p1().get(info.entity).map(|e| (e.x, e.y)).unwrap();
                    let adj = get_neighbouring_entities(&world_map.entities, atk_pos.0, atk_pos.1);
                    let count = adj.iter().filter(|&x| x.is_some()).count();
                    world_map.targeted_axioms.push((entity, Function::FlatStealSouls { dam: info.pride*count }, info.clone()));

                }
                Function::RedirectSouls { dam, dest } => {
                    let new_info = if let Ok((_transform_source, species, _breath, ax, _anim, pos, is_player)) = creatures.p0().get(dest) {
                        let (glamour, discipline, grace, pride) = (ax.status[0].stacks, ax.status[1].stacks,ax.status[2].stacks,ax.status[3].stacks);
                        CasterInfo{entity: dest, pos: (pos.x, pos.y), species: species.clone(), momentum: pos.momentum, is_player, glamour, grace, discipline, pride, effects: ax.status.clone()}
                    } else { panic!("The RedirectSouls's destination entity does not exist!")};
                    world_map.targeted_axioms.push((entity, Function::FlatStealSouls { dam }, new_info));
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
                                let collider = world_map.entities[xy_idx(nx as usize, ny as usize)].unwrap();
                                world_map.targeted_axioms.push((entity, Function::Collide { with: collider }, info.clone()));
                                break;
                            }
                            else {(fx, fy) = (nx as usize, ny as usize)}
                        } else {
                            break;
                        }
                    }
                    world_map.targeted_axioms.push((entity, Function::Teleport { x: fx, y: fy }, info.clone()));
                },
                Function::FlatMomentumDash { dist } => {
                    let dest = (dist as i32 * info.momentum.0, dist as i32 * info.momentum.1);
                    world_map.targeted_axioms.push((entity, Function::Dash { dx: dest.0, dy: dest.1 }, info.clone()));
                },
                Function::MomentumSlamDash { dist } => {
                    world_map.targeted_axioms.push((entity, Function::MeleeSlam { dist }, info.clone()));
                    world_map.targeted_axioms.push((entity, Function::FlatMomentumDash { dist }, info.clone()));
                },
                Function::MeleeSlam { dist } => {
                    let coll_pos = creatures.p1().get(info.entity).map(|e| (e.x, e.y)).unwrap();
                    info.pos = coll_pos;
                    let targets = grab_coords_from_form(&world_map.entities, Form::MomentumTouch, info.clone());
                    for target in targets.entities {
                        world_map.targeted_axioms.push((target, Function::FlatMomentumDash { dist }, info.clone()));
                    }
                }
                Function::MomentumReverseDash => {
                    let dist = info.grace;
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
                _ => panic!("Unknown Function used!")
            };
            //let new_stats = &effects.status.clone();

        }
    }
    world_map.anim_queue.reverse(); // I will probably forget about this and rage later
    //if world_map.anim_queue.is_empty() {world_map.anim_queue.push((Entity::PLACEHOLDER, Animation::MinimumDelay))};
    next_state.set(TurnState::UnpackingAnimation);
}

fn unpack_animations(
    mut creatures: Query<(&SoulBreath, &mut Transform, &mut TextureAtlasSprite, &mut Animator<Transform>, &Position, Has<RealityAnchor>), With<Position>>,
    mut souls: Query<(&mut Animator<Transform>, &mut Visibility), (With<Soul>,Without<Position>)>,
    player: Query<&Position>,
    new_player: Query<Entity, With<RealityAnchor>>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    time: Res<Time>,
    mut commands: Commands,
    door_anims: Query<Entity, With<DoorAnimation>>,
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    world_map.animation_timer.tick(time.delta());
    if !world_map.animation_timer.just_finished() {
        return;
    }
    let (player_pos, player_opos, player_trans) = if let Ok(pos) = player.get(world_map.anim_reality_anchor) {
        ((pos.x,pos.y),(pos.ox, pos.oy),Vec2::new(11., 4.)) // These hardcoded values might be dangerous
    } else {panic!("0 or 2 players!")};
    let (entity, anim_choice) = match world_map.anim_queue.pop() { // The fact that this is pop and not a loop might cause "fake" lag with a lot of queued animations
        Some(element) => element,
        None => {
            for (_breath, trans_crea, _sprite, mut anim_crea, fini, _is_player) in creatures.iter_mut(){
                let end = Vec3::new(player_trans.x + (fini.x as f32 -player_pos.0 as f32)/2., player_trans.y + (fini.y as f32 -player_pos.1 as f32)/2., 0.);
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
    if matches!(anim_choice, Animation::MinimumDelay) {world_map.animation_timer.set_duration(Duration::from_millis(20));}
    else if let Ok((breath, transform, mut sprite, mut anim, fini, _is_player)) = creatures.get_mut(entity.to_owned()) {
        match anim_choice {
            Animation::UseDoor { orient, closing } => {
                if !closing {commands.entity(entity).insert(Visibility::Hidden);}
                let dx = [0.5, 0., 0.5, 0.];
                let dy = [0., 0.5, 0., 0.5];
                let mut sx = player_trans.x + (fini.x as f32 -player_pos.0 as f32)/2.;
                let mut sy = player_trans.y + (fini.y as f32 -player_pos.1 as f32)/2.;
                let (mut start_a, mut start_b) = (Vec3::new(sx, sy, -2.),Vec3::new(sx, sy, -2.));
                let (mut end_a, mut end_b) = (Vec3::new(sx + dx[orient], sy + dy[orient], -2.),Vec3::new(sx - dx[orient], sy - dy[orient], -2.));
                if closing {
                    sx = player_trans.x + (fini.x as f32 -player_opos.0 as f32)/2.;
                    sy = player_trans.y + (fini.y as f32 -player_opos.1 as f32)/2.;
                    (end_a, end_b) = (Vec3::new(sx, sy, -2.),Vec3::new(sx, sy, -2.));
                    (start_a, start_b) = (Vec3::new(sx + dx[orient], sy + dy[orient], -2.),Vec3::new(sx - dx[orient], sy - dy[orient], -2.));
                }
                let tween_a = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(500),
                    TransformPositionLens {
                        start: start_a,
                        end: end_a,
                    },
                );
                let tween_b = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(500),
                    TransformPositionLens {
                        start: start_b,
                        end: end_b,
                    },
                );
                commands.spawn((SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: sprite.index,
                        custom_size: Some(Vec2 { x: 0.5, y: 0.5 }),
                        ..default()
                    },
                    transform: Transform {
                        translation: start_a,
                        rotation: transform.rotation,
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle.handle.clone(),
                    ..default()
                },DoorAnimation, Animator::new(tween_a)));
                commands.spawn((SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: sprite.index,
                        custom_size: Some(Vec2 { x: 0.5, y: 0.5 }),
                        ..default()
                    },
                    transform: Transform {
                        translation: start_b,
                        rotation: transform.rotation,
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle.handle.clone(),
                    ..default()
                }, DoorAnimation, Animator::new(tween_b)));
                world_map.animation_timer.set_duration(Duration::from_millis(500));
                world_map.anim_queue.push((entity, Animation::RemoveDoorAnims {closed: closing}));

            }
            Animation::RemoveDoorAnims {closed} => {
                for door in door_anims.iter() {
                    commands.entity(door).despawn();
                }
                if closed {
                    commands.entity(entity).insert(Visibility::Visible);
                }
                world_map.animation_timer.set_duration(Duration::from_millis(1));
            }
            Animation::Polymorph {new_species}=> {
                sprite.index = match_species_with_sprite(&new_species);
            }
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
            Animation::SoulSwap => {
                world_map.anim_reality_anchor = if let Ok(ent) =  new_player.get_single() { ent } else { panic!("0 or 2+ players!")};
                world_map.animation_timer.set_duration(Duration::from_millis(1));
            }
            Animation::Passage => {
                for (_breath, transform, _sprite, mut anim, posi, is_player) in creatures.iter_mut(){
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
                let soul = if let Some(soul) = soul { soul } else { return; };
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
            },
            Animation::MessagePrint => {
                world_map.animation_timer.set_duration(Duration::from_millis(1));
            },
            Animation::RevealCreature => {
                commands.entity(entity).insert(Visibility::Visible);
            },
            Animation::MinimumDelay => {
                panic!("You should never reach this arm!");
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