use bevy::prelude::*;

use crate::components::{Position, Intangible};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldMap{ entities: generate_world_vector()});
        app.add_systems(Update, place_down_new_entities);
    }
}

pub const WORLD_WIDTH: usize = 45;
pub const WORLD_HEIGHT: usize = 45;

#[derive(Resource)]
pub struct WorldMap {
    pub entities: Vec<Option<Entity>>,
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

pub fn place_down_new_entities(
    query: Query<(Entity, &Position, Has<Intangible>), Added<Position>>,
    mut world_map: ResMut<WorldMap>
) {
    for (entity_id, position, is_intangible) in query.iter(){
        if is_intangible {
            continue;
        }
        assert_eq!(world_map.entities[xy_idx(position.x, position.y)], None);
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