use bevy::prelude::*;

use crate::{axiom::{Effect, Form, Function, PlantAxiom}, input::ActionType, soul::Soul, species::Species};

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
pub struct Thought {
    pub stored_path: Option<(Vec<(i32, i32)>, u32)>
}

#[derive(Component)]
pub struct SoulSlot {
    pub slot: Option<Soul>,
}

#[derive(Component)]
pub struct Plant {
    pub stem: Vec<Entity>,
    pub program: Vec<Vec<PlantAxiom>>,
    pub sequences: Vec<Vec<Soul>>,
}

#[derive(Component)]
pub struct BuildQueue {
    pub build_queue: Vec<(Species, (usize, usize))>
}

#[derive(Component)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

// Add this component to a creature to have it not interact with the world and be pass-through.
#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct DoorAnimation;

#[derive(Component)]
pub struct EffectMarker;

#[derive(Component)]
pub struct CreatureDescription;

#[derive(Component)]
pub struct Wounded;

#[derive(Component)]
pub struct Segmentified;

#[derive(Component)]
pub struct MomentumMarker{
    pub dir: (i32, i32),
}

#[derive(Component)]
pub struct LogIndex{
    pub index: usize,
    pub going_to: f32,
}

#[derive(Component)]
pub struct EffectTracker{
    pub tracking_index: usize,
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Debug)]
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
    pub status: Vec<Effect>,
}