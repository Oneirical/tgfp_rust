use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{SpriteSheetHandle, components::UIElement};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, draw_chain_borders);
    }
}

fn draw_chain_borders(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 2_usize,
                custom_size: Some(Vec2::new(21.2, 21.2)),
                ..default()
            },
            transform: Transform {
                translation: Vec3{ x: 0., y: 0., z: 0.1},
                ..default()
            },
            ..default()
        },
        UIElement{
            x: 0 as f32,
            y: 0 as f32
        }
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 166_usize,
                custom_size: Some(Vec2::new(9., 9.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3{ x: 14., y: 0., z: 0.},
                rotation: Quat::from_rotation_z(PI/2.),
                ..default()
            },
            ..default()
        },
        UIElement{
            x: 14 as f32,
            y: 0 as f32
        }
    ));
    let mut main_square = get_chain_border(31, 31, (1.5, -1.5));
    //let mut side_left = get_chain_border(26, 4, (0.5, -15.5));
    let mut side_right = get_chain_border(24, 31, (30., -1.5));
    let mut all = Vec::new();
    all.append(&mut main_square);
    //all.append(&mut side_left);
    all.append(&mut side_right);
    
    for chain in all{
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : chain.sprite,
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: chain.position.0 as f32, y: chain.position.1 as f32, z: 1.0},
                    rotation: Quat::from_rotation_z(chain.rotation),
                    ..default()
                },
                ..default()
            },
            UIElement{
                x: chain.position.0 as f32,
                y: chain.position.1 as f32
            }
        ));
    }
}

fn get_chain_border(
    width: usize,
    height: usize,
    offset: (f32, f32),
) -> Vec<ChainIcon>{
    let mut output = Vec::new();
    for x in 0..width{
        for y in 0..height{
            if x == 0 || y == 0 || x == width-1 || y == height-1{
                let chain = match (x,y){
                    (0,0) => ChainType::TopLeft,
                    (0, y) if y == height-1 => ChainType::BotLeft,
                    (x, 0) if x == width-1 => ChainType::TopRight,
                    (x, y) if x == width-1 && y == height-1 => ChainType::BotRight,
                    _ => match (x,y) {
                        (0, _y) => ChainType::Left,
                        (x, _y) if x == width-1 => ChainType::Right,
                        (_x, 0) => ChainType::Top,
                        _ => ChainType::Bot
                    }
                };
                let sprite = if [ChainType::TopLeft, ChainType::TopRight, ChainType::BotLeft, ChainType::BotRight].contains(&chain){
                    140
                } else {
                    139
                };
                let rotation = match chain{
                    ChainType::TopLeft => PI/2.,
                    ChainType::BotLeft => 0.,
                    ChainType::TopRight => PI,
                    ChainType::BotRight => 3.*PI/2.,
                    ChainType::Left => 0.,
                    ChainType::Right => PI,
                    ChainType::Top => PI/2.,
                    ChainType::Bot => 3.*PI/2.,
                };
                output.push(ChainIcon { sprite, rotation, position: ((x as f32 - width as f32/2. + offset.0)/2.,(y as f32 - height as f32/2. + offset.1)/2.) });
            }
        }
    }
    output
}
#[derive(PartialEq)]
enum ChainType{
    TopLeft,
    TopRight,
    BotLeft,
    BotRight,
    Top,
    Right,
    Left,
    Bot
}

struct ChainIcon{
    sprite: usize,
    rotation: f32,
    position: (f32, f32)
}