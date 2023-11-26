use bevy::prelude::*;
use bevy_ggrs::*;

use crate::{components::BuildQueue, BuildDelay, species::CreatureBundle, SpriteSheetHandle};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_resource_with_reflect::<WorldMap>();
        app.insert_resource(WorldMap{ entities: generate_world_vector(), creature_count: 1 });
        app.add_systems(Update, unpack_build_queue);
    }
}

pub const WORLD_WIDTH: usize = 45;
pub const WORLD_HEIGHT: usize = 45;

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

pub fn unpack_build_queue(
    mut builds: Query<&mut BuildQueue>,
    mut timer: ResMut<BuildDelay>,
    time: Res<Time>,
    texture_atlas_handle: Res<SpriteSheetHandle>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands, 
){
    timer.time.tick(time.delta());
    if true { //timer.time.finished()
        for mut build_list in builds.iter_mut(){
            let task = match build_list.build_queue.pop(){
                Some(result) => result,
                None => continue
            };
            let position = task.1;
            let new_creature = CreatureBundle::new(&texture_atlas_handle)
                .with_position(position.0, position.1)
                .with_id(world_map.creature_count)
                //.with_anim_source(22, 22)
                .with_species(task.0);
            world_map.entities[xy_idx(position.0, position.1)] = world_map.creature_count;
            world_map.creature_count += 1;
            commands.spawn(new_creature);
        }
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