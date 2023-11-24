use bevy::prelude::*;
use bevy_ggrs::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_resource_with_clone::<WorldMap>();
        app.insert_resource(WorldMap{ entities: generate_world_vector(), creature_count: 1 });
    }
}

const WORLD_WIDTH: usize = 81;
const WORLD_HEIGHT: usize = 81;

#[derive(Resource, Clone)]
pub struct WorldMap {
    pub entities: Vec<Vec<usize>>,
    pub creature_count: usize,
}

fn generate_world_vector() -> Vec<Vec<usize>>{
    let mut output = Vec::with_capacity(WORLD_WIDTH);
    for x in 0..WORLD_WIDTH{
        output.push(Vec::with_capacity(WORLD_HEIGHT));
        for _y in 0..WORLD_HEIGHT{
            output[x].push(0);
        }
    }
    output
}