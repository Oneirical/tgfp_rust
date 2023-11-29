use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode, window::WindowMode};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;
use bevy_tweening::{*, lens::TransformPositionLens};
use components::*;
use input::*;
use map::{WorldMap, MapPlugin, WORLD_WIDTH, WORLD_HEIGHT, xy_idx};
use network::NetworkPlugin;
use species::{CreatureBundle, Species};
use ui::UIPlugin;
use vaults::{get_build_sequence, Vault};

mod components;
mod input;
mod network;
mod map;
mod species;
mod vaults;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // fill the entire browser window
                    fit_canvas_to_parent: true,
                    focused: true,
                    resizable: false,
                    resolution: (1024.0, 576.0).into(),
                    title: "The Games Foxes Play".into(),
                    mode: WindowMode::Fullscreen,
                    position: WindowPosition::Centered(MonitorSelection::Current),
                    ..default()
                }),
                ..default()
            }),
            GgrsPlugin::<Config>::default(),
        ))
        .add_plugins(NetworkPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(UIPlugin)
        .rollback_component_with_clone::<Transform>()
        //.rollback_component_with_clone::<RealityAnchor>()
        //.rollback_component_with_clone::<TextureAtlasSprite>()
        //.rollback_component_with_clone::<CreatureID>()
        .rollback_component_with_clone::<Position>()
        .rollback_resource_with_clone::<BuildDelay>()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(Msaa::Off) // This fixes weird black lines on the tiles.
        .add_systems(PreStartup, load_spritesheet)
        .add_systems(Startup, (setup, spawn_players))
        .add_systems(PostStartup, summon_walls)
        .add_systems(Update, (camera_follow, zoom_2d, toggle_resolution, hide_and_show_creatures))
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(GgrsSchedule, move_players)
        .insert_resource(ResolutionSettings {
            giga: Vec2::new(1920.0, 1080.0),
            large: Vec2::new(1664.0, 936.0),
            medium: Vec2::new(1408.0, 792.0),
            small: Vec2::new(1280.0, 720.0),
            tiny: Vec2::new(1024.0, 576.0),
        })
        .run();
}
type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(16.);
    commands.spawn(camera_bundle);
    commands.insert_resource(InputDelay{time: Timer::new(Duration::from_millis(200), TimerMode::Once)});
    commands.insert_resource(BuildDelay{time: Timer::new(Duration::from_millis(200), TimerMode::Repeating)});
}

#[derive(Resource)]
pub struct SpriteSheetHandle {
    handle: Handle<TextureAtlas>
}

#[derive(Resource)]
struct ResolutionSettings {
    giga: Vec2,
    large: Vec2,
    medium: Vec2,
    small: Vec2,
    tiny: Vec2,
}

#[derive(Resource, Clone)]
pub struct InputDelay {
    pub time: Timer
}

#[derive(Resource, Clone)]
pub struct BuildDelay {
    pub time: Timer
}

fn toggle_resolution(
    keys: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window>,
    resolution: Res<ResolutionSettings>,
) {
    let mut window = windows.single_mut();

    if keys.just_pressed(KeyCode::Key1) {
        let res = resolution.tiny;
        window.resolution.set(res.x, res.y);
        window.position.center(MonitorSelection::Current);
    }
    if keys.just_pressed(KeyCode::Key2) {
        let res = resolution.small;
        window.resolution.set(res.x, res.y);
        window.position.center(MonitorSelection::Current);
    }
    if keys.just_pressed(KeyCode::Key3) {
        let res = resolution.medium;
        window.resolution.set(res.x, res.y);
        window.position.center(MonitorSelection::Current);
    }
    if keys.just_pressed(KeyCode::Key4) {
        let res = resolution.large;
        window.resolution.set(res.x, res.y);
        window.position.center(MonitorSelection::Current);
    }
    if keys.just_pressed(KeyCode::Key5) {
        let res = resolution.giga;
        window.resolution.set(res.x, res.y);
        window.position.center(MonitorSelection::Current);
    }
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
        160, 2, None, None
    );
    let handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(SpriteSheetHandle{handle});
}

