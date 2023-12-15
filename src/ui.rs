use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, text::{BreakLineOn, Text2dBounds, TextLayoutInfo}, sprite::Anchor};
use bevy_tweening::{Tween, EaseFunction, lens::TransformPositionLens, Animator};

use crate::{SpriteSheetHandle, components::{MinimapTile, LogIndex, MomentumMarker}, map::{WORLD_HEIGHT, WORLD_WIDTH, WorldMap, xy_idx}, species::{Species, match_species_with_pixel}, TurnState, text::{LORE, split_text}};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (draw_chain_borders, draw_soul_deck));
        //app.add_systems(PostStartup, draw_minimap);
        //app.add_systems(OnEnter(TurnState::AwaitingInput), update_minimap);
        app.add_systems(Update, (place_down_text, push_log));
        app.insert_resource(CenterOfWheel{x: 16.5+7.25, y: 2.3+5.});
        app.add_event::<LogMessage>();
    }
}

#[derive(Bundle)]
pub struct UIBundle {
    sprite_bundle: SpriteSheetBundle,
    name: Name
}

#[derive(Resource, Default, Reflect)]
pub struct CenterOfWheel{
    pub x: f32,
    pub y: f32,
}

fn draw_soul_deck(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
    asset_server: Res<AssetServer>,
    ui_center: Res<CenterOfWheel>,
){
    let sprites = [58, 167];
    let rot = [PI/2., 0.];
    let momentum = [(1,0),(0,1),(-1,0),(0,-1)];
    for i in 0..8{
        let spacing = 1.5;
        let icon = commands.spawn((UIBundle{
            sprite_bundle:SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : sprites[i%2],
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: (i as f32*PI/4.).cos() * (spacing + 0.2*(1.-(i as f32%2.))) +ui_center.x, y: (i as f32*PI/4.).sin() * (spacing + 0.2*(1.-(i as f32%2.)))+ui_center.y, z: 0.2},
                    rotation: Quat::from_rotation_z(rot[i%2]*(i as f32-2.)/2.),
                    ..default()
                },
                ..default()
            },
            name: Name::new("Wheel Element")
        },
        )).id();
        if i%2 == 0 {
            commands.entity(icon).insert(MomentumMarker{dir: momentum[i/2]});
        }
        let font = asset_server.load("Play-Regular.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 20.,
            color: Color::WHITE,
        };
        let spacing = 2.5;
        let text = ["D","2","W","1","A","3","S","4"];
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(text[i], text_style.clone()),
                transform: Transform {
                    translation: Vec3{ x: (i as f32*PI/4.).cos() * spacing +ui_center.x, y: (i as f32*PI/4.).sin() * spacing+ui_center.y, z: 0.2},
                    scale: Vec3{x: 1./64., y: 1./64., z: 0.}, // Set to the camera scaling mode fixed size
                    
                    ..default()
                },
                ..default()
            },
            Name::new("Wheel Label"),
        ));
    }
}

fn update_minimap(
    mut minimap: Query<(&mut TextureAtlasSprite, &MinimapTile)>,
    query: Query<&Species>,
    map: Res<WorldMap>,
){
    for (mut sprite, tile) in minimap.iter_mut(){
        let tex = match map.entities[xy_idx(tile.x, tile.y)]{
            Some(entity) => if let Ok(species) = query.get(entity) { match_species_with_pixel(species) } else{ panic!("There is an entity in the map that doesn't have a species!")},
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
                        translation: Vec3{  x: -6.9+x as f32/size_factor+7.25, y: 3.4+y as f32/size_factor +5., z: 0.2},
                        ..default()
                    },
                    ..default()
                },
                name: Name::new("Minimap Tile")
            },
            MinimapTile{x, y}
            ));
        }
    }
}

#[derive(Event)]
pub struct LogMessage(pub usize);

