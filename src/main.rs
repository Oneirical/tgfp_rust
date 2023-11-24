use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use bevy_tweening::{*, lens::TransformPositionLens};
use components::*;
use input::*;
use map::{WorldMap, MapPlugin, WORLD_WIDTH, WORLD_HEIGHT, xy_idx};
use network::NetworkPlugin;

mod components;
mod input;
mod network;
mod map;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // fill the entire browser window
                    fit_canvas_to_parent: true,
                    title: "The Games Foxes Play".into(),
                    ..default()
                }),
                ..default()
            }),
            GgrsPlugin::<Config>::default(),
        ))
        .add_plugins(NetworkPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(TweeningPlugin)
        .rollback_component_with_clone::<Transform>()
        //.rollback_component_with_clone::<RealityAnchor>()
        //.rollback_component_with_clone::<TextureAtlasSprite>()
        //.rollback_component_with_clone::<CreatureID>()
        .rollback_component_with_clone::<Position>()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_systems(PreStartup, load_spritesheet)
        .add_systems(Startup, (setup, spawn_players))
        .add_systems(Update, camera_follow)
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(GgrsSchedule, move_players)
        .run();
}
type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn(camera_bundle);
    commands.insert_resource(InputDelay{time: Timer::new(Duration::from_millis(200), TimerMode::Repeating)});
}

#[derive(Resource)]
pub struct SpriteSheetHandle {
    handle: Handle<TextureAtlas>
}

#[derive(Resource, Clone)]
pub struct InputDelay {
    pub time: Timer
}


fn load_spritesheet(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
){
    let img_path = "spritesheet.png".to_owned();
    let texture_handle = asset_server.load(img_path);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(16.0, 16.0),
        80, 2, None, None
    );
    let handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(SpriteSheetHandle{handle});
}

fn spawn_players(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut world_map: ResMut<WorldMap>,
) {
    // Player 1
    let position = (0,0);
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::ZERO,
        },
    );
    commands
        .spawn((
            RealityAnchor { player_id: 0 },
            CreatureID { creature_id: world_map.creature_count },
            Position { x:position.0 , y:position.1  },
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : 0_usize,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 0., y: 0., z: 0.0},
                    
                    ..default()
                },
                ..default()
            },
            Animator::new(tween),
        ))
        .add_rollback();
    world_map.entities[xy_idx(position.0, position.1)] = world_map.creature_count;
    world_map.creature_count += 1;

    // Player 2
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3{ x: 2., y: 2., z: 0.0},
        },
    );
    let position = (2,2);
    commands
        .spawn((
            RealityAnchor { player_id: 1 },
            CreatureID { creature_id: world_map.creature_count },
            Position { x:position.0 , y:position.1  },
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.handle.clone(),
                sprite: TextureAtlasSprite{
                    index : 0_usize,
                    color: Color::Rgba { red: 0., green: 200., blue: 0., alpha: 1. },
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3{ x: 2., y: 2., z: 0.0},
                    rotation: Quat::from_rotation_z(PI),
                    
                    ..default()
                },
                ..default()
            },
            Animator::new(tween),
        ))
        .add_rollback();

    world_map.entities[xy_idx(position.0, position.1)] = world_map.creature_count;
    world_map.creature_count += 1;
}

fn move_players(
    mut players: Query<(&Transform, &mut Animator<Transform>, &mut Position, &RealityAnchor)>,
    inputs: Res<PlayerInputs<Config>>,
    mut world_map: ResMut<WorldMap>,
) {
    for (transform, mut anim, mut pos, anchor) in &mut players {
        let (input, _) = inputs[anchor.player_id];

        let mut direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }
        if direction.x < 0. && pos.x == 0 || direction.x > 0. && pos.x == WORLD_WIDTH{
            direction.x = 0.;
        }
        if direction.y < 0. && pos.y == 0 || direction.y > 0. && pos.y == WORLD_HEIGHT{
            direction.y = 0.;
        }
        assert!(world_map.entities[xy_idx(pos.x, pos.y)] != 0);
        let (old_x, old_y) = (pos.x, pos.y);
        let old_idx = xy_idx(pos.x, pos.y);
        pos.x = (pos.x as f32 + direction.x) as usize;
        pos.y = (pos.y as f32 + direction.y) as usize;
        if world_map.entities[xy_idx(pos.x, pos.y)] != 0 {
            (pos.x, pos.y) = (old_x, old_y);
            continue;
        }
        let idx = xy_idx(pos.x, pos.y);
        world_map.entities.swap(old_idx, idx);
        
        //let move_delta = direction;

        //let old_pos = transform.translation.xy();
        let start = transform.translation;
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(150),
            TransformPositionLens {
                start,
                end: Vec3::new(pos.x as f32, pos.y as f32, 0.),
            },
        );
        anim.set_tweenable(tween);
    }
}

fn camera_follow(
    local_players: Res<LocalPlayers>,
    players: Query<(&RealityAnchor, &Transform)>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<RealityAnchor>)>,
) {
    for (anchor, player_transform) in &players {
        // only follow the local player
        if !local_players.0.contains(&anchor.player_id) {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in &mut cameras {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}
