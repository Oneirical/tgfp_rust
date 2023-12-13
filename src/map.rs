use bevy::prelude::*;

use crate::{components::{Position, Intangible}, axiom::{Function, CasterInfo}, world::{Plane, match_plane_with_vaults}, species::{Species, match_species_with_sprite, match_species_with_rotation, is_invisible}, vaults::{extract_square, match_vault_with_spawn_loc}, SpriteSheetHandle};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldMap{ entities: vec![None; WORLD_HEIGHT*WORLD_WIDTH], targeted_axioms: Vec::new(), warp_zones: Vec::new()});
        app.add_systems(Update, place_down_new_entities);
        app.add_event::<PlanePassage>();
    }
}

pub const WORLD_WIDTH: usize = 45;
pub const WORLD_HEIGHT: usize = 45;

#[derive(Resource)]
pub struct WorldMap {
    pub entities: Vec<Option<Entity>>,
    pub targeted_axioms: Vec<(Entity,Function, CasterInfo)>,
    pub warp_zones: Vec<((usize, usize), Plane)>,
}

#[derive(Event)]
pub struct PlanePassage(pub Plane);

pub fn generate_world_vector() -> Vec<Option<Entity>>{
    vec![None; WORLD_HEIGHT*WORLD_WIDTH]
}

pub fn xy_idx (x: usize, y: usize) -> usize{
    (y * WORLD_WIDTH) + x
}

pub fn get_entity_at_coords (map: &[Option<Entity>], x: usize, y: usize) -> Option<Entity> {
    map[xy_idx(x, y)]
}

pub fn is_in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < WORLD_WIDTH as i32 && y >= 0 && y < WORLD_HEIGHT as i32
}

pub fn place_down_new_entities(
    query: Query<(Entity, &Species, &Position, Has<Intangible>), Added<Position>>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
    texture_atlas_handle: Res<SpriteSheetHandle>,
) {
    for (entity_id, species, position, is_intangible) in query.iter(){
        if species == &Species::Projector{
            let plane = Plane::Epsilon;
            world_map.warp_zones.push(((position.x, position.y), plane.clone()));
            let vault = match_plane_with_vaults(plane);
            let spawn = match_vault_with_spawn_loc(vault.clone());
            let projection = extract_square(vault, spawn.0, spawn.1);
            for (x, i) in projection.iter().enumerate(){
                for (y, j) in i.iter().enumerate() {
                    let visibility = if is_invisible(j){
                        Visibility::Hidden
                    } else { Visibility::Visible };
                    let child = commands.spawn((SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle.handle.clone(),
                        sprite: TextureAtlasSprite{
                            index : match_species_with_sprite(j),
                            custom_size: Some(Vec2::new(0.5, 0.5)),
                            ..default()
                        },
                        visibility,
                        transform: Transform {
                            translation: Vec3{ x: y as f32/2./8.-1.19, y: x as f32/2./8.-1.19, z: 2.0},
                            scale: Vec3 { x: 1./8., y: 1./8., z: 1. },
                            rotation: match_species_with_rotation(j),
                            ..default()
                        },
                        ..default()
                    },
                    )).id();
                    commands.entity(entity_id).add_child(child);
                }
            }
        }
        if is_intangible {
            continue;
        }
        assert_eq!(world_map.entities[xy_idx(position.x, position.y)], None, "THERE IS A CREATURE SPAWNING ON TOP OF ANOTHER AT POSITION ({0}, {1})!", position.x, position.y);
        world_map.entities[xy_idx(position.x, position.y)] = Some(entity_id);
    }
}

pub fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
 }