use bevy::prelude::*;

use crate::species::Species;

#[derive(Component, Clone)]
pub struct RealityAnchor {
    pub player_id: usize,
}

#[derive(Component, Clone)]
pub struct CreatureID {
    pub creature_id: usize,
}

#[derive(Component, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Component, Clone)]
pub struct BuildQueue {
    pub queue: Vec<(Species, (usize, usize))>
}