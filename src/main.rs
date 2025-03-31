use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (drone_control, follow_camera))
        .run();
}

#[derive(Component)]
struct Drone {
    target_altitude: f32,
    pitch_input: f32,
    roll_input: f32,
    yaw_input: f32,
}

#[derive(Component)]
struct FollowCamera;

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FollowCamera,
    ));

    // Ground
    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        TransformBundle::from_transform(Transform::from_xyz(0.0, -1.0, 0.0)),
    ));

    // Drone
    commands.spawn((
        Drone {
            target_altitude: 2.0,
            pitch_input: 0.0,
            roll_input: 0.0,
            yaw_input: 0.0,
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.2, 0.5),
        ExternalForce::default(),
        Velocity::default(),
        Damping {
            linear_damping: 10.0,  // **NO DRIFT**
            angular_damping: 15.0, // **SUPER STRONG AUTO-LEVEL**
        },
        GravityScale(1.0),
        TransformBundle::from_transform(Transform::from_xyz(0.0, 2.0, 0.0)),
    ));
}

fn drone_control(
    mut query: Query<(&mut ExternalForce, &mut Velocity, &mut Drone, &Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut force, mut velocity, mut drone, transform) in query.iter_mut() {
        let dt = time.delta_secs();

        // **Altitude Control**
        if keys.pressed(KeyCode::Space) {
            drone.target_altitude += 4.0 * dt;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            drone.target_altitude -= 4.0 * dt;
        }
        drone.target_altitude = drone.target_altitude.clamp(1.0, 10.0);

        let altitude_error = drone.target_altitude - transform.translation.y;
        let altitude_velocity = velocity.linvel.y;
        let altitude_adjustment = altitude_error * 15.0 - altitude_velocity * 6.0;
        let throttle = (9.81 + altitude_adjustment).clamp(5.0, 20.0);

        // **User Input Control**
        let control_strength = 4.0;
        let auto_level_strength = 20.0; // **MAXIMUM SELF-LEVELING**

        if keys.pressed(KeyCode::KeyW) {
            drone.pitch_input += control_strength * dt;
        } else if keys.pressed(KeyCode::KeyS) {
            drone.pitch_input -= control_strength * dt;
        } else {
            drone.pitch_input *= (1.0 - auto_level_strength * dt); // **STRONG AUTO-LEVEL**
        }

        if keys.pressed(KeyCode::KeyA) {
            drone.roll_input -= control_strength * dt;
        } else if keys.pressed(KeyCode::KeyD) {
            drone.roll_input += control_strength * dt;
        } else {
            drone.roll_input *= (1.0 - auto_level_strength * dt); // **STRONG AUTO-LEVEL**
        }

        if keys.pressed(KeyCode::KeyQ) {
            drone.yaw_input += control_strength * dt;
        } else if keys.pressed(KeyCode::KeyE) {
            drone.yaw_input -= control_strength * dt;
        } else {
            drone.yaw_input *= 0.8; // **Yaw stops naturally**
        }

        // **Clamp Tilt to Max 15 Degrees (0.26 Radians)**
        drone.pitch_input = drone.pitch_input.clamp(-0.26, 0.26);
        drone.roll_input = drone.roll_input.clamp(-0.26, 0.26);

        // **Force-Based Rotation Correction**
        let up_direction = transform.rotation * Vec3::Y;
        let thrust_force = up_direction * throttle;
        let gravity_force = Vec3::new(0.0, -9.81, 0.0);

        force.force = thrust_force + gravity_force;

        // **Proper Rotation Handling**
        let target_angvel = Vec3::new(
            -drone.pitch_input * 8.0,
            drone.yaw_input * 3.0,
            -drone.roll_input * 8.0,
        );
        velocity.angvel = velocity.angvel.lerp(target_angvel, 0.7); // **STRONG RESPONSE, NO DELAY**

        // **Speed Limit (Max 3m/s)**
        let max_speed = 3.0;
        if velocity.linvel.length() > max_speed {
            velocity.linvel = velocity.linvel.normalize() * max_speed;
        }
    }
}

fn follow_camera(
    drone_query: Query<&Transform, With<Drone>>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Drone>)>,
) {
    if let Ok(drone_transform) = drone_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let drone_pos = drone_transform.translation;
            let camera_offset = Vec3::new(-5.0, 5.0, 10.0);
            camera_transform.translation = drone_pos + camera_offset;
            camera_transform.look_at(drone_pos, Vec3::Y);
        }
    }
}
