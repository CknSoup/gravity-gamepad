mod debug;

use bevy::prelude::*;
use bevy::core::FixedTimestep;
use bevy_inspector_egui::{Inspectable};
use bevy::app::AppExit;
use bevy::render::camera::Camera2d;
use bevy_rapier2d::prelude::*;

const FLOOR: f32 = -100.0;
const GRAVITY: f32 = -1.8;
const JUMP: f32 = 25.0;
const TERMINAL_VERTICAL_VELOCITY: f32 = 10.0;
const TERMINAL_HORIZONTAL_VELOCITY: f32 = 10.0;
const FRICTION: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_startup_system(setup)
        // .add_startup_system(add_player)
        .add_system(keyboard_system)
        .add_system(gamepad_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.0))
                .with_system(physics)
        )
        .add_system(camera_movement)
        .add_system(display_contact_info)
        .run();
    return;
}

fn display_contact_info(
    rapier_context: Res<RapierContext>,
    ground_query: Query<(Entity, &Collider), With<Ground>>,
    player_query: Query<(Entity, &Collider), With<Player>>
) {
    let player_contact = player_query.get_single().ok().unwrap();
    
    for ground_contact in ground_query.iter() {
        if let Some(contact_pair) = rapier_context.contact_pair(player_contact.0, ground_contact.0) {
            if contact_pair.has_any_active_contacts() {
                println!("Contact here");
                
                // We may also read the contact manifolds to access the contact geometry.
                for manifold in contact_pair.manifolds() {
                    println!("Local-space contact normal: {}", manifold.local_n1());
                    println!("Local-space contact normal: {}", manifold.local_n2());
                    println!("World-space contact normal: {}", manifold.normal());

                    // // Read the geometric contacts.
                    for contact_point in manifold.points() {
                        println!("Found local contact point 1: {:?}", contact_point.local_p1());
                        println!("Found contact distance: {:?}", contact_point.dist()); // Negative if there is a penetration.
                        println!("Found contact impulse: {}", contact_point.impulse());
                        println!("Found friction impulse: {}", contact_point.tangent_impulse());
                    }

                    // Read the solver contacts.
                    for solver_contact in manifold.solver_contacts() {
                        // Keep in mind that all the solver contact data are expressed in world-space.
                        println!("Found solver contact point: {:?}", solver_contact.point());
                        println!("Found solver contact distance: {:?}", solver_contact.dist()); // Negative if there is a penetration.
                    }
                }
            }
        }
    }
    // let entity1 = ...; // A first entity with a collider attached.
    // let entity2 = ...; // A second entity with a collider attached.
    
    // /* Find the contact pair, if it exists, between two colliders. */
    // if let Some(contact_pair) = rapier_context.contact_pair(entity1, entity2) {
    //     // The contact pair exists meaning that the broad-phase identified a potential contact.
    //     if contact_pair.has_any_active_contact() {
    //         // The contact pair has active contacts, meaning that it
    //         // contains contacts for which contact forces were computed.
    //     }

    
    // }
}

#[derive(Component, Inspectable)]
struct Player;

#[derive(Component, Inspectable)]
struct Ground;

#[derive(Component)]
struct Enemy;

#[derive(Component, Inspectable, Default)]
struct PlayerControl {
    horizontal: f32,
    vertical: f32,
    jump: bool,
    action: bool,
}

fn physics(
    player_control_query: Query<&mut PlayerControl>,
    mut player: Query<(&mut Velocity, &mut ExternalForce, &mut ExternalImpulse), With<Player>>,
) {
    let player_control = player_control_query.get_single().ok().unwrap();
    for (mut player_velocity, mut player_extforce, mut player_extimpulse) in player.iter_mut() {
        // player_velocity.linvel = Vec2::new(player_control.horizontal, player_control.vertical);
        player_extimpulse.impulse = Vec2::new(player_control.horizontal * 800.0, 0.0);
        if player_control.jump {
            player_extimpulse.impulse = Vec2::new(0.0, 1500.5);
        }
    }
}

fn gamepad_system(
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_control: Query<&mut PlayerControl>,
) {
    fn controller_to_physics_transform(controller_stick: f32, temp: f32) -> f32 {
        match controller_stick {
            x if x.abs() > 0.75 => (x / x.abs()) * 0.5 * temp,
            x if x.abs() > 0.5 => (x / x.abs()) * 0.4 * temp,
            x if x.abs() > 0.25 => (x / x.abs()) * 0.2 * temp,
            x if x.abs() > 0.2 => (x / x.abs()) * 0.1 * temp,
            _ => 0.0,
        }
    }
    for gamepad in gamepads.iter().cloned() {
        for mut player_control in player_control.iter_mut() {
            let left_stick_x = axes
                .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
                .unwrap();
            // player_velocity.linvel = Vec2::new(controller_to_physics_transform(left_stick_x, 200.0), 0.0);
            player_control.horizontal = controller_to_physics_transform(left_stick_x, 200.0);

            if button_inputs.just_pressed(GamepadButton(gamepad, GamepadButtonType::South)) {
                println!("south");
                // player_extforce.force = Vec2::new(0.0, 10000.0);
                player_control.jump = true;
            }
            if button_inputs.just_pressed(GamepadButton(gamepad, GamepadButtonType::East)) {
                println!("east");
                // player_extforce.force = Vec2::new(0.0, 00.0);
                player_control.action = true;
            }
            if button_inputs.just_released(GamepadButton(gamepad, GamepadButtonType::South)) {
                println!("south");
                // player_extforce.force = Vec2::new(0.0, 10000.0);
                player_control.jump = false;
            }
            if button_inputs.just_released(GamepadButton(gamepad, GamepadButtonType::East)) {
                println!("east");
                // player_extforce.force = Vec2::new(0.0, 00.0);
                player_control.action = false;
            }
        }
        // for mut transform in camera_query.iter_mut() {
        //     let dpad_x = axes.get(GamepadAxis(gamepad, GamepadAxisType::DPadX)).unwrap();
        //     let dpad_y = axes.get(GamepadAxis(gamepad, GamepadAxisType::DPadY)).unwrap();
        //     // info!("Pressed {}", dpad_x);
        //     if dpad_x > 0.0 {
        //         transform.translation.x += 25.0;
        //     } else if dpad_x < 0.0 {
        //         transform.translation.x -= 25.0;
        //     }
        //     if dpad_y > 0.0 {
        //         transform.translation.y += 25.0;
        //     } else if dpad_y < 0.0 {
        //         transform.translation.y -= 25.0;
        //     }
        // }
    }
}

