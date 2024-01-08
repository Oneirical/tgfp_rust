use bevy::prelude::*;

use crate::axiom::{Effect, EffectType};

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {

    }
}

pub fn has_effect(
    effects: &Vec<Effect>,
    search: EffectType,
) -> Option<Effect> {
    for i in effects {
        if i.effect_type == search {
            return Some(i.clone());
        }
    }
    None
}