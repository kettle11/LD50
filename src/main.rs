use koi::*;

#[derive(Component, Clone)]
struct Controlled;

pub mod mouse_look;
use mouse_look::*;

pub mod rapier_integration;
use rapier_integration::*;

pub mod character_controller;
pub use character_controller::*;

#[derive(Component, Clone)]
struct GameState {
    game_mode: GameMode,
    loaded: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum GameMode {
    Title,
    Game,
}

fn scale_world_root(
    scale: Vec3,
    mut hierarchy_transforms: Query<(&mut Transform, Option<&HierarchyNode>)>,
) {
    for (transform, hierarchy_node) in hierarchy_transforms.iter_mut() {
        if hierarchy_node.map_or(true, |h| h.parent().is_none()) {
            transform.scale = scale;
        }
    }
}

fn prepare_model_world(world: &mut World, scale: Vec3) {
    (|hierarchy_transforms: Query<(&mut Transform, Option<&HierarchyNode>)>| {
        scale_world_root(scale, hierarchy_transforms)
    })
    .run(world);

    let commands = Commands::new();
    let commands_entity = world.spawn(commands);
    koi::update_root_global_transforms.run(&world);
    let mut commands = world.remove_component::<Commands>(commands_entity).unwrap();
    commands.apply(world);
    commands.clear();

    // Update the world's transforms
    let commands_entity = world.spawn(commands);
    koi::update_global_transforms.run(&world);
    let mut commands = world.remove_component::<Commands>(commands_entity).unwrap();
    commands.apply(world);
    commands.clear();

    koi::flatten_world(world);

    (|entities_with_mesh: Query<&mut Handle<Mesh>>| {
        for m in entities_with_mesh.entities_and_components() {
            commands.add_component(*m.0, Color::RED);
            commands.add_component(*m.0, Collider::AttachedMesh);
        }
    })
    .run(world);
    commands.apply(world);
}

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

        world.spawn(GameState {
            game_mode: GameMode::Title,
            loaded: false,
        });

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

        let mut ui = conditional(
            |world: &mut World, _| world.get_singleton::<GameState>().game_mode == GameMode::Title,
            center(text("The Last Sky Pirate").with_size(|_, _, _| 100.)),
        );
        world.spawn((Transform::new(), Camera::new_for_user_interface()));

        let worlds = world.get_singleton::<Assets<World>>();
        let gltfs = [worlds.load_with_options(
            "assets/boat.glb",
            LoadWorldOptions {
                run_on_world: Some(Box::new(|world: &mut World| {
                    prepare_model_world(world, Vec3::fill(2.0))
                })),
            },
        )];

        let mut loaded = false;
        move |event: Event, world: &mut World| {
            match event {
                Event::KappEvent(event) => {
                    if ui_manager.handle_event(&event, world, &mut standard_context) {
                        return true;
                    }
                }
                Event::FixedUpdate => {
                    // Check that all gltfs are loaded.
                    (|worlds: &mut Assets<World>, game_state: &mut GameState| {
                        let mut loaded = true;
                        for asset in &gltfs {
                            if worlds.is_placeholder(asset) {
                                loaded = false;
                                break;
                            }
                        }
                        if loaded {
                            game_state.loaded = true;
                        }
                    })
                    .run(world);

                    // Start the game.
                    (|game_state: &mut GameState, input: &Input| {
                        if input.key_down(Key::Space) {
                            game_state.game_mode = GameMode::Game
                        }
                    })
                    .run(world);

                    if !loaded {
                        let mut commands = Commands::new();
                        (|worlds: &mut Assets<World>, game_state: &mut GameState| {
                            if game_state.loaded {
                                let boat = worlds.get_mut(&gltfs[0]).clone_world();
                                commands.add_world(boat);
                                loaded = true;
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    }

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
