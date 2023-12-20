use std::time::Duration;

use crate::{components::{Position, QueuedAction, SoulBreath, AxiomEffects}, SpriteSheetHandle, input::ActionType, axiom::{Form, Function}};
use bevy::prelude::*;
use bevy_tweening::{*, lens::TransformPositionLens};
use std::f32::consts::PI;

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
    breath: SoulBreath,
    axioms: AxiomEffects,
}

impl CreatureBundle { // Creatures displayed on screen.
    pub fn new(
        tex_handle: &SpriteSheetHandle
    ) -> Self {
        let texture_atlas_handle = &tex_handle.handle;
        let mut lots_of_vec = Vec::new();
        for _i in 0..5 {lots_of_vec.push(Vec::new())}
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
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 0., y: 0., z: 0.0},
                    scale: Vec3{x: 0.5, y: 0.5, z:1.},
                    
                    ..default()
                },
                ..default()
            },
            animation: Animator::new(tween),
            name: Name::new("Bugged Creature"),
            species: Species::BuggedSpecies,
            position: Position { x: 0, y: 0, ox: 0, oy: 0, momentum: (-1, 0)},
            action: QueuedAction { action: ActionType::Nothing},
            breath: SoulBreath { pile: lots_of_vec.clone(), discard: lots_of_vec.clone(), held: Vec::new(), },
            axioms: AxiomEffects { axioms: vec![
                (Form::MomentumBeam, Function::StealSouls { dam: 10 }),
                (Form::MomentumBeam, Function::StealSouls { dam: 10 }),
                (Form::MomentumBeam, Function::Empty),
                (Form::MomentumBeam, Function::Empty),
            ], polarity: vec![-1,-1,-1,-1] }
        }
    }
    pub fn with_data(
        mut self,
        x: usize,
        y: usize,
        offset: (f32, f32),
        species: Species,
    ) -> Self{
        self = self.with_species(species);
        self = self.with_position(x, y, offset);
        self
    }
    pub fn with_position(mut self, x: usize, y: usize, offset: (f32, f32)) -> Self {
        self.position.x = x;
        self.position.y = y;
        let end = Vec3::new(self.position.x as f32/2. + offset.0, self.position.y as f32/2. + offset.1, self.sprite_bundle.transform.translation.z);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1),
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
        if is_invisible(&species){
            self.sprite_bundle.visibility = Visibility::Hidden;
        }
        (self.axioms.axioms, self.axioms.polarity) = match_species_with_axioms(&species);
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

#[derive(Component, PartialEq, Clone, Debug)]
pub enum Species {
    Wall,
    Terminal,
    BuggedSpecies,
    Void,
    Projector,
    Felidol,
    TermiWall,
    RiftBorder{dir: usize},
    EpsilonHead,
    EpsilonTail {order: usize},
    LunaMoth,
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
        Species::RiftBorder { dir: _ } => 108,
        Species::Projector => 2,
        Species::EpsilonHead => 67,
        Species::EpsilonTail { order: _ } => 68,
        Species::LunaMoth => 44,
    }
}

pub fn match_species_with_name(
    species: &Species
)-> &'static str {
    match species{
        Species::Wall => "Rampart of Nacre",
        Species::BuggedSpecies => "Bugged, Please Report",
        Species::Terminal => "Terminal, the Reality Anchor",
        Species::Felidol => "Greedswept Felidol",
        Species::Void => "Bugged, Please Report",
        Species::TermiWall => "Tangled Circuits",
        Species::RiftBorder { dir: _ } => "Thought-Matter Rift",
        Species::Projector => "Hypnotic Well",
        Species::EpsilonHead => "Epsilon, Adorned in Jade",
        Species::EpsilonTail { order: _ } => "Rubberized Mecha-Segment",
        Species::LunaMoth => "Cosmos Worn as Robes",
    }
}

pub fn match_species_with_axioms(
    species: &Species
) -> (Vec<(Form, Function)>,Vec<i32>) {
    match species{
        Species::LunaMoth => (vec![
            (Form::Ego, Function::MomentumDash { dist: 5 }),
            (Form::MomentumTouch, Function::StealSouls { dam: 4 }),
            (Form::MomentumBeam, Function::MomentumReverseDash { dist: 5 }),
            (Form::MomentumBeam, Function::MomentumReverseDash { dist: 5 }), // Circlet slash, pull closer?
        ], vec![1,-2,-1,-1] ),
        Species::EpsilonHead => (vec![
            (Form::MomentumBeam, Function::StealSouls { dam: 4 }),
            (Form::MomentumBeam, Function::StealSouls { dam: 4 }),
            (Form::MomentumBeam, Function::Empty),
            (Form::MomentumBeam, Function::Empty), // Circlet slash, pull closer?
        ], vec![-1,-1,-1,-1] ),
        _ => (vec![
            (Form::Empty, Function::Empty),
            (Form::Empty, Function::Empty),
            (Form::Empty, Function::Empty),
            (Form::Empty, Function::Empty),
        ], vec![0,0,0,0] ),
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
        Species::RiftBorder { dir } => Quat::from_rotation_z((PI/2.)*(*dir as f32)),
        _ => Quat::from_rotation_z(0.)
    }
}

pub fn is_intangible(
    species: &Species
) -> bool{
    match species{
        Species::RiftBorder { dir: _ } => true,
        Species::Projector => true,
        _ => false
    }
}

pub fn is_invisible(
    species: &Species
) -> bool {
    match species {
        Species::Void => true,
        Species::Projector => true,
        _ => false,
    }
}