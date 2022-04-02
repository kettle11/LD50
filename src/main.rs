use koi::*;

#[derive(Component, Clone)]
struct Controlled;

pub mod mouse_look;
use mouse_look::*;

pub mod rapier_integration;
use rapier_integration::*;

pub mod character_controller;
pub use character_controller::*;

fn main() {
    App::new().setup_and_run(|world: &mut World| {
        // Setup things here.

        // Spawn a camera and make it look towards the origin.

        /*
        world.spawn((
            Transform::new()
                .with_position(Vec3::new(0.0, 4.0, 3.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            Camera::new(),
            CameraControls::new(),
        ));
        */

        spawn_skybox(world, "assets/venice_sunset.hdr");

        world.spawn((
            Transform::new()
                .with_position(Vec3::Y * -200.0)
                .with_scale(Vec3::fill(300.0)),
            Collider::Cuboid(Vec3::fill(0.5)),
            Mesh::CUBE,
            Material::DEFAULT,
            Color::AZURE,
        ));

        let character_parent = world.spawn((
            Transform::new().with_position(Vec3::Y * 30.0),
            Collider::Sphere(0.5),
            RigidBody {
                kinematic: false,
                can_rotate: (false, false, false),
                ..Default::default()
            },
            CharacterController,
            MouseLook::new(),
        ));

        let camera = world.spawn((Transform::new().with_position(Vec3::Y * 1.0), Camera::new()));
        set_parent(world, Some(character_parent), camera);

        // Spawn a cube that we can control

        world.spawn(RapierPhysicsManager::new());

        move |event: Event, world: &mut World| {
            match event {
                Event::FixedUpdate => {
                    RapierPhysicsManager::fixed_update(world);
                    MouseLook::fixed_update.run(world);
                    CharacterController::fixed_update.run(world);
                    // Perform physics and game related updates here.
                }
                Event::Draw => {
                    // Things that occur before rendering can go here.
                }
                _ => {}
            }

            // Do not consume the event and allow other systems to respond to it.
            false
        }
    });
}
