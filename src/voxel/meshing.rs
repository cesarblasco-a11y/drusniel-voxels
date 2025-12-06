use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::constants::VOXEL_SIZE;
use crate::voxel::chunk::Chunk;
use crate::voxel::types::{VoxelType, Voxel};
use crate::voxel::world::VoxelWorld;

#[derive(Component)]
pub struct ChunkMesh {
    pub chunk_position: IVec3,
}

#[derive(Copy, Clone, Debug)]
pub enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

pub fn generate_chunk_mesh(
    chunk: &Chunk,
    world: &VoxelWorld,
) -> MeshData {
    let mut mesh_data = MeshData::new();
    
    // Naive meshing for Phase 1 to ensure correctness first
    // Will upgrade to greedy meshing in optimization pass if needed, 
    // but let's try to implement basic face culling first.
    
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let local = UVec3::new(x, y, z);
                let voxel = chunk.get(local);
                
                if !voxel.is_solid() {
                    continue;
                }

                // Check all 6 faces
                check_face(chunk, world, local, Face::Top, &mut mesh_data, voxel);
                check_face(chunk, world, local, Face::Bottom, &mut mesh_data, voxel);
                check_face(chunk, world, local, Face::North, &mut mesh_data, voxel);
                check_face(chunk, world, local, Face::South, &mut mesh_data, voxel);
                check_face(chunk, world, local, Face::East, &mut mesh_data, voxel);
                check_face(chunk, world, local, Face::West, &mut mesh_data, voxel);
            }
        }
    }

    mesh_data
}

fn check_face(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
    mesh_data: &mut MeshData,
    voxel: VoxelType,
) {
    if is_face_visible(chunk, world, local, face) {
        add_face(mesh_data, local, face, voxel);
    }
}

fn is_face_visible(
    chunk: &Chunk,
    world: &VoxelWorld,
    local: UVec3,
    face: Face,
) -> bool {
    let (dx, dy, dz) = match face {
        Face::Top => (0, 1, 0),
        Face::Bottom => (0, -1, 0),
        Face::North => (0, 0, -1),
        Face::South => (0, 0, 1),
        Face::East => (1, 0, 0),
        Face::West => (-1, 0, 0),
    };

    let neighbor_x = local.x as i32 + dx;
    let neighbor_y = local.y as i32 + dy;
    let neighbor_z = local.z as i32 + dz;

    // If neighbor is within chunk
    if neighbor_x >= 0 && neighbor_x < 16 &&
       neighbor_y >= 0 && neighbor_y < 16 &&
       neighbor_z >= 0 && neighbor_z < 16 {
        let neighbor_voxel = chunk.get(UVec3::new(neighbor_x as u32, neighbor_y as u32, neighbor_z as u32));
        return !neighbor_voxel.is_solid();
    }

    // If neighbor is outside chunk, check world
    let chunk_pos = chunk.position();
    let world_pos = VoxelWorld::chunk_to_world(chunk_pos) + IVec3::new(neighbor_x, neighbor_y, neighbor_z); // This logic is slightly wrong for local -> world conversion with offset
    
    // Correct logic:
    // chunk_pos is in chunk coords.
    // chunk_to_world gives the bottom-left corner of the chunk in world coords.
    // local is offset from that corner.
    // So world_pos of the *current* voxel is chunk_origin + local.
    // Neighbor world pos is current_world_pos + direction.
    
    let chunk_origin = VoxelWorld::chunk_to_world(chunk_pos);
    let current_world_pos = chunk_origin + IVec3::new(local.x as i32, local.y as i32, local.z as i32);
    let neighbor_world_pos = current_world_pos + IVec3::new(dx, dy, dz);
    
    if let Some(neighbor_voxel) = world.get_voxel(neighbor_world_pos) {
        !neighbor_voxel.is_solid()
    } else {
        // If outside world bounds, assume visible (or not, depending on preference)
        // Usually we want to see the edge of the world
        true
    }
}

fn add_face(
    mesh_data: &mut MeshData,
    local: UVec3,
    face: Face,
    voxel: VoxelType,
) {
    let x = local.x as f32 * VOXEL_SIZE;
    let y = local.y as f32 * VOXEL_SIZE;
    let z = local.z as f32 * VOXEL_SIZE;
    let s = VOXEL_SIZE;

    let (v0, v1, v2, v3, normal) = match face {
        Face::Top => (
            [x, y + s, z + s], [x + s, y + s, z + s], [x + s, y + s, z], [x, y + s, z],
            [0.0, 1.0, 0.0]
        ),
        Face::Bottom => (
            [x, y, z], [x + s, y, z], [x + s, y, z + s], [x, y, z + s],
            [0.0, -1.0, 0.0]
        ),
        Face::North => (
            [x + s, y, z], [x, y, z], [x, y + s, z], [x + s, y + s, z],
            [0.0, 0.0, -1.0]
        ),
        Face::South => (
            [x, y, z + s], [x + s, y, z + s], [x + s, y + s, z + s], [x, y + s, z + s],
            [0.0, 0.0, 1.0]
        ),
        Face::East => (
            [x + s, y, z + s], [x + s, y, z], [x + s, y + s, z], [x + s, y + s, z + s],
            [1.0, 0.0, 0.0]
        ),
        Face::West => (
            [x, y, z], [x, y, z + s], [x, y + s, z + s], [x, y + s, z],
            [-1.0, 0.0, 0.0]
        ),
    };

    let start_idx = mesh_data.positions.len() as u32;
    
    mesh_data.positions.push(v0);
    mesh_data.positions.push(v1);
    mesh_data.positions.push(v2);
    mesh_data.positions.push(v3);
    
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    mesh_data.normals.push(normal);
    
    // UVs - simple for now, need atlas logic
    // Assuming 4x4 atlas for now
    let atlas_idx = voxel.atlas_index();
    let cols = 4.0;
    let rows = 4.0;
    let col = (atlas_idx % 4) as f32;
    let row = (atlas_idx / 4) as f32;
    
    let u_min = col / cols;
    let u_max = (col + 1.0) / cols;
    let v_min = row / rows;
    let v_max = (row + 1.0) / rows;
    
    mesh_data.uvs.push([u_min, v_max]);
    mesh_data.uvs.push([u_max, v_max]);
    mesh_data.uvs.push([u_max, v_min]);
    mesh_data.uvs.push([u_min, v_min]);
    
    // Reverse winding order to CCW (0, 2, 1) and (0, 3, 2)
    // Current vertices were defined in a way that resulted in CW winding for (0, 1, 2)
    
    mesh_data.indices.push(start_idx);
    mesh_data.indices.push(start_idx + 2);
    mesh_data.indices.push(start_idx + 1);
    
    mesh_data.indices.push(start_idx);
    mesh_data.indices.push(start_idx + 3);
    mesh_data.indices.push(start_idx + 2);
}
