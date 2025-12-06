use bevy::prelude::*;
use crate::camera::controller::{spawn_camera, fly_camera_system};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, fly_camera_system);
    }
}
