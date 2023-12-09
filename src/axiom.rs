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
#[derive(Clone)]
pub enum Function {
    Empty,
    Dash { pow: usize },
    DiscardSlot { slot: usize },
}

pub fn match_soul_with_axiom(
    soul: &Soul
) -> usize {
    match soul {
        Soul::Feral => 2,
        Soul::Ordered => 1,
        Soul::Saintly => 0,
        Soul::Vile => 3,
        Soul::Serene => 0, // Temporary
    }
}

pub struct ReturnedForm{
    pub entities: Vec<Entity>,
    pub coords: Vec<(usize,usize)>,
}

pub struct CasterInfo{
    pub pos: (usize,usize),
    pub species: Species,
}

pub fn grab_coords_from_form( // vec in vec for better, synchronized animations?
    map: &Vec<Option<Entity>>,
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