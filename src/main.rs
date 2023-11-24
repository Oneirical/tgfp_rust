use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use components::*;
use input::*;
use map::{WorldMap, MapPlugin};
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
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_clone::<RealityAnchor>()
        .rollback_component_with_clone::<TextureAtlasSprite>()
        .rollback_component_with_clone::<CreatureID>()
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
const MAP_SIZE: i32 = 41;

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn(camera_bundle);
}

fn load_spritesheet( // I am so glad this works. Just looking at this code is going to make me fail NNN. - 8th November 2023
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

#[derive(Resource)]
pub struct SpriteSheetHandle {
    handle: Handle<TextureAtlas>
}

fn spawn_players(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut world_map: ResMut<WorldMap>,
) {
    // Player 1
    let position = (0,0);
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
        ))
        .add_rollback();
    world_map.entities[position.0][position.1] = world_map.creature_count;
    world_map.creature_count += 1;

    // Player 2
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
        ))
        .add_rollback();

    world_map.entities[position.0][position.1] = world_map.creature_count;
    world_map.creature_count += 1;
}

fn move_players(
    mut players: Query<(&mut Transform, &RealityAnchor)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, anchor) in &mut players {
        let (input, _) = inputs[anchor.player_id];

        let direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        let move_speed = 7.;
        let move_delta = direction * move_speed * time.delta_seconds();

        let old_pos = transform.translation.xy();
        let limit = Vec2::splat(MAP_SIZE as f32 / 2. - 0.5);
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
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
