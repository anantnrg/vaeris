use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default()) // Physics engine
        .add_plugins(RapierDebugRenderPlugin::default()) // Debug visualization
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Update, control_drone)
        .run();
}

// Drone Component with Throttle
#[derive(Component)]
struct Drone {
    throttle: f32, // Power level
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_physics(mut commands: Commands) {
    // Ground
    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        TransformBundle::from_transform(Transform::from_xyz(0.0, -2.0, 0.0)),
    ));

    // Drone
    commands.spawn((
        Drone { throttle: 9.81 }, // Start with gravity-neutral lift
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.2, 0.5),
        ExternalForce::default(),
        Damping {
            linear_damping: 2.0, // Reduce excessive velocity
            angular_damping: 2.0,
        },
        GravityScale(0.0), // We handle gravity manually
        TransformBundle::from_transform(Transform::from_xyz(0.0, 2.0, 0.0)),
    ));
}

// Drone controls with stable throttle
fn control_drone(
    mut query: Query<(&mut ExternalForce, &mut Drone, &Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut force, mut drone, transform) in query.iter_mut() {
        let mut thrust = Vec3::ZERO;

        // Adjust throttle gradually
        if keys.pressed(KeyCode::Space) {
            drone.throttle += 5.0 * time.delta_secs();
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            drone.throttle -= 5.0 * time.delta_secs();
        }

        // Clamp throttle to avoid crazy speeds
        drone.throttle = drone.throttle.clamp(0.0, 20.0);

        // Apply hover stabilization
        let vertical_velocity = transform.translation.y; // Current height
        let hover_force = drone.throttle - 9.81; // Neutral hover is at 9.81

        // Apply throttle force upwards, balancing with altitude
        thrust.y += hover_force - vertical_velocity * 0.5; // Dampen upwards movement

        // Lateral movement (WASD)
        let move_speed = 5.0;
        if keys.pressed(KeyCode::KeyW) {
            thrust.z -= move_speed;
        }
        if keys.pressed(KeyCode::KeyS) {
            thrust.z += move_speed;
        }
        if keys.pressed(KeyCode::KeyA) {
            thrust.x -= move_speed;
        }
        if keys.pressed(KeyCode::KeyD) {
            thrust.x += move_speed;
        }

        force.force = thrust;
    }
}
