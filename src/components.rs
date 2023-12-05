use bevy::prelude::*;

use crate::species::Species;

#[derive(Component)]
pub struct RealityAnchor {
    pub player_id: usize,
}

#[derive(Component)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct BuildQueue {
    pub build_queue: Vec<(Species, (usize, usize))>
}

#[derive(Component, Reflect)]
pub struct UIElement {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Faith {
    pub faith: usize,
}

#[derive(Component)]
pub struct RightFaith;

#[derive(Component)]
pub struct FaithPoint{
    pub num: usize,
}