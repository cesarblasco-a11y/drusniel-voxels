use bevy::prelude::*;
use voxel_builder::voxel::plugin::VoxelPlugin;
use voxel_builder::rendering::plugin::RenderingPlugin;
use voxel_builder::camera::plugin::CameraPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Pixel art look
        .add_plugins(VoxelPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(CameraPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1000.0,
        })
        .run();
}