fn keyboard_system(
    keyboard: Res<Input<KeyCode>>,
    mut player_control: Query<&mut PlayerControl>,
) {
    for mut player_control in player_control.iter_mut() {
        if keyboard.just_pressed(KeyCode::A) {
            player_control.horizontal = -1.0;
        }
        if keyboard.just_released(KeyCode::A) {
            player_control.horizontal = 0.0;
        }
        if keyboard.just_pressed(KeyCode::D) {
            player_control.horizontal = 1.0;
        }
        if keyboard.just_released(KeyCode::D) {
            player_control.horizontal = 0.0;
        }
        if keyboard.just_pressed(KeyCode::W) {
            player_control.vertical = 1.0;
        }
        if keyboard.just_released(KeyCode::W) {
            player_control.vertical = 0.0;
        }
        if keyboard.just_pressed(KeyCode::S) {
            player_control.vertical = -1.0;
        }
        if keyboard.just_released(KeyCode::S) {
            player_control.vertical = 0.0;
        }
        if keyboard.just_pressed(KeyCode::J) {
            player_control.jump = true;
        }
        if keyboard.just_released(KeyCode::J) {
            player_control.jump = false;
        }
        if keyboard.just_pressed(KeyCode::K) {
            player_control.action = true;
        }
        if keyboard.just_released(KeyCode::K) {
            player_control.action = false;
        }
    }
}

fn camera_movement(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
) {
    for player_transform in player.iter() {
        let mut camera_transform = camera_query.single_mut();
        // for mut camera_transform in camera_query.iter_mut()
        {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut shutdown: EventWriter<AppExit>,
) {
    // let rp = RapierConfiguration {
    //     ..default()
    // };
    // rp.
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn()
        .insert(PlayerControl::default());

    let mut poly1: Vec<Vec2> = Vec::new();
    let mut i = 0.0;
    let mut j = -90.0;
    // while i < 100.0 {
    //     i += 0.1;
    //     j += 0.1;
    //     poly1.push(Vec2::new(i, j));
    // }
    poly1.push(Vec2::new(1101.0, -90.0));
    poly1.push(Vec2::new(1501.0, -105.0));
    poly1.push(Vec2::new(2001.0, -105.0));
    poly1.push(Vec2::new(2501.0, -105.0));
    poly1.push(Vec2::new(5000.0, -105.0));
    poly1.push(Vec2::new(5000.0, -105.00001));
    poly1.push(Vec2::new(8000.0, -105.00001));
    let polyline = Collider::polyline(poly1, None);
    // polyline.
    commands.spawn()
        .insert(polyline)
        // .insert(Collider::heightfield(vec!(
        //     -1000.0, -500.0, -250.0, 0.0, 250.0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0, 5000.0), matri
        .insert(Ground);

    let mut poly2: Vec<Vec2> = Vec::new();
    i = -500.0;
    while i < 500.0 {
        i += 0.5;
        j = (i) * (i) / 1000.0;
        poly2.push(Vec2::new(i, j));
    };
    //poly2.push(Vec2::new(0.0, -90.0));
    let polyline2 = Collider::polyline(poly2, None);
    // polyline.
    commands.spawn()
        .insert(polyline2)
        .insert(Ground);

    commands.spawn()
        .insert(Player)
        .insert_bundle(
            SpriteBundle {
                transform: Transform {
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    translation: Vec3::new(50.0, 200.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.5),
                    flip_x: false,
                    flip_y: false,
                    custom_size: None,
                    anchor: Default::default()
                },
                ..default()})
        .insert(Collider::capsule(Vect::new(0.0, 1.0), Vect::new(0.0, 0.0), 0.5))
        // .insert(Collider::cuboid(0.5, 1.0))
        // .insert(Collider::ball(0.5))
        .insert(GravityScale(5.0))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ColliderMassProperties::Density(250.0))
        .insert(Restitution::coefficient(0.0))
        .insert(Velocity {
            linvel: Default::default(),
            angvel: 0.0,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            ..default()
        })
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            ..default()
        });

    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform {
            scale: Vec3::new(30.0, 30.0, 0.0),
            translation: Vec3::new(50.0, -90.0, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 0.5, 1.0),
            flip_x: false,
            flip_y: false,
            custom_size: None,
            anchor: Default::default()
        },
        ..default()
    });
}