fn place_down_text(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut events: EventReader<LogMessage>
){
    for event in events.read(){
        let mut text_sections = Vec::new();
        let chosen_text = LORE[event.0];
        let split_text = split_text(chosen_text, &asset_server);
        for (snippet, style) in split_text {
            text_sections.push(TextSection::new(snippet, style));
        }
        let text = Text {
            sections: text_sections,
            alignment: TextAlignment::Left,
            linebreak_behavior: BreakLineOn::WordBoundary
        };
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(0),
            TransformPositionLens {
                start: Vec3{ x: 12.1+7.25, y: -10.+5., z: 0.07},
                end: Vec3{ x: 12.1+7.25, y: -10.+5., z: 0.07},
            },
        );
        commands.spawn((
            Text2dBundle {
                text,
                text_anchor: Anchor::BottomLeft,
                transform: Transform {
                    translation: Vec3{ x: 12.1+7.25, y: -10.+5., z: 0.07},
                    scale: Vec3{x: 1./64., y: 1./64., z: 0.}, // Set to the camera scaling mode fixed size
                    ..default()
                },
                text_2d_bounds: Text2dBounds { size: Vec2 { x: 550., y: 600. }},
                ..default()
            },
            LogIndex { index: 0 },
            Name::new("Log Message"),
            Animator::new(tween)
        ));
        
    }

}

fn push_log(
    mut new_text: Query<(Entity, &TextLayoutInfo, &mut Animator<Transform>, &Transform, &mut LogIndex)>,
    mut commands: Commands
){
    let mut newcomer = None;
    for (entity, entry, mut anim, transform, mut num) in new_text.iter_mut(){
        if num.index == 0 && transform.translation.x != 0.{ // needs transform to be modified by the main update before operating otherwise it is just 000
            let size = Vec2::new(entry.logical_size.x/64., entry.logical_size.y/64.);
            newcomer = Some((entity, size));
            let final_pos = (12.1+7.25, -3.7 + (size.y)/20.);
            let tween_tr = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(300),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3{ x: final_pos.0, y: final_pos.1, z: 0.07},
                },
            );
            anim.set_tweenable(tween_tr);
            num.index = 1;
            break;
        }
    }
    for (entity, _entry, mut anim, transform, mut num) in new_text.iter_mut(){
        if newcomer.is_some(){
            if newcomer.unwrap().0 == entity {continue;}
            let final_pos = (12.1+7.25, transform.translation.y + 0.2 + newcomer.unwrap().1.y);
            if final_pos.1 > 4. {
                commands.entity(entity).despawn();
                continue;
            }
            let tween_tr = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(300),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3{ x: final_pos.0, y: final_pos.1, z: 0.07},
                },
            );
            anim.set_tweenable(tween_tr);
            num.index += 1;
        }
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
                translation: Vec3{ x: 11., y: 4., z: 0.05},
                ..default()
            },
            ..default()
        },
        name: Name::new("Grid Border Mask"),
    });
    commands.spawn(UIBundle{
        sprite_bundle: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 3_usize,
                custom_size: Some(Vec2::new(15.1, 5.1)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform {
                translation: Vec3{ x: 19.35+7.25, y: 4.9, z: 0.1},
                ..default()
            },
            ..default()
        },
        name: Name::new("Top Log Border Mask"),
    });
    commands.spawn(UIBundle{
        sprite_bundle: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 3_usize,
                custom_size: Some(Vec2::new(15.1, 5.1)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform {
                translation: Vec3{x: 19.35+7.25, y: -11.4+5., z: 0.1},
                ..default()
            },
            ..default()
        },
        name: Name::new("Bottom Log Border Mask"),
    });
    commands.spawn(UIBundle{
        sprite_bundle: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.handle.clone(),
            sprite: TextureAtlasSprite{
                index : 3_usize,
                custom_size: Some(Vec2::new(6.1, 17.)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform {
                translation: Vec3{x: 26.5, y: 4.2, z: 0.01},
                ..default()
            },
            ..default()
        },
        name: Name::new("Bottom Log Border Mask"),
    });
    /*
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
    */
    let mut main_square = get_chain_border(31, 31, (8., -1.5));
    let mut side_left = get_chain_border(6, 6, (-11.5, 11.));
    let mut side_left_bottom = get_chain_border(6, 24, (-11.5, -5.));
    let mut side_right = get_chain_border(18, 18, (33.5, 5.));
    let mut side_right_bottom = get_chain_border(18, 12, (33.5, -11.));
    let mut all = Vec::new();
    all.append(&mut main_square);
    all.append(&mut side_left);
    all.append(&mut side_left_bottom);
    all.append(&mut side_right);
    all.append(&mut side_right_bottom);
    
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
                    translation: Vec3{ x: chain.position.0+7.25, y: chain.position.1+5., z: 1.0},
                    rotation: Quat::from_rotation_z(chain.rotation),
                    ..default()
                },
                ..default()
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