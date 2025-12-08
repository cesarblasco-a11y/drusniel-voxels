use bevy::prelude::*;

/// Settings that drive the sky and sun animation
#[derive(Resource)]
pub struct AtmosphereSettings {
    /// Length of a full day/night cycle in seconds
    pub day_length: f32,
    /// Current time within the cycle
    pub time: f32,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            day_length: 1800.0, // 30 minutes for a full cycle
            // Start during the day (slightly past sunrise)
            time: 1800.0 * 0.25,
        }
    }
}

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct Cloud;

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AtmosphereSettings::default())
            // Soft initial sky tint
            .insert_resource(ClearColor(Color::srgba(0.60, 0.70, 0.90, 1.0)))
            .add_systems(Startup, setup_atmosphere)
            .add_systems(Update, animate_atmosphere);
    }
}

fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun directional light with extended shadow range
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(-0.3, -1.0, -0.2).normalize(), Vec3::Y),
        Sun,
    ));

    // Small scattered cloud puffs high in the sky
    let cloud_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.15),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // Spawn small cloud wisps scattered across the sky - much higher and smaller
    let cloud_specs = [
        (Vec3::new(100.0, 180.0, 100.0), 25.0),
        (Vec3::new(250.0, 200.0, 50.0), 20.0),
        (Vec3::new(400.0, 190.0, 300.0), 30.0),
        (Vec3::new(150.0, 210.0, 400.0), 22.0),
        (Vec3::new(350.0, 185.0, 200.0), 18.0),
        (Vec3::new(50.0, 195.0, 350.0), 28.0),
    ];

    for (pos, size) in cloud_specs {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size)))),
            MeshMaterial3d(cloud_material.clone()),
            Transform::from_translation(pos),
            Cloud,
        ));
    }

    // Water plane at sea level - terrain basins will naturally fill
    // Sea level matches the low points of terrain generation
    const SEA_LEVEL: f32 = 18.0;
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.35, 0.55, 0.7),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        metallic: 0.0,
        reflectance: 0.6,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    // Large water plane covering the world at sea level
    // World is 512x512, centered at (256, 256)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(512.0)))),
        MeshMaterial3d(water_material),
        Transform::from_xyz(256.0, SEA_LEVEL, 256.0),
    ));
}

fn animate_atmosphere(
    time: Res<Time>,
    mut settings: ResMut<AtmosphereSettings>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), (With<Sun>, Without<Cloud>)>,
    mut ambient: ResMut<AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
    mut clouds: Query<&mut Transform, (With<Cloud>, Without<Sun>)>,
) {
    // Advance time
    settings.time = (settings.time + time.delta_secs()) % settings.day_length;
    let phase = settings.time / settings.day_length; // 0..1

    // Sun position: overhead at noon, gentle arc for sunrise/sunset
    let theta = phase * std::f32::consts::TAU;
    let altitude = theta.sin(); // 1 at noon, -1 at midnight
    let azimuth = theta.cos();  // horizontal movement
    let sun_dir = Vec3::new(azimuth * 0.35, -altitude.max(0.2), 0.45).normalize_or_zero();

    // Lighting strength based on altitude
    let day_factor = saturate((altitude + 0.4) * 1.0).max(0.7); // keep a higher floor for nights
    // Pull sun down and ambient up to reduce contrast
    let sun_strength = lerp(2500.0, 9000.0, day_factor);
    let ambient_strength = lerp(3500.0, 8000.0, day_factor);

    // Update sun
    if let Ok((mut transform, mut light)) = sun_query.single_mut() {
        transform.look_to(sun_dir, Vec3::Y);
        light.illuminance = sun_strength;
        light.color = Color::srgba(
            lerp(0.85, 0.95, day_factor),
            lerp(0.78, 0.94, day_factor),
            lerp(0.72, 0.92, day_factor),
            1.0,
        );
    }

    // Update ambient and sky tint
    ambient.brightness = ambient_strength;
    ambient.color = Color::srgba(
        lerp(0.08, 0.65, day_factor),
        lerp(0.10, 0.75, day_factor),
        lerp(0.15, 0.90, day_factor),
        1.0,
    );
    clear_color.0 = Color::srgba(
        lerp(0.10, 0.65, day_factor),
        lerp(0.14, 0.78, day_factor),
        lerp(0.20, 0.92, day_factor),
        1.0,
    );

    // Drift clouds slowly across the sky
    let drift = Vec3::new(0.02, 0.0, 0.005) * time.delta_secs();
    for mut transform in clouds.iter_mut() {
        transform.translation += drift;
        // wrap around a large radius to avoid floating point drift
        if transform.translation.x > 600.0 {
            transform.translation.x -= 1200.0;
        }
        if transform.translation.z > 600.0 {
            transform.translation.z -= 1200.0;
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
