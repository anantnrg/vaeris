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

// Simple Drone Component
#[derive(Component)]
struct Drone;

fn setup_graphics(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}

fn setup_physics(mut commands: Commands) {
    // Ground
    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        TransformBundle::from_transform(Transform::from_xyz(0.0, -2.0, 0.0)),
    ));

    // Drone
    commands.spawn((
        Drone,
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.2, 0.5),
        ExternalForce::default(),
        TransformBundle::from_transform(Transform::from_xyz(0.0, 2.0, 0.0)),
    ));
}

// Controls for the drone (basic thrust & movement)
fn control_drone(
    mut query: Query<&mut ExternalForce, With<Drone>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mut force in query.iter_mut() {
        let mut thrust = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            thrust.z -= 5.0; // Forward
        }
        if keys.pressed(KeyCode::KeyS) {
            thrust.z += 5.0; // Backward
        }
        if keys.pressed(KeyCode::KeyA) {
            thrust.x -= 5.0; // Left
        }
        if keys.pressed(KeyCode::KeyD) {
            thrust.x += 5.0; // Right
        }
        if keys.pressed(KeyCode::Space) {
            thrust.y += 10.0; // Up (Thrust)
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            thrust.y -= 5.0; // Down (Throttle down)
        }

        force.force = thrust;
    }
}
