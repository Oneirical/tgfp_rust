use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, EaseFunction, lens::TransformPositionLens};
use rand::Rng;

use crate::{SpriteSheetHandle, components::{UIElement, SoulBreath, RealityAnchor, Position, MomentumMarker}, ui::CenterOfWheel};

pub struct SoulPlugin;

impl Plugin for SoulPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, distribute_some_souls);
        app.add_systems(Update, soul_rotation);
        app.insert_resource(SoulRotationTimer{timer: Timer::new(Duration::from_millis(10000), TimerMode::Repeating)});
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

#[derive(Resource)]
pub struct SoulRotationTimer {
    pub timer: Timer
}

fn soul_rotation(
    ui_center: Res<CenterOfWheel>,
    current: Res<CurrentEntityInUI>,
    query: Query<(&SoulBreath, &Position)>,
    mut soul: Query<(&mut Transform, &Animator<Transform>, &mut UIElement, &Soul)>,
    mut time: ResMut<SoulRotationTimer>,
    mut momentum_mark: Query<(&mut TextureAtlasSprite, &MomentumMarker)>,
    epoch: Res<Time>,
){
    time.timer.tick(epoch.delta());
    let (draw, held, disc, momentum) = if let Ok((breath, pos)) = query.get(current.entity) { (&breath.pile, &breath.held, &breath.discard, pos.momentum) } 
    else{ panic!("The entity meant to be represented in the UI doesn't have a SoulBreath component!")};
    for (mut sprite, mom) in momentum_mark.iter_mut(){
        if mom.dir == momentum {
            sprite.index = 59;
        }
        else {
            sprite.index = 58;
        }
    }
    for j in draw.iter() {
        for i in j{
            if let Ok((mut trans, anim, mut ui, soul_type)) = soul.get_mut(*i) {
                let index = draw[match_soul_with_display_index(soul_type)].iter().position(|&ent| ent == *i);
                let index = match index {
                    Some(ind) => ind,
                    None => {
                        dbg!(draw);
                        panic!("waa");
                    }
                };
                (ui.x, ui.y) = get_soul_rot_position(soul_type, (ui_center.x, ui_center.y), false, time.timer.elapsed_secs(), index);
                if anim.tweenable().progress() != 1.0 { continue; }
                trans.scale = Vec3{ x: 1., y: 1., z: 0.};
            }
            else{ panic!("A soul in the draw pile has no UIElement component!")};
        }
    }
    for i in held.iter().enumerate(){
        let slot_coords_ui = [
            ((3.*PI/4.).cos() * 1.5 +ui_center.x, (3.*PI/4.).sin() * 1.5 +ui_center.y),
            ((1.*PI/4.).cos() * 1.5 +ui_center.x, (1.*PI/4.).sin() * 1.5 +ui_center.y),
            ((5.*PI/4.).cos() * 1.5 +ui_center.x, (5.*PI/4.).sin() * 1.5 +ui_center.y),
            ((7.*PI/4.).cos() * 1.5 +ui_center.x, (7.*PI/4.).sin() * 1.5 +ui_center.y)
        ];
        if let Ok((mut trans, anim, mut ui, _soul_type)) = soul.get_mut(*i.1) { 
            (ui.x, ui.y) = slot_coords_ui[i.0];
            if anim.tweenable().progress() != 1.0 { continue; }
            trans.scale = Vec3{ x: 3., y: 3., z: 0.}; // TODO add an animation filter to avoid bobbing
        }
        else{ panic!("A soul in the draw pile has no UIElement component!")};
    }
    for j in disc.iter() {
        for i in j.iter(){
            if let Ok((mut trans, anim, mut ui, soul_type)) = soul.get_mut(*i) { 
                let index = disc[match_soul_with_display_index(soul_type)].iter().position(|&ent| ent == *i);
                (ui.x, ui.y) = get_soul_rot_position(soul_type, (ui_center.x, ui_center.y), true, time.timer.elapsed_secs(), index.unwrap());
                if anim.tweenable().progress() != 1.0 { continue; }
                trans.scale = Vec3{ x: 1., y: 1., z: 0.};
            }
            else{ panic!("A soul in the draw pile has no UIElement component!")};
        }
    }
}

pub fn get_soul_rot_position(
    soul_type: &Soul,
    ui_center: (f32, f32),
    is_discard: bool,
    time: f32,
    stack_pos: usize,
) -> (f32, f32){
    let spacing = if is_discard { 4. } else { 3. };
    let offset = 30.15;
    let dist_between_souls = 210.;
    let slide_factor = match soul_type {
        //_ => stack_pos as f32*PI/dist_between_souls,
        Soul::Saintly => stack_pos as f32*PI/dist_between_souls,
        Soul::Ordered => (1.*offset)+stack_pos as f32*PI/dist_between_souls,
        Soul::Feral => (2.*offset)+stack_pos as f32*PI/dist_between_souls,
        Soul::Vile => (3.*offset)+stack_pos as f32*PI/dist_between_souls,
        Soul::Serene => (4.*offset)+stack_pos as f32*PI/dist_between_souls,
    };
    (
        (slide_factor + time*-PI/5.).cos() * spacing +ui_center.0,
        (slide_factor + time*-PI/5.).sin() * spacing +ui_center.1,
    )
}

fn distribute_some_souls(
    mut commands: Commands,
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut player: Query<&mut SoulBreath, With<RealityAnchor>>,
){  
    for i in 0..200{
        let soul = vec![Soul::Serene, Soul::Feral, Soul::Ordered, Soul::Saintly, Soul::Vile];
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3{ x: 0., y: 0., z: 0.5},
            },
        );
        let scale = if i < 4 {
            Vec3::new(3., 3., 0.)
        } else { Vec3::new(1., 1., 0.) };
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..5);
        let entity = commands.spawn(SoulBundle{
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : match_soul_with_sprite(&soul[index]),
                    custom_size: Some(Vec2::new(0.25, 0.25)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::ZERO,
                    scale,
                    ..default()
                },
                ..default()
            },
            animation: Animator::new(tween),
            name: Name::new("Breathed Soul"),
            soul: soul[index].clone(),
            ui: UIElement { x: 0., y: 0. }
        }).id();
        if let Ok(mut breath) = player.get_single_mut() {
            if i < 4 {
                breath.held.push(entity);
            }
            else if i < 12 {
                breath.pile[index].push(entity);
            }
            else {
                breath.discard[index].push(entity);
            }
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

pub fn match_soul_with_display_index(
    soul: &Soul,
) -> usize{
    match soul{
        Soul::Feral => 1,
        Soul::Ordered => 2,
        Soul::Saintly => 3,
        Soul::Serene => 0,
        Soul::Vile => 4
    }
}