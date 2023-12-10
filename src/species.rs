use std::time::Duration;

use crate::{components::{Position, QueuedAction, SoulBreath}, SpriteSheetHandle, input::ActionType, axiom::{Form, Function}};
use bevy::prelude::*;
use bevy_tweening::{*, lens::TransformPositionLens};
use std::f32::consts::PI;

#[derive(Component, PartialEq, Clone, Debug)]
pub enum Species {
    Wall,
    Terminal,
    BuggedSpecies,
    Void,
    Felidol,
    TermiWall,
    HypnoWell{dir: usize},
}

pub enum MapColour {
    White,
    Plum,
}

#[derive(Bundle)]
pub struct CreatureBundle {
    sprite_bundle: SpriteSheetBundle,
    animation: Animator<Transform>,
    name: Name,
    species: Species,
    position: Position,
    action: QueuedAction,
    breath: SoulBreath
}

impl CreatureBundle { // Creatures displayed on screen.
    pub fn new(
        tex_handle: &SpriteSheetHandle
    ) -> Self {
        let texture_atlas_handle = &tex_handle.handle;
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ZERO,
            },
        );
        Self{
            sprite_bundle : SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                sprite: TextureAtlasSprite{
                    index : 0_usize,
                    custom_size: Some(Vec2::new(0.5, 0.5)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 0., y: 0., z: 0.0},
                    
                    ..default()
                },
                ..default()
            },
            animation: Animator::new(tween),
            name: Name::new("Bugged Creature"),
            species: Species::BuggedSpecies,
            position: Position { x: 0, y: 0, momentum: (-1, 0)},
            action: QueuedAction { action: ActionType::Nothing},
            breath: SoulBreath { pile: Vec::new(), held: Vec::new(), discard: Vec::new(), axioms: vec![
                (Form::Ego, Function::LinearDash { dist: 0 }),
                (Form::Empty, Function::Empty),
                (Form::Empty, Function::Empty),
                (Form::Empty, Function::Empty),
            ]}
        }
    }
    pub fn with_data(
        mut self,
        x: usize,
        y: usize,
        species: Species,
    ) -> Self{
        self = self.with_species(species);
        self = self.with_position(x, y);
        self
    }
    pub fn with_position(mut self, x: usize, y: usize) -> Self {
        self.position.x = x;
        self.position.y = y;
        let end = Vec3::new(self.position.x as f32/2., self.position.y as f32/2., self.sprite_bundle.transform.translation.z);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: end,
                end
            },
        );
        self.animation.set_tweenable(tween);
        self
    }
    pub fn with_species(mut self, species: Species) -> Self {
        self.sprite_bundle.sprite.index = match_species_with_sprite(&species);
        self.name = Name::new(match_species_with_name(&species));
        self.sprite_bundle.transform.rotation = match_species_with_rotation(&species);
        if is_intangible(&species){
            self.sprite_bundle.transform.translation.z = -0.1;
        }
        self.species = species;
        self
    }
    pub fn with_anim_source(mut self, x: usize, y: usize) -> Self{ // Should always be called after with_position.
        let end = Vec3::new(self.position.x as f32, self.position.y as f32, 0.);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: Vec3::new(x as f32, y as f32, 0.),
                end
            },
        );
        self.animation = Animator::new(tween);
        self
    }
}

pub fn match_species_with_sprite(
    species: &Species
)-> usize{
    match species{
        Species::Wall => 3,
        Species::BuggedSpecies => 1,
        Species::Terminal => 0,
        Species::Void => 2,
        Species::Felidol => 49,
        Species::TermiWall => 37,
        Species::HypnoWell { dir: _ } => 108
    }
}

pub fn match_species_with_name(
    species: &Species
)-> &'static str {
    match species{
        Species::Wall => "Rampart of Nacre",
        Species::BuggedSpecies => "Bugged, Please Report",
        Species::Terminal => "Terminal",
        Species::Felidol => "Greedswept Felidol",
        Species::Void => "A Bugged Void",
        Species::TermiWall => "Tangled Circuits",
        Species::HypnoWell { dir: _ } => "Thought-Matter Rift",
    }
}

pub fn match_species_with_pixel(
    species: &Species
) -> usize {
    match_color_with_pixel(&match_species_with_color(species))
}

fn match_species_with_color(
    species: &Species
) -> MapColour {
    match species{
        Species::Terminal => MapColour::Plum,
        _ => MapColour::White
    }
}

fn match_color_with_pixel(
    color: &MapColour
) -> usize{
    match color{
        MapColour::Plum => 109,
        MapColour::White => 106,
    }
}

pub fn match_species_with_rotation(
    species: &Species
) -> Quat{
    match species{
        Species::HypnoWell { dir } => Quat::from_rotation_z((PI/2.)*(*dir as f32)),
        _ => Quat::from_rotation_z(0.)
    }
}

pub fn is_intangible(
    species: &Species
) -> bool{
    match species{
        Species::HypnoWell { dir: _ } => true,
        _ => false
    }
}