use bevy::ecs::entity::Entity;

use crate::{soul::Soul, species::Species, map::get_entity_at_coords};

pub enum Effect {
    Glamour {stacks : usize},
    Pride {stacks : usize},
    Discipline {stacks : usize},
    Grace {stacks: usize},
}
#[derive(Clone)]
pub enum Form {
    Empty,
    Ego,
}
#[derive(Clone, Debug)]
pub enum Function {
    Empty,
    Dash { dx: i32, dy: i32 }, // Position is incremented by dx and dy, but stops when it hits an edge or a creature.
    Teleport { x: usize, y: usize }, // 
    LinearDash { dist: usize },
    DiscardSoul { soul: Entity, slot: usize },
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

pub struct ReturnedForm{
    pub entities: Vec<Entity>,
    pub coords: Vec<(usize,usize)>,
}

#[derive(Clone, Debug)]
pub struct CasterInfo{
    pub pos: (usize,usize),
    pub species: Species,
    pub momentum: (i32,i32),
    pub is_player: bool,
}

pub fn grab_coords_from_form( // vec in vec for better, synchronized animations?
    map: &[Option<Entity>],
    form: Form,
    caster: CasterInfo,
) -> ReturnedForm {
    let coords = match form {
        Form::Empty => Vec::new(),
        Form::Ego => vec![caster.pos],
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