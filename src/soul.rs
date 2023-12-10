use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, EaseFunction, lens::TransformPositionLens};

use crate::{SpriteSheetHandle, components::{UIElement, SoulBreath, RealityAnchor}, ui::CenterOfWheel};

pub struct SoulPlugin;

impl Plugin for SoulPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, distribute_some_souls);
        app.add_systems(Update, soul_rotation);
    }
}

#[derive(Component, Clone)]
pub enum Soul {
    Feral,
    Ordered,
    Vile,
    Saintly,
    Serene,
}

#[derive(Bundle)]
pub struct SoulBundle {
    sprite_bundle: SpriteSheetBundle,
    animation: Animator<Transform>,
    name: Name,
    soul: Soul,
    ui: UIElement,
}

#[derive(Resource)]
pub struct CurrentEntityInUI {
    pub entity: Entity,
}

fn soul_rotation(
    ui_center: Res<CenterOfWheel>,
    current: Res<CurrentEntityInUI>,
    query: Query<&SoulBreath>,
    mut soul: Query<(&mut UIElement, &Soul)>,
    time: Res<Time>,
){
    let (draw, _held, disc) = if let Ok(breath) = query.get(current.entity) { (&breath.pile, &breath.held, &breath.discard) } 
    else{ panic!("The entity meant to be represented in the UI doesn't have a SoulBreath component!")};
    let spacing = 3.;
    for i in draw.iter().enumerate() {
        if let Ok((mut ui, soul_type)) = soul.get_mut(*i.1) {
            let offset = 30.15;
            let dist_between_souls = 412.;
            let slide_factor = match soul_type {
                Soul::Saintly => i.0 as f32*PI/dist_between_souls,
                Soul::Ordered => (1.*offset)+i.0 as f32*PI/dist_between_souls,
                Soul::Feral => (2.*offset)+i.0 as f32*PI/dist_between_souls,
                Soul::Vile => (3.*offset)+i.0 as f32*PI/dist_between_souls,
                Soul::Serene => (4.*offset)+i.0 as f32*PI/dist_between_souls,
            };
            (ui.x, ui.y) = (
                (slide_factor + time.elapsed_seconds_wrapped()*PI/5.).cos() * spacing +ui_center.x,
                (slide_factor + time.elapsed_seconds_wrapped()*PI/5.).sin() * spacing +ui_center.y,
            );
        }
        else{ panic!("A soul in the draw pile has no UIElement component!")};
    }
    let spacing = 0.6;
    for i in disc.iter().enumerate() {
        if let Ok((mut ui, _soul_type)) = soul.get_mut(*i.1) { 
            (ui.x, ui.y) = (
                (i.0 as f32*PI/32. + time.elapsed_seconds_wrapped()*PI/5.).cos() * spacing +ui_center.x,
                (i.0 as f32*PI/32. + time.elapsed_seconds_wrapped()*PI/5.).sin() * spacing +ui_center.y,
            );
        }
        else{ panic!("A soul in the draw pile has no UIElement component!")};
    }
}

fn distribute_some_souls(
    mut commands: Commands,
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut player: Query<&mut SoulBreath, With<RealityAnchor>>,
){  
    for i in 0..30{
        let soul = vec![Soul::Serene, Soul::Feral, Soul::Ordered, Soul::Saintly, Soul::Vile];
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3{ x: 0., y: 0., z: 0.5},
            },
        );
        let entity = commands.spawn(SoulBundle{
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : match_soul_with_sprite(&soul[i%5]),
                    custom_size: Some(Vec2::new(0.25, 0.25)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::ZERO,
                    ..default()
                },
                ..default()
            },
            animation: Animator::new(tween),
            name: Name::new("Breathed Soul"),
            soul: soul[i%5].clone(),
            ui: UIElement { x: 0., y: 0. }
        }).id();
        if let Ok(mut breath) = player.get_single_mut() {
            if i < 4 {breath.held.push(entity)}
            else if i < 18 {breath.pile.push(entity)}
            else {breath.pile.push(entity)}
        } else {
            panic!("There are zero or more than 1 players!")
        }   
    }
}

fn match_soul_with_sprite(
    soul: &Soul,
) -> usize{
    match soul{
        Soul::Feral => 164,
        Soul::Ordered => 161,
        Soul::Saintly => 160,
        Soul::Serene => 26,
        Soul::Vile => 165
    }
}