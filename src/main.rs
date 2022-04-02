use koi::*;

#[derive(Component, Clone)]
struct Controlled;

pub mod mouse_look;
use mouse_look::*;

pub mod rapier_integration;
use rapier_integration::*;

pub mod character_controller;
pub use character_controller::*;

mod explosion_manager;
use explosion_manager::*;

#[derive(Component, Clone)]
pub struct GameState {
    game_mode: GameMode,
    can_grapple: bool,
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
}

fn main() {
    App::new().setup_and_run(|world: &mut World| {
        let environment_size = 200.;
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
            can_grapple: false,
        });

        spawn_skybox(world, "assets/venice_sunset.hdr");

        let sounds = world.get_singleton::<Assets<Sound>>();
        let upbeat_vibes_song = sounds.load("assets/upbeat_vibes.wav");

        // Setup the player
        let mut player_audio_source = AudioSource::new();
        //player_audio_source.play(&upbeat_vibes_song, true);
        let camera = world.spawn((
            Transform::new().with_position(Vec3::Y * 1.0),
            Camera::new(),
            CharacterControllerCamera,
            Listener::new(),
            MouseLook::new(),
            player_audio_source,
        ));

        // Setup the player
        let character_controller = CharacterController::new(world);
        let character_parent = world.spawn((
            Transform::new().with_position(Vec3::Y * 100.0 + Vec3::XZ * environment_size / 2.0),
            Collider::Sphere(0.5),
            RigidBody {
                kinematic: false,
                can_rotate: (false, false, false),
                ..Default::default()
            },
            character_controller,
        ));
        set_parent(world, Some(character_parent), camera);

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
                primary_text_color: Color::INTERNATIONAL_ORANGE,
                primary_color: Color::BLACK.with_alpha(0.5),
                padding: 12.,
                ..Default::default()
            },
            StandardInput::default(),
            fonts,
        );

        let mut ui = stack((
            conditional(
                |world: &mut World, _| {
                    world.get_singleton::<GameState>().game_mode == GameMode::Title
                },
                center(text("The Last Sky Pirate").with_size(|_, _, _| 100.)),
            ),
            conditional(
                |world: &mut World, _| {
                    world.get_singleton::<GameState>().game_mode != GameMode::Title
                },
                center(stack((
                    rectangle(Vec2::fill(4.0)),
                    fill(|world: &mut World, _, _| {
                        if world.get_singleton::<GameState>().can_grapple {
                            Color::WHITE
                        } else {
                            Color::BLACK
                        }
                    }),
                ))),
            ),
        ));
        world.spawn((Transform::new(), Camera::new_for_user_interface()));

        let worlds = world.get_singleton::<Assets<World>>();
        let models = [
            worlds.load_with_options(
                "assets/boat3.glb",
                LoadWorldOptions {
                    run_on_world: Some(Box::new(|world: &mut World| {
                        prepare_model_world(world, Vec3::fill(3.0));
                        let mut commands = Commands::new();
                        (|entities_with_mesh: Query<&mut Handle<Mesh>>| {
                            for m in entities_with_mesh.entities_and_components() {
                                commands.add_component(*m.0, Color::RED);
                                commands.add_component(*m.0, Collider::AttachedMeshConvex);
                                commands.add_component(
                                    *m.0,
                                    RigidBody {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.01,
                                        linear_damping: 0.8,
                                        angular_damping: 0.9,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    },
                                );
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    })),
                },
            ),
            worlds.load_with_options(
                "assets/floating_island.glb",
                LoadWorldOptions {
                    run_on_world: Some(Box::new(|world: &mut World| {
                        prepare_model_world(world, Vec3::fill(1.0));
                        let mut commands = Commands::new();
                        (|entities_with_mesh: Query<&mut Handle<Mesh>>| {
                            for m in entities_with_mesh.entities_and_components() {
                                commands.add_component(*m.0, Collider::AttachedMeshConvex);
                                commands.add_component(
                                    *m.0,
                                    RigidBody {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.0,
                                        linear_damping: 1.0,
                                        angular_damping: 1.0,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    },
                                );
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    })),
                },
            ),
            worlds.load_with_options(
                "assets/barrel.glb",
                LoadWorldOptions {
                    run_on_world: Some(Box::new(|world: &mut World| {
                        prepare_model_world(world, Vec3::fill(0.3));
                        let mut commands = Commands::new();
                        (|entities_with_mesh: Query<&mut Handle<Mesh>>| {
                            for m in entities_with_mesh.entities_and_components() {
                                commands.add_component(*m.0, Collider::AttachedMeshConvex);
                                commands.add_component(
                                    *m.0,
                                    RigidBody {
                                        gravity_scale: 1.0,

                                        ..Default::default()
                                    },
                                );
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    })),
                },
            ),
            worlds.load_with_options(
                "assets/boat3.glb",
                LoadWorldOptions {
                    run_on_world: Some(Box::new(|world: &mut World| {
                        prepare_model_world(world, Vec3::fill(2.0));
                        let mut commands = Commands::new();
                        (|entities_with_mesh: Query<&mut Handle<Mesh>>| {
                            for m in entities_with_mesh.entities_and_components() {
                                commands.add_component(*m.0, Color::RED);
                                commands.add_component(*m.0, Collider::AttachedMeshConvex);
                                commands.add_component(
                                    *m.0,
                                    RigidBody {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.01,
                                        linear_damping: 0.6,
                                        angular_damping: 0.0,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    },
                                );
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    })),
                },
            ),
        ];

        ExplosionManager::setup_system(world);

        let mut loaded = false;
        let mut setup = false;
        move |event: Event, world: &mut World| {
            match event {
                Event::KappEvent(event) => {
                    if ui_manager.handle_event(&event, world, &mut standard_context) {
                        return true;
                    }
                }
                Event::FixedUpdate => {
                    // Check that all models are loaded.
                    (|worlds: &mut Assets<World>, game_state: &mut GameState| {
                        loaded = true;
                        for asset in &models {
                            if worlds.is_placeholder(asset) {
                                loaded = false;
                                break;
                            }
                        }
                    })
                    .run(world);

                    // Start the game.
                    (|game_state: &mut GameState, input: &Input| {
                        if input.key_down(Key::Space) {
                            game_state.game_mode = GameMode::Game
                        }
                        if input.key_down(Key::T) {
                            game_state.game_mode = GameMode::Title
                        }
                    })
                    .run(world);

                    if loaded && !setup {
                        setup = true;
                        // Setup the water plane
                        let water_material = (|materials: &mut Assets<Material>| {
                            materials.add(new_pbr_material(
                                Shader::PHYSICALLY_BASED_TRANSPARENT_DOUBLE_SIDED,
                                PBRProperties {
                                    roughness: 0.02,
                                    base_color: Color::new_from_bytes(7, 80, 97, 200),
                                    ..Default::default()
                                },
                            ))
                        })
                        .run(world);

                        world.spawn((
                            Transform::new().with_scale(Vec3::fill(1000000.)),
                            Mesh::PLANE,
                            water_material.clone(),
                            RenderFlags::DEFAULT.with_layer(RenderFlags::DO_NOT_CAST_SHADOWS),
                        ));

                        let mut commands = Commands::new();

                        (|worlds: &mut Assets<World>| {
                            let mut random = Random::new();
                            for _ in 0..50 {
                                let v = random.f32();
                                let random_position = Vec3::new(
                                    random.f32() * environment_size,
                                    random.f32() * 100.0 + 50.,
                                    random.f32() * environment_size,
                                );
                                if v > 0.3 {
                                    let mut boat = worlds.get_mut(&models[0]).clone_world();

                                    (|transform: &mut Transform| {
                                        transform.position = random_position;
                                        transform.rotation = Quat::from_angle_axis(
                                            random.f32() * std::f32::consts::TAU,
                                            Vec3::Y,
                                        );
                                    })
                                    .run(&mut boat);

                                    commands.add_world(boat);

                                    // Spawn some barrels on top
                                    for _ in 0..3 {
                                        let mut barrel = worlds.get_mut(&models[2]).clone_world();

                                        let random_offset = Vec3::new(
                                            random.f32() * 2.0 - 1.0 - 4.0,
                                            7.0,
                                            random.f32() * 2.0 - 1.0 - 4.0,
                                        );
                                        (|transform: &mut Transform| {
                                            transform.position = random_position + random_offset;

                                            transform.rotation = Quat::from_angle_axis(
                                                random.f32() * std::f32::consts::TAU,
                                                Vec3::Y,
                                            );
                                        })
                                        .run(&mut barrel);

                                        commands.add_world(barrel);
                                    }
                                } else {
                                    let mut boat = worlds.get_mut(&models[1]).clone_world();

                                    (|transform: &mut Transform| {
                                        transform.position = random_position;

                                        transform.rotation = Quat::from_angle_axis(
                                            random.f32() * std::f32::consts::TAU,
                                            Vec3::Y,
                                        );
                                    })
                                    .run(&mut boat);

                                    commands.add_world(boat);

                                    // Spawn some barrels on top
                                    for _ in 0..3 {
                                        let mut barrel = worlds.get_mut(&models[2]).clone_world();

                                        let random_range = 4.0;
                                        let random_offset = Vec3::new(
                                            random.f32() * random_range - random_range / 2.0 - 4.0,
                                            7.0,
                                            random.f32() * random_range - random_range / 2.0 - 4.0,
                                        );
                                        (|transform: &mut Transform| {
                                            transform.position = random_position + random_offset;

                                            transform.rotation = Quat::from_angle_axis(
                                                random.f32() * std::f32::consts::TAU,
                                                Vec3::Y,
                                            );
                                        })
                                        .run(&mut barrel);

                                        commands.add_world(barrel);
                                    }
                                }
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    }

                    if setup {
                        ExplosionManager::fixed_update_system.run(world);
                        MouseLook::fixed_update.run(world);
                        CharacterController::fixed_update.run(world);
                        RapierPhysicsManager::fixed_update(world);
                    }
                    // Perform physics and game related updates here.
                }
                Event::Draw => {
                    Cable::update_meshes_system.run(world);

                    ui_manager.prepare(world, &mut standard_context);
                    ui_manager.layout(world, &mut standard_context, &mut ui);
                    ui_manager.render_ui(world);
                    // Things that occur before rendering can go here.
                }
            }

            // Do not consume the event and allow other systems to respond to it.
            false
        }
    });
}
