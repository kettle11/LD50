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

        // Setup UI
        let mut ui_manager = UIManager::new(world);

        let mut fonts = Fonts::empty();
        fonts
            .new_font_from_bytes(include_bytes!("../assets/Jomhuria-Regular.ttf"))
            .unwrap();
        //fonts.load_default_fonts();

        let mut standard_context = StandardContext::new(
            StandardStyle {
                primary_text_color: Color::WHITE,
                primary_color: Color::BLACK.with_alpha(0.5),
                padding: 12.,
                ..Default::default()
            },
            StandardInput::default(),
            fonts,
        );

        let mut ui = center(text("The Last Sky Pirate").with_size(|_, _, _| 100.));
        world.spawn((Transform::new(), Camera::new_for_user_interface()));

        move |event: Event, world: &mut World| {
            match event {
                Event::KappEvent(event) => {
                    if ui_manager.handle_event(&event, world, &mut standard_context) {
                        return true;
                    }
                }
                Event::FixedUpdate => {
                    RapierPhysicsManager::fixed_update(world);
                    MouseLook::fixed_update.run(world);
                    CharacterController::fixed_update.run(world);
                    // Perform physics and game related updates here.
                }
                Event::Draw => {
                    ui_manager.prepare(world, &mut standard_context);
                    ui_manager.layout(world, &mut standard_context, &mut ui);
                    ui_manager.render_ui(world);
                    // Things that occur before rendering can go here.
                }
                _ => {}
            }

            // Do not consume the event and allow other systems to respond to it.
            false
        }
    });
}
