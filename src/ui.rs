use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_tweening::{Tween, EaseFunction, lens::TransformPositionLens, Animator};

use crate::{SpriteSheetHandle, components::{UIElement, RightFaith, RealityAnchor, Faith, FaithPoint, MinimapTile}, map::{WORLD_HEIGHT, WORLD_WIDTH, WorldMap, xy_idx}, species::{Species, match_species_with_sprite, match_species_with_pixel}};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (draw_chain_borders, draw_resource_bars));
        app.add_systems(PostStartup, draw_minimap);
        app.add_systems(Update, update_minimap);
    }
}

#[derive(Bundle)]
pub struct UIBundle {
    sprite_bundle: SpriteSheetBundle,
    ui: UIElement,
    name: Name
}

fn update_player_faith(
    player: Query<&Faith, &RealityAnchor>,
    right_border: Query<&RightFaith>,
){

}

fn update_minimap(
    mut minimap: Query<(&mut TextureAtlasSprite, &MinimapTile)>,
    query: Query<&Species>,
    map: Res<WorldMap>,
){
    for (mut sprite, tile) in minimap.iter_mut(){
        let tex = match map.entities[xy_idx(tile.x, tile.y)]{
            Some(entity) => if let Ok(species) = query.get(entity) { match_species_with_pixel(species.clone()) } else{ panic!("There is an entity in the map that doesn't have a species!")},
            None => 107,
        };
        if sprite.index != tex{
            sprite.index = tex;
        }
    }
}

fn draw_minimap(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    for x in 0..WORLD_WIDTH{
        for y in 0..WORLD_HEIGHT{
            let size_factor = 16.;
            commands.spawn((UIBundle{
                sprite_bundle:SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.handle.clone(),
                    sprite: TextureAtlasSprite{
                        index : 107_usize,
                        custom_size: Some(Vec2::new(1., 1.)),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3{ x: 0., y: 0., z: 0.2},
                        ..default()
                    },
                    ..default()
                },
                ui: UIElement { x: -6.9+x as f32/size_factor, y: 3.4+y as f32/size_factor },
                name: Name::new("Minimap Tile")
            },
            MinimapTile{x, y}
            ));
        }
    }
}

fn draw_resource_bars(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    let names = ["Left Faith","Right Faith"];
    let rot = [Quat::from_rotation_z(0.),Quat::from_rotation_z(PI)];
    let pos = [(11.3,5.7),(16.46,5.7)];
    for i in 0..2{
        let entity_id = commands.spawn(UIBundle{
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : 30_usize,
                    custom_size: Some(Vec2::new(1.3, 1.3)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 0., y: 0., z: 0.2},
                    rotation: rot[i % 2],
                    ..default()
                },
                ..default()
            },
            ui: UIElement{
                x: pos[i].0,
                y: pos[i].1,
            },
            name: Name::new(names[i]),
        }).id();
        if i % 2 != 0{
            let tween = Tween::new(
                EaseFunction::BackInOut,
                Duration::from_millis(500),
                TransformPositionLens {
                    start: Vec3{ x: 0., y: 0., z: 0.2},
                    end: Vec3{ x: 0., y: 0., z: 0.2}
                },
            );
            commands.entity(entity_id).insert((RightFaith, Animator::new(tween)));
        }
    }
    for i in 0..20{
        commands.spawn((UIBundle{
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : 9_usize,
                    custom_size: Some(Vec2::new(0.8, 0.8)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 0., y: 0., z: 0.2},
                    ..default()
                },
                ..default()
            },
            ui: UIElement{
                x: 11.8 + i as f32/4.,
                y: 5.7,
            },
            name: Name::new("Faith Point"),
        }, 
        FaithPoint{num: i}));
    }

}

fn draw_chain_borders(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
    commands.spawn(UIBundle{
        sprite_bundle: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 2_usize,
                custom_size: Some(Vec2::new(25.1, 25.1)),
                ..default()
            },
            transform: Transform {
                translation: Vec3{ x: 0., y: 0., z: 0.1},
                ..default()
            },
            ..default()
        },
        ui: UIElement{
            x: 3.75,
            y: -1.
        },
        name: Name::new("Grid Border Mask"),
    });
    commands.spawn(UIBundle{
        sprite_bundle: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 166_usize,
                custom_size: Some(Vec2::new(9., 9.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3{ x: 14., y: 0., z: 0.2},
                rotation: Quat::from_rotation_z(PI/2.),
                ..default()
            },
            ..default()
        },
        ui: UIElement{
            x: 14.,
            y: 0.
        },
        name: Name::new("Inventory Tree"),
    });
    let mut main_square = get_chain_border(31, 31, (8., -1.5));
    let mut side_left = get_chain_border(6, 6, (-11.5, 11.));
    let mut side_left_bottom = get_chain_border(6, 24, (-11.5, -5.));
    let mut side_right = get_chain_border(18, 31, (33.5, -1.5));
    let mut all = Vec::new();
    all.append(&mut main_square);
    all.append(&mut side_left);
    all.append(&mut side_left_bottom);
    all.append(&mut side_right);
    
    for chain in all{
        commands.spawn(UIBundle{
            sprite_bundle: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : chain.sprite,
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: chain.position.0, y: chain.position.1, z: 1.0},
                    rotation: Quat::from_rotation_z(chain.rotation),
                    ..default()
                },
                ..default()
            },
            ui: UIElement{
                x: chain.position.0,
                y: chain.position.1
            },
            name: Name::new("Small Chain Border")
        });
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