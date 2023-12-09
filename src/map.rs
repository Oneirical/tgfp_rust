use bevy::prelude::*;

use crate::{components::{Position, Intangible}, axiom::{Function, CasterInfo}};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldMap{ entities: generate_world_vector(), targeted_axioms: Vec::new()});
        app.add_systems(Update, place_down_new_entities);
    }
}

pub const WORLD_WIDTH: usize = 45;
pub const WORLD_HEIGHT: usize = 45;

#[derive(Resource)]
pub struct WorldMap {
    pub entities: Vec<Option<Entity>>,
    pub targeted_axioms: Vec<(Entity,Function, CasterInfo)>,
}

fn generate_world_vector() -> Vec<Option<Entity>>{
    let mut output = Vec::with_capacity(WORLD_WIDTH*WORLD_HEIGHT);
    for _x in 0..WORLD_WIDTH{
        for _y in 0..WORLD_HEIGHT{
            output.push(None);
        }
    }
    output
}

pub fn xy_idx (x: usize, y: usize) -> usize{
    (y * WORLD_WIDTH) + x
}

pub fn get_entity_at_coords (map: &Vec<Option<Entity>>, x: usize, y: usize) -> Option<Entity> {
    map[xy_idx(x, y)]
}

pub fn is_in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < WORLD_WIDTH as i32 && y >= 0 && y < WORLD_HEIGHT as i32
}

pub fn place_down_new_entities(
    query: Query<(Entity, &Position, Has<Intangible>), Added<Position>>,
    mut world_map: ResMut<WorldMap>
) {
    for (entity_id, position, is_intangible) in query.iter(){
        if is_intangible {
            continue;
        }
        assert_eq!(world_map.entities[xy_idx(position.x, position.y)], None, "THERE IS A CREATURE SPAWNING ON TOP OF ANOTHER AT POSITION ({0}, {1})!", position.x, position.y);
        world_map.entities[xy_idx(position.x, position.y)] = Some(entity_id);
    }
}

/*
pub fn coords_at_edge (
    coords: (usize, usize)
) -> bool
{
    coord_at_edge_x(coords.0) | coord_at_edge_y(coords.1)
}

pub fn coord_at_edge_x (x:usize) -> bool{
    x == 0 || x == WORLD_WIDTH
}

pub fn coord_at_edge_y (y:usize) -> bool{
    y == 0 || y == WORLD_HEIGHT
}
*/

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