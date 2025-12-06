use bevy::prelude::*;
use std::hash::Hash;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
#[repr(u8)]
pub enum VoxelType {
    #[default]
    Air = 0,
    TopSoil = 1,
    SubSoil = 2,
    Rock = 3,
    Bedrock = 4,
    Sand = 5,
    Clay = 6,
}

#[derive(Clone, Debug)]
pub struct VoxelTypeInfo {
    pub solid: bool,
    pub hardness: f32,
    pub tool_required: ToolType,
    pub atlas_index: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ToolType {
    None,
    Shovel,
    Pickaxe,
}

// Trait for voxel queries (meshing needs this)
pub trait Voxel {
    fn is_solid(&self) -> bool;
    fn atlas_index(&self) -> u8;
}
