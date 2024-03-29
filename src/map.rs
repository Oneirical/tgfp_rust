use std::time::Duration;

use bevy::prelude::*;
use pathfinding::directed::astar::astar;

use crate::{components::{Position, Intangible}, axiom::{Function, CasterInfo, tup_usize_to_i32}, world::{Plane, match_plane_with_vaults}, species::{Species, match_species_with_sprite, match_species_with_rotation, is_invisible}, vaults::{extract_square, match_vault_with_spawn_loc}, SpriteSheetHandle, turn::Animation};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldMap{ entities: vec![None; WORLD_HEIGHT*WORLD_WIDTH], targeted_axioms: Vec::new(), warp_zones: Vec::new(), floor_axioms: Vec::new(), anim_queue: Vec::new(), animation_timer: Timer::new(Duration::from_millis(1),TimerMode::Repeating), anim_reality_anchor: Entity::PLACEHOLDER});
        app.add_systems(Update, place_down_new_entities);
    }
}

pub const WORLD_WIDTH: usize = 45;
pub const WORLD_HEIGHT: usize = 45;

#[derive(Resource)]
pub struct WorldMap {
    pub entities: Vec<Option<Entity>>,
    pub targeted_axioms: Vec<(Entity,Function, CasterInfo)>,
    pub floor_axioms: Vec<((usize, usize), Function, CasterInfo)>,
    pub warp_zones: Vec<((usize, usize), Plane)>,
    pub anim_queue: Vec<(Entity, Animation)>,
    pub animation_timer: Timer,
    pub anim_reality_anchor: Entity,
}

pub fn generate_world_vector() -> Vec<Option<Entity>>{
    vec![None; WORLD_HEIGHT*WORLD_WIDTH]
}

pub fn xy_idx (x: usize, y: usize) -> usize{
    (y * WORLD_WIDTH) + x
}

pub fn idx_xy (idx: usize) -> (usize, usize) {
    (idx%WORLD_WIDTH, idx/WORLD_WIDTH)
}

pub fn get_neighbours(x: usize, y: usize,) -> Vec<Option<(usize, usize)>>{
    let mut output = Vec::with_capacity(4);
    let offsets = [(-1,0),(1,0),(0,1),(0,-1)];
    for (dx,dy) in offsets{
        let fx = x.checked_add_signed(dx);
        let fy = y.checked_add_signed(dy);
        if fx.is_some_and(|fx| fx < WORLD_WIDTH) && fy.is_some_and(|fy| fy < WORLD_HEIGHT){
            output.push(Some((fx.unwrap(),fy.unwrap())));
        } else { output.push(None)}
    }
    output
}

pub fn get_empty_neighbours(map: &[Option<Entity>], x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut output = Vec::with_capacity(4);
    for pair in get_neighbours(x, y) {
        match pair {
            Some((nx,ny)) => if map[xy_idx(nx, ny)].is_none() { output.push((nx, ny)) },
            None => (),
        };
    }
    output
}

pub fn get_neighbouring_entities (map: &[Option<Entity>], x: usize, y: usize) -> Vec<Option<Entity>> {
    let mut output = Vec::with_capacity(4);
    for pair in get_neighbours(x, y) {
        match pair {
            Some((nx,ny)) => output.push(map[xy_idx(nx, ny)]),
            None => output.push(None),
        };
    }
    output
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
                            translation: Vec3{ x: y as f32/2./8.-1.19, y: x as f32/2./8.-1.19, z: -0.1},
                            scale: Vec3 { x: 1./8., y: 1./8., z: 1. },
                            rotation: match_species_with_rotation(j),
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
        if world_map.entities[xy_idx(position.x, position.y)] != None {
            dbg!("THERE IS A CREATURE SPAWNING ON TOP OF ANOTHER AT POSITION ({0}, {1})!", position.x, position.y);
        }
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

pub fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
 }
 
 pub fn get_best_move(
    start: (usize, usize),
    destination: (usize, usize),
    movements: Vec<(i32, i32)>,
 ) -> Option<(i32, i32)> { 
    let mut min_distance = manhattan_distance(tup_usize_to_i32(start), tup_usize_to_i32(destination));
    if movements.is_empty() { return None };
    let mut best_movement = movements[0];
 
    for movement in movements {
        let new_start = (start.0 as i32 + movement.0, start.1 as i32+ movement.1);
        let distance = manhattan_distance(new_start, tup_usize_to_i32(destination));
        if distance < min_distance {
            min_distance = distance;
            best_movement = movement;
        }
    }
 
    Some(best_movement)
 }

 pub fn get_all_factions_except_one(
    vec_vecs: &mut Vec<Vec<Entity>>,
    exclude: usize,
 ) -> Vec<Entity> {
    vec_vecs.swap_remove(exclude);
    let mut output = Vec::new();
    for i in vec_vecs {
        output.append(i);
    }
    output
 }

 pub fn pathfind_to_location(
    start: (usize, usize),
    destination: (usize, usize),
    map: &[Option<Entity>],
) -> Option<(Vec<(i32, i32)>, u32)> {
    let goal = tup_usize_to_i32(destination);
    let init = tup_usize_to_i32(start);
    let result = astar(&init,
                   |&(x, y)| detect_walls(x, y, vec![start, destination], map)
                              .into_iter().map(|p| (p, 1)),
                   |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
                   |&p| p == goal);
    result
}

fn detect_walls(
    x: i32,
    y: i32,
    force_empty: Vec<(usize, usize)>,
    map: &[Option<Entity>]
) -> Vec<(i32, i32)> {
    
    let mut neighbors = Vec::new();
    let diffs = [(x-1,y),(x+1,y), (x,y-1), (x,y+1)];
    let mut idxs = Vec::new();
    for (dx, dy) in diffs {
        if is_in_bounds(dx, dy) {
            idxs.push(xy_idx(dx as usize, dy as usize));
        }
    }
    for (i, idx) in idxs.iter().enumerate() {
        let mut okay = map[*idx].is_none();
        for (ex, ey) in &force_empty {
            if *idx == xy_idx(*ex, *ey) {
                okay = true;
            }
        }
        if okay {neighbors.push(diffs[i])};
    }  
    neighbors
}

pub fn get_astar_best_move(
    start: (usize, usize),
    destination: (usize, usize),
    map: &[Option<Entity>],
) -> Option<(i32, i32)> {
    let astar = pathfind_to_location(start, destination, map);
    match astar {
        None => return None,
        Some(path) => {
            if path.0.len() <= 1 {
                return None;
            }
            let path = path.0;
            Some((path[1].0-path[0].0, path[1].1-path[0].1))
        }
    }
}