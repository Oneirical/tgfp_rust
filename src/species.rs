use std::time::Duration;

use crate::{components::{Position, QueuedAction, SoulBreath, AxiomEffects, Faction, Thought}, SpriteSheetHandle, input::ActionType, axiom::{Form, Function, Effect, EffectType, match_form_with_name}};
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
    faction: Faction,
    thought: Thought,
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
            breath: SoulBreath { pile: lots_of_vec.clone(), discard: lots_of_vec.clone(), held: Vec::new(), soulless: false},
            faction: Faction::Unaligned,
            axioms: AxiomEffects { axioms: vec![
                (Form::MomentumBeam, Function::FlatStealSouls { dam: 10 }),
                (Form::MomentumBeam, Function::FlatStealSouls { dam: 10 }),
                (Form::MomentumBeam, Function::Empty),
                (Form::MomentumBeam, Function::Empty),
            ], polarity: vec![-1,-1,-1,-1], status: vec![
                Effect{stacks: 1, effect_type: EffectType::Glamour},
                Effect{stacks: 1, effect_type: EffectType::Discipline},
                Effect{stacks: 1, effect_type: EffectType::Grace},
                Effect{stacks: 1, effect_type: EffectType::Pride},
            ]},
            thought: Thought {stored_path: None},
        }
    }
    pub fn with_data(
        mut self,
        x: usize,
        y: usize,
        offset: (f32, f32),
        start_anim: Option<(f32, f32)>,
        species: Species,
    ) -> Self{
        self = self.with_species(species);
        self = self.with_position(x, y, offset, start_anim);
        self
    }
    pub fn with_position(mut self, x: usize, y: usize, offset: (f32, f32), start_anim: Option<(f32, f32)>) -> Self {
        self.position.x = x;
        self.position.y = y;
        let end = Vec3::new(self.position.x as f32/2. + offset.0, self.position.y as f32/2. + offset.1, self.sprite_bundle.transform.translation.z);
        let start = match start_anim {
            None => end,
            Some((sx, sy)) => Vec3 { x: sx, y: sy, z: end.z}
        };
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(500),
            TransformPositionLens {
                start,
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
        self.faction = match_species_with_faction(&species);
        self.species = species;
        self
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Species {
    Wall,
    Terminal,
    BuggedSpecies,
    Void,
    Projector,
    Felidol,
    TermiWall,
    RiftBorder{dir: usize},
    EpsilonHead {len: usize},
    EpsilonTail{order: i32},
    LunaMoth,
    AxiomCrate,
    GlamourCrate,
    DisciplineCrate,
    GraceCrate,
    PrideCrate,
    Harmonizer,
    Airlock {dir: usize},
    ChromeNurse,
    SegmentTransformer,
    CrateActivator {caste: usize},
    FormCrate {form: Form},
    FunctionCrate {function: Box<Function>},
    Platform,
    Ladder,
    PlantSegment,
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
        Species::EpsilonHead {len: _} => 67,
        Species::EpsilonTail {order: _} => 68,
        Species::LunaMoth => 44,
        Species::AxiomCrate => 19,
        Species::Harmonizer => 26,
        Species::GlamourCrate => 19,
        Species::DisciplineCrate => 20,
        Species::GraceCrate => 21,
        Species::PrideCrate => 22,
        Species::Airlock {dir: _ } => 17,
        Species::ChromeNurse => 6,
        Species::SegmentTransformer => 78,
        Species::CrateActivator { caste } => 160+caste,
        Species::FormCrate { form: _ } => 20,
        Species::FunctionCrate { function: _ } => 21,
        Species::Platform => 57,
        Species::Ladder => 58,
        Species::PlantSegment => 43,
    }
}

pub fn match_species_with_faction(
    species: &Species
) -> Faction {
    match species {
        Species::LunaMoth => Faction::Feral,
        Species::EpsilonHead { len: _ }=> Faction::Ordered,
        Species::EpsilonTail {order: _}=> Faction::Ordered,
        Species::Terminal => Faction::Ordered,
        Species::SegmentTransformer => Faction::Ordered,
        _ => Faction::Unaligned,
    }
}

pub fn match_species_with_name(
    species: &Species
)-> String {
    let ret = match species{
        Species::Wall => "Rampart of Nacre",
        Species::BuggedSpecies => "Bugged, Please Report",
        Species::Terminal => "Terminal, the Reality Anchor",
        Species::Felidol => "Greedswept Felidol",
        Species::Void => "Bugged, Please Report",
        Species::TermiWall => "Tangled Circuits",
        Species::RiftBorder { dir: _ } => "Thought-Matter Rift",
        Species::Projector => "Hypnotic Well",
        Species::EpsilonHead{ len: _ } => "Epsilon, Adorned in Jade",
        Species::EpsilonTail {order: _}=> "Rubberized Mecha-Segment",
        Species::LunaMoth => "Cosmos Worn as Robes",
        Species::AxiomCrate => "Axiomatic Crate",
        Species::GlamourCrate => "Adorned Crate",
        Species::DisciplineCrate => "Steel-Plated Crate",
        Species::GraceCrate => "Vibrating Crate",
        Species::PrideCrate => "Tar-Soaked Crate",
        Species::Harmonizer => "Harmonic Organizer",
        Species::Airlock {dir: _ } => "Quicksilver Curtains",
        Species::ChromeNurse => "Chrome Attendant",
        Species::SegmentTransformer => "Bio-Mechanizer",
        Species::CrateActivator { caste: _ } => "Axiom Activator",
        Species::FormCrate { form: _ } => "Form Crate",
        Species::FunctionCrate { function: _ } => "Function Crate",
        Species::Platform => "Pneumatic Platform",
        Species::Ladder => "Ascendant Gust",
        Species::PlantSegment => "World Stem",
    }.to_owned();
    ret
}

pub fn match_species_with_priority(
    species: &Species
) -> i32 {
    match species{
        Species::EpsilonHead { len: _ } => -2,
        Species::Airlock { dir: _ } => -99,
        Species::EpsilonTail { order } => *order,
        _ => 0,
    }
}

pub fn match_species_with_axioms(
    species: &Species
) -> (Vec<(Form, Function)>,Vec<i32>) {
    match species{
        Species::LunaMoth => (vec![
            (Form::Ego, Function::MomentumDash),
            (Form::MomentumTouch, Function::StealSouls),
            (Form::MomentumBeam, Function::MomentumReverseDash),
            (Form::MomentumBeam, Function::MomentumReverseDash), // Circlet slash, pull closer?
        ], vec![1,-2,-1,-1] ),
        Species::EpsilonHead { len: _ } => (vec![
            (Form::MomentumBeam, Function::MomentumReverseDash),
            (Form::MomentumLateral, Function::Coil),
            (Form::MomentumBeam, Function::StealSouls),
            (Form::SmallBurst, Function::BlinkOuter),
        ], vec![-1,-1,-1,0] ),
        Species::Terminal => (vec![
            (Form::MomentumBeam, Function::SummonCreature { species: Species::ChromeNurse }),
            (Form::MomentumBeam, Function::MomentumReverseDash),
            (Form::SmallBurst, Function::Synchronize),
            (Form::SmallBurst, Function::CyanCharm), // TODO there is an infinite loop, fix it
        ], vec![0,0,0,0]),
        Species::ChromeNurse => (vec![
            (Form::MomentumBeam, Function::MarkPatient),
            (Form::MomentumBeam, Function::MarkPatient),
            (Form::MomentumBeam, Function::MarkPatient),
            (Form::Empty, Function::Empty),
        ], vec![1,1,1,0]),
        Species::SegmentTransformer => (vec![
            (Form::MomentumTouch, Function::Segmentize),
            (Form::MomentumTouch, Function::Segmentize),
            (Form::MomentumTouch, Function::Segmentize),
            (Form::Empty, Function::Empty),
        ], vec![-1,-1,-1,0]),
        Species::FormCrate { form } => (vec![
            (form.clone(), Function::Empty),
            (form.clone(), Function::Empty),
            (form.clone(), Function::Empty),
            (form.clone(), Function::Empty),
        ], vec![0,0,0,0] ),
        Species::FunctionCrate { function } => (vec![
            (Form::Empty, *function.clone()),
            (Form::Empty, *function.clone()),
            (Form::Empty, *function.clone()),
            (Form::Empty, *function.clone()),
        ], vec![0,0,0,0] ),
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
        Species::Airlock { dir } => Quat::from_rotation_z((PI/2.)*(*dir as f32)),
        _ => Quat::from_rotation_z(0.)
    }
}

pub fn is_intangible(
    species: &Species
) -> bool{
    match species{
        Species::RiftBorder { dir: _ } => true,
        Species::Projector => true,
        //Species::Platform => true,
        Species::Ladder => true,
        _ => false
    }
}

pub fn is_grab_point(
    species: &Species
) -> bool{
    match species{
        Species::Platform => true,
        Species::Ladder => true,
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

pub fn is_openable(
    species: &Species
) -> bool {
    match species {
        Species::Airlock { dir: _ } => true,
        _ => false,
    }
}

pub fn is_soulless(
    species: &Species
) -> bool {
    match species {
        Species::Terminal => false,
        Species::EpsilonHead { len: _ } => false,
        Species::ChromeNurse => false,
        Species::SegmentTransformer => false,
        _ => true,
    }
}

pub fn is_pushable(
    species: &Species
) -> bool {
    match species {
        Species::GlamourCrate => true,
        Species::DisciplineCrate => true,
        Species::GraceCrate => true,
        Species::PrideCrate => true,
        Species::FormCrate { form: _ } => true,
        Species::FunctionCrate { function: _ } => true,
        _ => false,
    }
}

pub fn match_faction_with_index(
    faction: &Faction
) -> Option<usize> {
    match faction {
        Faction::Saintly => Some(0),
        Faction::Ordered => Some(1),
        Faction::Feral => Some(2),
        Faction::Vile => Some(3),
        Faction::Serene => Some(4),
        Faction::Unaligned => None,
    }
}