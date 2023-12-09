use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode, window::WindowMode, input::common_conditions::input_toggle_active};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::TweeningPlugin;
use components::*;
use input::*;
use map::MapPlugin;
use soul::{SoulPlugin, CurrentEntityInUI};
use species::{CreatureBundle, Species, is_intangible};
use turn::TurnPlugin;
use ui::UIPlugin;
use vaults::{get_build_sequence, Vault};

mod components;
mod input;
mod map;
mod species;
mod vaults;
mod ui;
mod turn;
mod soul;
mod axiom;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
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
            }))
        .add_plugins(MapPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(TurnPlugin)
        .add_plugins(SoulPlugin)
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_state::<TurnState>()
        .register_type::<UIElement>()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(Msaa::Off) // This fixes weird black lines on the tiles.
        .insert_resource(CameraOffset{uix: 3., uiy: 0., playx: -3.75, playy: 1.})
        .add_systems(PreStartup, load_spritesheet)
        .add_systems(Startup, (setup, spawn_players, summon_walls))
        .add_systems(Update, (camera_follow, toggle_resolution, hide_and_show_creatures))
        .insert_resource(ResolutionSettings {
            giga: 80.,
            large: 64.,
            medium: 48.,
            small: 32.,
            tiny: 16.,
        })
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(64.);
    //camera_bundle.projection.scale = 0.99;
    commands.spawn(camera_bundle);
    commands.insert_resource(InputDelay{time: Timer::new(Duration::from_millis(100), TimerMode::Once)});
    commands.insert_resource(BuildDelay{time: Timer::new(Duration::from_millis(200), TimerMode::Repeating)});
}

#[derive(Resource)]
pub struct SpriteSheetHandle {
    handle: Handle<TextureAtlas>
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum TurnState {
    #[default]
    AwaitingInput,
    CalculatingResponse,
    ExecutingTurn,
    DispensingFunctions
}

#[derive(Resource)]
struct ResolutionSettings {
    giga: f32,
    large: f32,
    medium: f32,
    small: f32,
    tiny: f32,
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
    mut query_camera: Query<&mut OrthographicProjection, With<Camera2d>>,
    resolution: Res<ResolutionSettings>,
) {
    let mut projection = query_camera.single_mut();

    if keys.just_pressed(KeyCode::Numpad1) {
        projection.scaling_mode = ScalingMode::WindowSize(resolution.tiny);
    }
    if keys.just_pressed(KeyCode::Numpad2) {
        projection.scaling_mode = ScalingMode::WindowSize(resolution.small);
    }
    if keys.just_pressed(KeyCode::Numpad3) {
        projection.scaling_mode = ScalingMode::WindowSize(resolution.medium);
    }
    if keys.just_pressed(KeyCode::Numpad4) {
        projection.scaling_mode = ScalingMode::WindowSize(resolution.large);
    }
    if keys.just_pressed(KeyCode::Numpad5) {
        projection.scaling_mode = ScalingMode::WindowSize(resolution.giga);
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
    let position = (22,8);
    let player_1 = CreatureBundle::new(&texture_atlas_handle)
        .with_data(position.0, position.1, Species::Terminal);
    let entity = commands.spawn((
        player_1, 
        RealityAnchor { player_id: 0},
    )).id();
    commands.insert_resource(CurrentEntityInUI{entity});
}

fn summon_walls(
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut commands: Commands, 
){
    let queue = get_build_sequence(Vault::EpicWow, (1,1));
    for task in &queue{
        /*let task = match build_list.build_queue.pop(){
            Some(result) => result,
            None => continue
        }; */
        let position = task.1;
        let new_creature = CreatureBundle::new(&texture_atlas_handle)
            .with_data(position.0, position.1, task.0.clone());
        let entity_id = commands.spawn(new_creature).id();
        if is_intangible(&task.0){
            commands.entity(entity_id).insert(Intangible);
        }
    }
}

#[derive(Resource, Reflect)]
struct CameraOffset{
    uix: f32,
    uiy: f32,
    playx: f32,
    playy: f32,
}

fn camera_follow(
    players: Query<&Transform, With<RealityAnchor>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<RealityAnchor>)>,
    mut ui: Query<(&mut Transform, &UIElement), (Without<Camera>, Without<RealityAnchor>)>,
    offset: Res<CameraOffset>,
) {
    for player_transform in &players {

        let pos = player_transform.translation;

        for mut transform in &mut cameras {
            transform.translation.x = pos.x+offset.uix; // To offset it to the left
            transform.translation.y = pos.y+offset.uiy;
        }
        for (mut transform, ui) in &mut ui {
            transform.translation.x = ui.x + pos.x+offset.playx;
            transform.translation.y = ui.y + pos.y+offset.playy;
        }
    }
}

fn hide_and_show_creatures(
    mut creatures: Query<(&mut Visibility, &Position)>,
    players: Query<&Position, With<RealityAnchor>>,
){
    for player_pos in &players {
        let view_range = 20;
        for (mut vis, crea_pos) in &mut creatures {
            if (crea_pos.x as i32-player_pos.x as i32).abs() > view_range || (crea_pos.y as i32-player_pos.y as i32).abs() > view_range {
                *vis = Visibility::Hidden;
            } else { *vis = Visibility::Visible};
        }
    }
}