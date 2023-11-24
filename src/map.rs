use bevy::prelude::*;
use bevy_ggrs::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_resource_with_reflect::<WorldMap>();
        app.insert_resource(WorldMap{ entities: generate_world_vector(), creature_count: 1 });
    }
}

pub const WORLD_WIDTH: usize = 81;
pub const WORLD_HEIGHT: usize = 81;

#[derive(Resource, Reflect, Default)]
pub struct WorldMap {
    pub entities: Vec<usize>,
    pub creature_count: usize,
}

fn generate_world_vector() -> Vec<usize>{
    let mut output = Vec::with_capacity(WORLD_WIDTH);
    for _x in 0..WORLD_WIDTH{
        for _y in 0..WORLD_HEIGHT{
            output.push(0);
        }
    }
    output
}

pub fn xy_idx (x: usize, y: usize) -> usize{
    (y as usize * WORLD_WIDTH) + x as usize
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