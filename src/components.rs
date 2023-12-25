use bevy::prelude::*;

use crate::{species::Species, input::ActionType, axiom::{Form, Function}, soul::Soul};

#[derive(Component)]
pub struct RealityAnchor {
    pub player_id: usize,
}

#[derive(Component)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub ox: usize, //old positions (last turn)
    pub oy: usize,
    pub momentum: (i32, i32),
}

#[derive(Component)]
pub struct BuildQueue {
    pub build_queue: Vec<(Species, (usize, usize))>
}

#[derive(Component)]
pub struct Faith {
    pub faith: usize,
}

#[derive(Component)]
pub struct RightFaith;

// Add this component to a creature to have it not interact with the world and be pass-through.
#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct EffectMarker;

#[derive(Component)]
pub struct MomentumMarker{
    pub dir: (i32, i32),
}

#[derive(Component)]
pub struct LogIndex{
    pub index: usize,
    pub going_to: f32,
}

#[derive(Component, PartialEq)]
pub enum Faction{
    Saintly,
    Feral,
    Vile,
    Serene,
    Ordered,
    Unaligned,
}

#[derive(Component)]
pub struct MinimapTile{
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct QueuedAction{
    pub action: ActionType,
}

#[derive(Component)]
pub struct SoulBreath{
    pub pile: Vec<Vec<Entity>>,
    pub held: Vec<Entity>,
    pub discard: Vec<Vec<Entity>>,
    pub soulless: bool,
}

#[derive(Component)]
pub struct AxiomEffects{
    pub axioms: Vec<(Form, Function)>,
    pub polarity: Vec<i32>,
}