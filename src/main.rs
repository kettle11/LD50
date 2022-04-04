#![feature(portable_simd)]

mod ui;
use koi::*;
pub use ui::*;

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
use rocket::check_rocket_collisions_system;
use terrain_generator::*;

mod rocket;

mod find_flat_parts;

mod terrain_generator;

mod worm;
use worm::*;

#[derive(Component, Clone)]
pub struct GameState {
    game_mode: GameMode,
    can_grapple: bool,
    needs_reset: bool,
    player_max_height: f32,
    victory: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum GameMode {
    Title,
    Game,
    GameOver,
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
        // Setup things here.

        let mut camera = Camera::new();
        camera.clear_color = Some(Color::WHITE);
        let mut controls = CameraControls::new();
        controls.max_speed *= 100.;

        //Spawn a camera and make it look towards the origin
        let title_camera = world.spawn((
            Transform::new()
                .with_position(Vec3::new(-847.27747, 111.74761, -594.119))
                .with_rotation(Quat::from_xyzw(
                    -0.11213023,
                    0.8636663,
                    -0.2205305,
                    -0.43913826,
                )),
            camera,
            // controls,
        ));

        world.spawn(GameState {
            game_mode: GameMode::Title,
            can_grapple: false,
            needs_reset: false,
            player_max_height: 0.0,
            victory: false,
        });

        let size_xz = 64;

        let mut terrain = Terrain::new(size_xz, 512);
        terrain.create_chunks(world);

        world.spawn((
            Color::YELLOW,
            Mesh::SPHERE,
            Transform::new().with_scale(Vec3::fill(30.0)),
            Material::DEFAULT,
        ));

        let mut player_camera_entity = None;
        //let mut terrain_chunks = Vec::new();

        /*
        for i in 0..2 {k
            let (world_chunk_mesh, has_at_least_one_triangle) =
                (|graphics: &mut Graphics, meshes: &mut Assets<Mesh>| {
                    let generated_chunk_mesh =
                        terrain.create_chunk_mesh(Vec3u::new(0, offset_y, 0), size_xz);
                    let has_at_least_one_triangle = !generated_chunk_mesh.indices.is_empty();
                    (
                        meshes.add(Mesh::new(graphics, generated_chunk_mesh)),
                        has_at_least_one_triangle,
                    )
                })
                .run(world);

            terrain_meshes.push(world_chunk_mesh.clone());
            if has_at_least_one_triangle {
                terrain_chunks.push(world.spawn((
                    world_chunk_mesh,
                    Material::DEFAULT,
                    Transform::new().with_position(
                        Vec3::Y * (offset_y as f32 / size_xz as f32) * terrain.scale + world_offset,
                    ),
                    Collider::AttachedMesh,
                )));
            } else {
                println!("EMPTY CHUNK!");
            }
            offset_y += size_xz;
        }
        */

        spawn_reflection_probe(world, "assets/venice_sunset.hdr");

        let sounds = world.get_singleton::<Assets<Sound>>();
        let upbeat_vibes_song = sounds.load("assets/upbeat_vibes.wav");
        //let shoot_grapple_sound = sounds.load("assets/shoot_grapple.wav");

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

        let mut ui = get_ui();

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
                                    RigidBody::new(RigidBodyInner {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.0,
                                        linear_damping: 0.0,
                                        angular_damping: 0.0,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    }),
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
                                    RigidBody::new(RigidBodyInner {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.0,
                                        linear_damping: 1.0,
                                        angular_damping: 1.0,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    }),
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
                                    RigidBody::new(RigidBodyInner {
                                        gravity_scale: 1.0,

                                        ..Default::default()
                                    }),
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
                                    RigidBody::new(RigidBodyInner {
                                        kinematic: false,
                                        can_rotate: (true, true, true),
                                        gravity_scale: 0.1,
                                        linear_damping: 0.0,
                                        angular_damping: 0.0,
                                        velocity: Vec3::ZERO,
                                        ..Default::default()
                                    }),
                                );
                            }
                        })
                        .run(world);
                        commands.apply(world);
                    })),
                },
            ),
            worlds.load_with_options(
                "assets/rocket.glb",
                LoadWorldOptions {
                    run_on_world: Some(Box::new(|world: &mut World| {
                        let mut commands = Commands::new();

                        (|transform: Query<&mut Transform>| {
                            for (e, _) in transform.entities_and_components().next() {
                                commands.add_component(
                                    *e,
                                    Powerup {
                                        grants_rockets: true,
                                        collected: false,
                                        grants_cable_length: false,
                                    },
                                )
                            }
                        })
                        .run(world);
                        commands.apply(world);
                        prepare_model_world(world, Vec3::fill(2.0));
                    })),
                },
            ),
        ];

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
            Transform::new().with_scale(Vec3::fill(10000.)),
            Mesh::PLANE,
            water_material.clone(),
            RenderFlags::DEFAULT.with_layer(RenderFlags::DO_NOT_CAST_SHADOWS),
        ));

        world.spawn((
            Transform::new()
                .with_position(Vec3::Y * -1000.0)
                .with_scale(Vec3::fill(1000000.)),
            Mesh::PLANE,
            Color::BLACK,
            RenderFlags::DEFAULT.with_layer(RenderFlags::DO_NOT_CAST_SHADOWS),
        ));

        ExplosionManager::setup_system(world);

        let mut random = Random::new_with_seed(13);

        let mut camera_rotation_angle: f32 = 0.0;

        let mut loaded = false;

        let low_poly_uv_sphere = (|meshes: &mut Assets<Mesh>, graphics: &mut Graphics| {
            meshes.add(Mesh::new(graphics, uv_sphere(4, 4, Vec2::ONE)))
        })
        .run(world);

        move |event: Event, world: &mut World| {
            match event {
                Event::KappEvent(event) => {
                    if ui_manager.handle_event(&event, world, &mut standard_context) {
                        return true;
                    }
                    match event {
                        KappEvent::PointerDown { .. } | KappEvent::KeyDown { .. } => {
                            let world_state = world.get_singleton::<GameState>();
                            match world_state.game_mode {
                                GameMode::GameOver | GameMode::Title => {
                                    world_state.needs_reset = true;
                                }
                                _ => {}
                            }
                        }

                        _ => {}
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
                        // if input.key_down(Key::T) {
                        //     game_state.game_mode = GameMode::Title
                        // }
                    })
                    .run(world);

                    let needs_setup = &mut world.get_singleton::<GameState>().needs_reset;
                    if loaded && *needs_setup {
                        println!("RESETTING");
                        *needs_setup = false;

                        let mut commands = Commands::new();

                        // (|rigid_bodies: Query<
                        //     &mut RapierRigidBody,
                        //     Without<CharacterController>,
                        // >| {
                        //     for (entity, _) in rigid_bodies.entities_and_components() {
                        //         commands.add_component(*entity, ToDespawn);
                        //     }
                        // })
                        // .run(world);
                        commands.apply(world);

                        let game_state = world.get_singleton::<GameState>();
                        game_state.game_mode = GameMode::Game;
                        game_state.player_max_height = 0.0;

                        println!("SETTING UP");

                        // Setup the player

                        let mut setup_already = false;
                        // Reset or spawn the player
                        let player_start_transform = Transform::new()
                            .with_position(Vec3::new(38.728767, 47.28899, 22.055452))
                            .with_rotation(Quat::from_angle_axis(
                                std::f32::consts::TAU * 0.3,
                                Vec3::Y,
                            ));
                        if (|player: (&mut CharacterController, &mut Transform, &mut RigidBody)| {
                            *player.1 = player_start_transform;
                            player.2.velocity = Vec3::ZERO;
                            player.2.mutated_position = true;
                            player.2.mutated_velocity = true;
                            player.0.reset();
                            setup_already = true;
                        })
                        .try_run(world)
                        .is_err()
                        {
                            let mut player_audio_source = AudioSource::new();

                            let camera = world.spawn((
                                Transform::new().with_position(Vec3::Y * 1.0),
                                {
                                    let mut camera = Camera::new();
                                    camera.clear_color = Some(Color::WHITE);
                                    camera.enabled = false;
                                    camera
                                },
                                CharacterControllerCamera,
                                Listener::new(),
                                MouseLook::new(),
                                player_audio_source,
                            ));

                            player_camera_entity = Some(camera);

                            // Setup the player
                            let character_controller = CharacterController::new(world);

                            let character_parent = world.spawn((
                                player_start_transform,
                                Collider::Sphere(1.0),
                                RigidBody::new(RigidBodyInner {
                                    kinematic: false,
                                    can_rotate: (false, false, false),
                                    ..Default::default()
                                }),
                                character_controller,
                                AudioSource::new(),
                            ));
                            set_parent(world, Some(character_parent), camera);
                        }

                        /*
                        world.spawn((
                            Transform::new()
                                .with_scale(Vec3::fill(200.))
                                .with_position(-Vec3::Y * 99. + Vec3::XZ * 100.),
                            Mesh::CUBE,
                            Material::DEFAULT,
                            Color::WHITE,
                            Collider::Cuboid(Vec3::fill(0.5)),
                            RenderFlags::DEFAULT.with_layer(RenderFlags::DO_NOT_CAST_SHADOWS),
                        ));
                        */

                        reset_powerups.run(world);

                        if !setup_already {
                            for _ in 0..150 {
                                let random_position =
                                    Vec3::new(
                                        random.f32() * terrain.scale,
                                        random.f32() * 2000.0 + 50.,
                                        random.f32() * terrain.scale,
                                    ) - Vec3::new(terrain.scale / 2.0, 0.0, terrain.scale / 2.0);
                                world.spawn((
                                    Transform::new()
                                        .with_position(random_position)
                                        .with_scale(Vec3::fill(8.0)),
                                    low_poly_uv_sphere.clone(),
                                    Collider::Sphere(0.5),
                                    Color::from_srgb_hex(0xFFD700, 1.0),
                                    Material::UNLIT,
                                    Powerup {
                                        collected: false,
                                        grants_rockets: false,
                                        grants_cable_length: true,
                                    },
                                ));
                            }
                            setup_worm(world);

                            (|worlds: &mut Assets<World>| {
                                let mut rocket = worlds.get_mut(&models[4]).clone_world();

                                (|transform: &mut Transform| {
                                    transform.position = Vec3::Y * 3191.0;
                                })
                                .run(&mut rocket);

                                commands.add_world(rocket);

                                for _ in 0..300 {
                                    let v = random.f32();
                                    let random_position = Vec3::new(
                                        random.f32() * terrain.scale,
                                        random.f32() * 3000.0 + 50.,
                                        random.f32() * terrain.scale,
                                    ) - Vec3::new(
                                        terrain.scale / 2.0,
                                        0.0,
                                        terrain.scale / 2.0,
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
                                            let mut barrel =
                                                worlds.get_mut(&models[2]).clone_world();

                                            let random_offset = Vec3::new(
                                                random.f32() * 2.0 - 1.0 - 4.0,
                                                7.0,
                                                random.f32() * 2.0 - 1.0 - 4.0,
                                            );
                                            (|transform: &mut Transform| {
                                                transform.position =
                                                    random_position + random_offset;

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
                                            let mut barrel =
                                                worlds.get_mut(&models[2]).clone_world();

                                            let random_range = 4.0;
                                            let random_offset = Vec3::new(
                                                random.f32() * random_range
                                                    - random_range / 2.0
                                                    - 4.0,
                                                7.0,
                                                random.f32() * random_range
                                                    - random_range / 2.0
                                                    - 4.0,
                                            );
                                            (|transform: &mut Transform| {
                                                transform.position =
                                                    random_position + random_offset;

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
                        }
                        (|(worm_transform, worm): (&mut Transform, &mut Worm)| {
                            worm_transform.position = Vec3::Y * -200.0;
                            worm.lerp_target = None; // Some(3000.0);
                            worm.rockets_hit = 0;
                        })
                        .run(world);

                        commands.apply(world);

                        update_root_global_transforms.run(world);
                        update_global_transforms.run(world);
                        apply_commands(world);
                    }

                    let game_state = world.get_singleton::<GameState>();

                    match game_state.game_mode {
                        GameMode::Game => {
                            world
                                .get_component_mut::<Camera>(title_camera)
                                .unwrap()
                                .enabled = false;
                            if let Some(player_camera_entity) = player_camera_entity {
                                let transform = *world
                                    .get_component_mut::<GlobalTransform>(player_camera_entity)
                                    .unwrap();
                                let camera = world
                                    .get_component_mut::<Camera>(player_camera_entity)
                                    .unwrap();
                                // Blend the color towards blue
                                camera.clear_color = Some(Color::interpolate(
                                    Color::WHITE,
                                    Color::AZURE.with_chroma(0.4),
                                    (transform.position.y / 3191.0).clamp(0.0, 1.0),
                                ));
                                camera.enabled = true;
                            }
                            ExplosionManager::fixed_update_system.run(world);
                            MouseLook::fixed_update.run(world);
                            CharacterController::fixed_update.run(world);
                            RapierPhysicsManager::despawn.run(world);
                            RapierPhysicsManager::fixed_update(world);
                            collect_powerups.run(world);
                            check_rocket_collisions_system.run(world);
                            worm::run_worm(world);
                        }
                        GameMode::GameOver => {
                            println!("UNLOCKING MOUSE");
                            MouseLook::unlock.run(world);
                        }
                        GameMode::Title => {
                            RapierPhysicsManager::despawn.run(world);
                            RapierPhysicsManager::fixed_update(world);
                            world
                                .get_component_mut::<Camera>(title_camera)
                                .unwrap()
                                .enabled = true;
                            if let Some(player_camera_entity) = player_camera_entity {
                                world
                                    .get_component_mut::<Camera>(player_camera_entity)
                                    .unwrap()
                                    .enabled = false;
                            }

                            let camera_transform =
                                world.get_component_mut::<Transform>(title_camera).unwrap();
                            let distance = 500.0;
                            let (sin, cos) = camera_rotation_angle.sin_cos();
                            camera_rotation_angle += 0.0004;
                            let y_offset = camera_rotation_angle * 5.0 * Vec3::Y;
                            *camera_transform = camera_transform
                                .with_position(Vec3::new(cos, 0.0, sin) * distance + y_offset)
                                .looking_at(Vec3::Y * 750.0 + y_offset, Vec3::Y);
                        }
                        _ => {}
                    }
                    // Perform physics and game related updates here.
                }
                Event::Draw => {
                    /*
                    (|cameras: Query<(&Transform, &Camera)>| {
                        for camera in cameras.iter() {
                            if camera.1.enabled {
                                println!("TRANSFORM: {:?}", camera.0);
                            }
                        }
                    }).run(world);
                    */
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

#[derive(Component, Clone)]
pub struct Powerup {
    collected: bool,
    grants_rockets: bool,
    grants_cable_length: bool,
}
fn collect_powerups(
    (player_transform, character_controller): (&GlobalTransform, &mut CharacterController),
    mut powerups: Query<(&mut Transform, &mut Powerup)>,
) {
    for powerup in powerups.iter_mut() {
        if !powerup.1.collected {
            let l = (powerup.0.position - player_transform.position).length_squared();
            let collected = if powerup.1.grants_cable_length {
                l < 8.0 * 8.0
            } else {
                l < 4.0 * 4.0
            };
            if collected {
                println!("COLLECT POWERUP");
                powerup.1.collected = true;

                // Cheap way to hide it
                powerup.0.scale = Vec3::ZERO;
                if powerup.1.grants_rockets {
                    character_controller.can_shoot = true;
                }
                if powerup.1.grants_cable_length {
                    character_controller.max_cable_length += 20.0;
                }
            }
        }
    }
}

fn reset_powerups(mut powerups: Query<(&mut Transform, &mut Powerup)>) {
    for powerup in powerups.iter_mut() {
        powerup.1.collected = false;
        powerup.0.scale = Vec3::ZERO;
    }
}
