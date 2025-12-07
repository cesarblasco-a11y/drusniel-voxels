use bevy::prelude::*;
use crate::camera::controller::{spawn_camera, player_camera_system};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (spawn_camera, lock_cursor_on_start))
            .add_systems(Update, player_camera_system);
    }
}

fn lock_cursor_on_start(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
    }
}

