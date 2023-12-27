use bevy::ecs::entity::Entity;

use crate::{soul::Soul, species::Species, map::{get_entity_at_coords, bresenham_line, is_in_bounds, xy_idx}};

#[derive(Clone, Debug)]
pub struct Effect {
    pub stacks: usize,
    pub effect_type: EffectType,
}

#[derive(Clone, Debug)]
pub enum EffectType {
    Glamour, // ++ casting, -- deal dmg // Your soul, a droplet, drowning in an ocean of endless lives.
    Pride, // ++ deal dmg, -- take dmg // Us, standing on towers of gold so high and bright they burn away all doubt. You, so, so below, in a pit of submission so hidden one wonders how we even noticed your existence.
    Discipline, // ++ take dmg, -- move // Maximize pleasure. Minimize pain. Maximize reproduction. Minimize solitude. Your flesh is one of carbon, yet your soul mimicks silicon.
    Grace, // ++ move, -- casting // You ran without thought or reason, pursued in a meadow where each blade of grass had been turned to a steel knife, until none was left but blood.
    Possession {link: Entity},
}

#[derive(Clone)]
pub enum Form {
    Empty,
    Ego,
    MomentumBeam,
    MomentumTail,
    MomentumTouch,
}
#[derive(Clone, Debug)]
pub enum Function {
    Empty,
    Dash { dx: i32, dy: i32 }, // Position is incremented by dx and dy, but stops when it hits an edge or a creature.
    Teleport { x: usize, y: usize }, // 
    MomentumDash { dist: usize },
    MomentumReverseDash { dist: usize },
    DiscardSoul { soul: Entity, slot: usize },
    StealSouls { dam: usize },
    SwapAnchor,
    RedirectSouls { dam: usize, dest: Entity},
    Collide {with: Entity},
    MessageLog {message_id: usize},
    ApplyEffect { effect: Effect },
    PossessCreature {duration: usize},
    MomentumSlamDash {dist: usize},
    MeleeSlam {dist: usize},
}

pub fn match_soul_with_axiom(
    soul: &Soul
) -> usize {
    match soul {
        Soul::Feral => 2,
        Soul::Ordered => 1,
        Soul::Saintly => 0,
        Soul::Vile => 3,
        Soul::Serene => 0, // Temporary, imitates saintly
    }
}

pub fn match_axiom_with_soul(
    axiom: usize
) -> Soul {
    match axiom {
        2 => Soul::Feral,
        1 => Soul::Ordered,
        0 => Soul::Saintly,
        3 => Soul::Vile,
        _ => Soul::Serene
    }
}

pub struct ReturnedForm{
    pub entities: Vec<Entity>,
    pub coords: Vec<(usize,usize)>,
}

#[derive(Clone, Debug)]
pub struct CasterInfo{
    pub entity: Entity,
    pub pos: (usize,usize),
    pub species: Species,
    pub momentum: (i32,i32),
    pub is_player: bool,
}

impl CasterInfo{
    pub fn placeholder(
    ) -> CasterInfo {
        CasterInfo { entity: Entity::PLACEHOLDER, pos: (0,0), species: Species::BuggedSpecies, momentum: (0,1), is_player: false }
    }
}

pub fn grab_coords_from_form( // vec in vec for better, synchronized animations?
    map: &[Option<Entity>],
    form: Form,
    caster: CasterInfo,
) -> ReturnedForm {
    let coords = match form {
        Form::Empty => Vec::new(),
        Form::Ego => vec![caster.pos],
        Form::MomentumBeam => blocked_beam(tup_usize_to_i32(caster.pos), (caster.pos.0 as i32+ caster.momentum.0*45, caster.pos.1 as i32+ caster.momentum.1*45), map),
        Form::MomentumTail => vec![tup_i32_to_usize((tup_usize_to_i32(caster.pos).0-caster.momentum.0, tup_usize_to_i32(caster.pos).1-caster.momentum.1))],
        Form::MomentumTouch => vec![tup_i32_to_usize((tup_usize_to_i32(caster.pos).0+caster.momentum.0, tup_usize_to_i32(caster.pos).1+caster.momentum.1))],
    };
    let mut entities = Vec::with_capacity(coords.len());
    for (x,y) in &coords {
        match get_entity_at_coords(map, *x, *y) {
            Some(ent) => entities.push(ent),
            None => continue,
        }
    }
    ReturnedForm { entities, coords } 
}

fn blocked_beam(
    start: (i32,i32),
    end: (i32, i32),
    map: &[Option<Entity>],
) -> Vec<(usize, usize)> {
    let mut line = bresenham_line(start.0 as i32, start.1 as i32, end.0, end.1);
    line.remove(0);
    let mut stop_point = 0;
    for (i, (nx, ny)) in line.iter().enumerate() {
        if is_in_bounds(*nx, *ny){
            if map[xy_idx(*nx as usize,* ny as usize)].is_some() {
                stop_point = i+1;
                break;
            }
            else {continue;}
        } else {
            stop_point = i;
            break;
        }
    }

    line.drain(stop_point..);

    let mut output = Vec::new();
    for tu in line{
        output.push(tup_i32_to_usize(tu))
    }
    output
}

pub fn tup_usize_to_i32(
    tuple: (usize, usize)
) -> (i32, i32) {
    (tuple.0 as i32, tuple.1 as i32)
}

pub fn tup_i32_to_usize(
    tuple: (i32, i32)
) -> (usize, usize) {
    (tuple.0 as usize, tuple.1 as usize)
}