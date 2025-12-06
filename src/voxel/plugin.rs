use bevy::prelude::*;
use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::voxel::chunk::Chunk;
use crate::voxel::meshing::generate_chunk_mesh;
use crate::voxel::types::VoxelType;
use crate::voxel::world::VoxelWorld;
use crate::rendering::materials::VoxelMaterial;
use crate::config::loader::load_config;

pub struct VoxelPlugin;

#[derive(Resource)]
pub struct WorldConfig {
    pub size_chunks: IVec3,
    pub chunk_size: i32,
    pub greedy_meshing: bool,
}

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldConfig {
                size_chunks: IVec3::new(32, 4, 32),
                chunk_size: 16,
                greedy_meshing: true,
            })
            .insert_resource(VoxelWorld::new(IVec3::new(32, 4, 32)))
            .add_systems(Startup, setup_voxel_world)
            .add_systems(Update, mesh_dirty_chunks_system);
    }
}

fn setup_voxel_world(
    mut world: ResMut<VoxelWorld>,
) {
    // Generate test terrain
    for chunk_pos in world.all_chunk_positions() {
        let mut chunk = Chunk::new(chunk_pos);
        
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_y_base = chunk_pos.y * CHUNK_SIZE_I32;
                
                for y in 0..CHUNK_SIZE {
                    let world_y = world_y_base + y as i32;
                    let voxel = match world_y {
                        0 => VoxelType::Bedrock,
                        1..=8 => VoxelType::Rock,
                        9..=14 => VoxelType::SubSoil,
                        15..=16 => VoxelType::TopSoil,
                        _ => VoxelType::Air,
                    };
                    chunk.set(UVec3::new(x as u32, y as u32, z as u32), voxel);
                }
            }
        }
        
        chunk.mark_dirty();
        world.insert_chunk(chunk);
    }
}

fn mesh_dirty_chunks_system(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<VoxelMaterial>,
) {
    // Collect dirty chunks first to avoid borrowing issues
    let dirty_chunks: Vec<IVec3> = world.dirty_chunks().collect();
    
    for chunk_pos in dirty_chunks {
        if let Some(chunk) = world.get_chunk_mut(chunk_pos) {
            // Clear dirty flag
            chunk.clear_dirty();
            
            // Generate mesh
            let mesh_data = generate_chunk_mesh(chunk, &world);
            
            if mesh_data.is_empty() {
                if let Some(entity) = chunk.mesh_entity() {
                    commands.entity(entity).despawn();
                    chunk.set_mesh_entity(Entity::PLACEHOLDER); // Hacky, should use Option properly
                }
                continue;
            }
            
            let mesh = mesh_data.into_mesh();
            let mesh_handle = meshes.add(mesh);
            
            if let Some(entity) = chunk.mesh_entity() {
                commands.entity(entity).insert(Mesh3d(mesh_handle));
            } else {
                let world_pos = VoxelWorld::chunk_to_world(chunk_pos);
                let entity = commands.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material.handle.clone()),
                    Transform::from_xyz(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32),
                )).id();
                chunk.set_mesh_entity(entity);
            }
        }
    }
}
