use bevy::prelude::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::core_pipeline::tonemapping::Tonemapping;
use voxel_builder::voxel::plugin::VoxelPlugin;
use voxel_builder::rendering::plugin::RenderingPlugin;
use voxel_builder::camera::plugin::CameraPlugin;
use voxel_builder::interaction::InteractionPlugin;
use voxel_builder::viewmodel::PickaxePlugin;
use voxel_builder::vegetation::VegetationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Pixel art look
        .add_plugins(VoxelPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(PickaxePlugin)
        .add_plugins(VegetationPlugin)
        // Warm sky with slight pink/purple tint (Valheim style)
        .insert_resource(ClearColor(Color::srgb(0.65, 0.75, 0.9)))
        // Warm ambient light for that golden hour feel
        .insert_resource(AmbientLight {
            color: Color::srgb(0.9, 0.85, 0.75),
            brightness: 600.0,
        })
        .add_systems(Startup, setup_environment)
        .add_systems(Update, animate_sun)
        .run();
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct SunVisual;

fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun - Directional light with warm color and shadows
    let sun_direction = Vec3::new(-0.4, -0.6, -0.5).normalize();

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85), // Warm sunlight
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ZERO).looking_to(sun_direction, Vec3::Y),
        // Configure cascaded shadow maps for large terrain
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 30.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
        Sun,
    ));

    // Visual sun sphere in the sky
    let sun_position = Vec3::new(300.0, 200.0, 250.0);
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.7),
            emissive: LinearRgba::new(5.0, 4.5, 3.0, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(sun_position),
        SunVisual,
    ));

    // Sun glow (larger transparent sphere)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(35.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.9, 0.6, 0.15),
            emissive: LinearRgba::new(1.0, 0.9, 0.5, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(sun_position),
        SunVisual,
    ));

    // Distant fog effect - warm horizon haze (Valheim style)
    let haze_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.75, 0.9, 0.35), // Warm pink/purple haze
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // Horizon haze ring
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI * 0.25;
        let distance = 450.0;
        let x = 256.0 + angle.cos() * distance;
        let z = 256.0 + angle.sin() * distance;

        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::new(-angle.cos(), 0.0, -angle.sin()), Vec2::new(100.0, 80.0)))),
            MeshMaterial3d(haze_material.clone()),
            Transform::from_xyz(x, 30.0, z),
        ));
    }

    // Cloud layer (simple scattered cloud planes)
    let cloud_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // Spawn some clouds
    let cloud_positions = [
        (100.0, 120.0, 150.0, 40.0),
        (200.0, 130.0, 100.0, 55.0),
        (350.0, 115.0, 200.0, 45.0),
        (150.0, 125.0, 350.0, 50.0),
        (400.0, 135.0, 400.0, 60.0),
        (50.0, 110.0, 300.0, 35.0),
        (300.0, 140.0, 50.0, 48.0),
        (250.0, 118.0, 280.0, 42.0),
        (180.0, 128.0, 420.0, 52.0),
        (420.0, 122.0, 180.0, 38.0),
    ];

    for (x, y, z, size) in cloud_positions {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size)))),
            MeshMaterial3d(cloud_material.clone()),
            Transform::from_xyz(x, y, z),
        ));
        // Second layer for depth
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size * 0.7)))),
            MeshMaterial3d(cloud_material.clone()),
            Transform::from_xyz(x + 5.0, y - 3.0, z + 5.0),
        ));
    }
}

// Animate sun position for day/night cycle (slow)
fn animate_sun(
    time: Res<Time>,
    mut sun_query: Query<&mut Transform, With<Sun>>,
    mut sun_visual_query: Query<&mut Transform, (With<SunVisual>, Without<Sun>)>,
) {
    let cycle_speed = 0.005; // Slower day cycle
    let angle = time.elapsed_secs() * cycle_speed;

    // Sun direction rotates around the world
    let sun_dir = Vec3::new(
        angle.cos() * 0.5,
        -0.6 - angle.sin().abs() * 0.3, // Sun higher at noon
        angle.sin() * 0.5,
    ).normalize();

    for mut transform in sun_query.iter_mut() {
        *transform = Transform::from_translation(Vec3::ZERO).looking_to(sun_dir, Vec3::Y);
    }

    // Move visual sun sphere to match
    let sun_distance = 300.0;
    let sun_pos = Vec3::new(256.0, 50.0, 256.0) - sun_dir * sun_distance;

    for mut transform in sun_visual_query.iter_mut() {
        transform.translation = sun_pos;
    }
}