fn spawn_players(
    mut commands: Commands, 
    texture_atlas_handle: Res<SpriteSheetHandle>,
) {
    // Player 1
    let position = (21,7);
    let player_1 = CreatureBundle::new(&texture_atlas_handle)
        .with_position(position.0, position.1)
        .with_species(Species::Terminal);
    commands.spawn((
        player_1, 
        RealityAnchor { player_id: 0},
        BuildQueue { build_queue : Vec::new()}
    ))
    .add_rollback();

    // Player 2
    /*
    let position = (2,2);
    let player_2 = CreatureBundle::new(&texture_atlas_handle)
        .with_position(position.0, position.1)
        .with_id(world_map.creature_count)
        .with_species(Species::Terminal)
        .with_rotation(PI)
        .with_tint(Color::Rgba { red: 0., green: 200., blue: 0., alpha: 1. });
    commands.spawn((
        player_2, 
        RealityAnchor { player_id: 1},
        BuildQueue { build_queue : Vec::new()}
    ))
    .add_rollback();
    world_map.entities[xy_idx(position.0, position.1)] = world_map.creature_count;
    world_map.creature_count += 1;
    */
}

fn summon_walls(
    mut build: Query<&mut BuildQueue>
){
    for mut build_queue in build.iter_mut(){
        // THIS IS A TEST
        if build_queue.build_queue.is_empty(){
            build_queue.build_queue.append(&mut get_build_sequence(Vault::EpicWow, (0,0)));
        }
    }
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
        assert!(world_map.entities[xy_idx(pos.x, pos.y)] != None);
        let (old_x, old_y) = (pos.x, pos.y);
        let old_idx = xy_idx(pos.x, pos.y);
        pos.x = (pos.x as f32 + direction.x) as usize;
        pos.y = (pos.y as f32 + direction.y) as usize;
        if world_map.entities[xy_idx(pos.x, pos.y)] != None {
            (pos.x, pos.y) = (old_x, old_y);
            continue;
        }
        let idx = xy_idx(pos.x, pos.y);
        world_map.entities.swap(old_idx, idx);
        

        let start = transform.translation;
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(200),
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
    mut ui: Query<(&mut Transform, &UIElement), (Without<Camera>, Without<RealityAnchor>)>,
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
        for (mut transform, ui) in &mut ui {
            transform.translation.x = ui.x + pos.x;
            transform.translation.y = ui.y + pos.y;
        }
    }
}

fn hide_and_show_creatures(
    mut creatures: Query<(&mut Visibility, &Position)>,
    players: Query<(&RealityAnchor, &Position)>,
    local_players: Res<LocalPlayers>,
){
    for (anchor, player_pos) in &players {
        // only follow the local player
        if !local_players.0.contains(&anchor.player_id) {
            continue;
        }
        let view_range = 8;
        for (mut vis, crea_pos) in &mut creatures {
            if (crea_pos.x as i32-player_pos.x as i32).abs() > view_range || (crea_pos.y as i32-player_pos.y as i32).abs() > view_range {
                *vis = Visibility::Hidden;
            } else { *vis = Visibility::Visible};
        }
    }
}

fn zoom_2d(
    mut q: Query<&mut OrthographicProjection, With<Camera2d>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if input.pressed(KeyCode::O) {
        let mut projection = q.single_mut();

        // example: zoom in
        projection.scale += 0.8 * time.delta_seconds();
        // example: zoom out
        //projection.scale *= 0.75;
    
        // always ensure you end up with sane values
        // (pick an upper and lower bound for your application)
        projection.scale = projection.scale.clamp(0.5, 5.0);
    }
    else if input.pressed(KeyCode::P) {
        let mut projection = q.single_mut();

        // example: zoom in
        projection.scale -= 0.8 * time.delta_seconds();
        projection.scale = projection.scale.clamp(0.5, 5.0);
    }
}