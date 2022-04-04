use rapier3d::parry::query::TOIStatus;

use crate::rocket::spawn_rocket;
use crate::*;

#[derive(Component, Clone)]
pub struct CharacterController {
    grapple_target: Entity,
    grapple_line: Entity,
    grapple_position: Option<(Vec3, f32)>,
    extra_jumps: usize,
    pub max_cable_length: f32,
    pub can_shoot: bool,
    jump_sound: Handle<Sound>,
    grapple_sound: Handle<Sound>,
    wind_sound: Handle<Sound>,
}

#[derive(Component, Clone)]
pub struct GrappleTarget;

#[derive(Component, Clone)]
pub struct GrappleLine;

#[derive(Component, Clone)]
pub struct CharacterControllerCamera;

pub const MAX_EXTRA_JUMPS: usize = 2;

impl CharacterController {
    pub fn new(world: &mut World) -> Self {
        let sounds = world.get_singleton::<Assets<Sound>>();
        let jump_sound = sounds.load("assets/jump.wav");
        let grapple_sound = sounds.load("assets/shoot_grapple.wav");
        let wind_sound = sounds.load("assets/wind_sound.wav");

        Self {
            grapple_target: world.spawn((
                Mesh::SPHERE,
                Transform::new(),
                Material::DEFAULT,
                Color::AZURE,
                GrappleTarget,
            )),
            grapple_line: world.spawn((
                Mesh::CYLINDER,
                Transform::new(),
                Material::DEFAULT,
                Color::BLACK.with_lightness(0.5),
                Cable::new(),
            )),
            max_cable_length: 50.0,
            grapple_position: None,
            extra_jumps: MAX_EXTRA_JUMPS,
            can_shoot: false,
            jump_sound,
            grapple_sound,
            wind_sound,
        }
    }

    pub fn reset(&mut self) {
        self.grapple_position = None;
        self.max_cable_length = 50.0;
        self.extra_jumps = MAX_EXTRA_JUMPS;
    }

    pub fn fixed_update(
        input: &Input,
        rapier_physics: &mut RapierPhysicsManager,
        (camera_transform, _camera, _, head_audio_source): (
            &GlobalTransform,
            &Camera,
            &CharacterControllerCamera,
            &mut AudioSource,
        ),
        mut controlled: Query<(
            &mut Transform,
            &mut CharacterController,
            &mut RigidBody,
            &RapierCollider,
            &mut AudioSource,
        )>,
        time: &Time,
        (grapple_target_transform, _): (&mut Transform, &GrappleTarget),
        (_grapple_line_transform, cable): (&mut Transform, &mut Cable),
        game_state: &mut GameState,
        explosion_manager: &mut ExplosionManager,
        commands: &mut Commands,
    ) {
        for (transform, character_controller, rigid_body, rapier_collider, audio_source) in
            controlled.iter_mut()
        {

            game_state.player_max_height = transform.position.y.max(game_state.player_max_height);

            if game_state.victory {
                return;
            }

            if transform.position.y < 0.0 {
                rigid_body.gravity_scale = -4.0;
            } else {
                rigid_body.gravity_scale = 1.0;
            }
            rapier_physics.query_pipeline.update(
                &rapier_physics.island_manager,
                &rapier_physics.rigid_body_set,
                &rapier_physics.collider_set,
            );
            let grounded = rapier_physics.query_pipeline.cast_shape(
                &rapier_physics.collider_set,
                &Isometry::translation(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ),
                &[0.0, -1.0, 0.0].into(),
                &rapier3d::prelude::Ball::new(1.0),
                0.3,
                rapier3d::prelude::InteractionGroups::all(),
                Some(&|c| c != rapier_collider.0),
            );

            if let Some(v) = grounded {
                let toi = v.1;
                //println!("TOI: {:?}", toi);
            }

            let grounded = grounded.is_some();

            if grounded {
                character_controller.extra_jumps = MAX_EXTRA_JUMPS;
            }

            let mut max = 6.0;

            let ground_acceleration = 0.2;
            let air_acceleration = 0.1;
            let mut acceleration = if grounded {
                ground_acceleration
            } else {
                air_acceleration
            };

            if character_controller.grapple_position.is_some() {
                acceleration = 0.2;
                max = 12.0;
            }
            let mut forward = camera_transform.forward();

            if character_controller.grapple_position.is_none() {
                forward.y = 0.0;
                forward = forward.normalized();
            }

            let mut right = camera_transform.right();
            if character_controller.grapple_position.is_none() {
                right.y = 0.0;
                right = right.normalized();
            }

            let horizontal_velocity = Vec3::new(rigid_body.velocity.x, 0.0, rigid_body.velocity.z);

            if input.key(Key::W) && horizontal_velocity.dot(forward) < max {
                rigid_body.velocity += forward * acceleration;
            }
            if input.key(Key::S) && horizontal_velocity.dot(-forward) < max {
                rigid_body.velocity -= forward * acceleration;
            }
            if input.key(Key::A) && horizontal_velocity.dot(-right) < max {
                rigid_body.velocity -= right * acceleration;
            }
            if input.key(Key::D) && horizontal_velocity.dot(right) < max {
                rigid_body.velocity += right * acceleration;
            }

            rigid_body.mutated_velocity = true;

            let mut shape_velocity = rigid_body.velocity;
            shape_velocity.x = 0.0;
            shape_velocity.y = 0.0;
            //let shape_velocity: [f32; 3] = shape_velocity.into();

            use rapier3d::prelude::*;

            let mut jumped = false;
            if grounded || character_controller.extra_jumps > 0 {
                if input.key_down(Key::Space) {
                    rigid_body.velocity += Vec3::Y * 5.0;
                    jumped = true;
                    character_controller.extra_jumps =
                        character_controller.extra_jumps.saturating_sub(1);
                    audio_source.play(&character_controller.jump_sound, false);
                }
            }

            let camera_ray = koi::Ray3::new(camera_transform.position, camera_transform.forward());
            let origin: [f32; 3] = camera_ray.origin.into();
            let direction: [f32; 3] = camera_ray.direction.into();

            let ray = Ray::new(origin.into(), direction.into());
            rapier_physics.query_pipeline.update(
                &rapier_physics.island_manager,
                &rapier_physics.rigid_body_set,
                &rapier_physics.collider_set,
            );

            let grapple_distance = character_controller.max_cable_length;
            let ray_cast = rapier_physics.query_pipeline.cast_ray(
                &rapier_physics.collider_set,
                &ray,
                grapple_distance,
                false,
                rapier3d::prelude::InteractionGroups::all(),
                Some(&|c| c != rapier_collider.0),
            );
            game_state.can_grapple = ray_cast.is_some();

            if character_controller.can_shoot {
                if input.pointer_button_down(PointerButton::Secondary) {
                    /*
                    if let Some(result) = ray_cast {
                        let position = camera_ray.get_point(result.1);
                        explosion_manager.new_explosion(position, 5.0);
                    }
                    */
                    println!("SPAWNING ROCKET");
                    spawn_rocket(
                        commands,
                        camera_transform.position,
                        camera_transform.forward(),
                    )
                }
            }

            /*
            // Hardcoded location for rocket powerup.
            if (transform.position - (Vec3::Y * 3191.0)).length() < 1.0 {
                println!("COLLECTED POWERUP");
                character_controller.can_shoot = true;
            }
            */

            if input.pointer_button_down(PointerButton::Primary) {
                if let Some(result) = ray_cast {
                    let position = camera_ray.get_point(result.1);

                    println!("GRAPPLING");
                    audio_source.play(&character_controller.grapple_sound, false);
                    grapple_target_transform.position = position;

                    /*
                    immediate_drawer.set_color(Color::YELLOW);
                    immediate_drawer.draw_sphere_for_n_frames(
                        Transform::new().with_position(camera_ray.origin),
                        120 * 4,
                    );

                    immediate_drawer.set_color(Color::CYAN);

                    immediate_drawer.draw_sphere_for_n_frames(
                        Transform::new()
                            .with_position(camera_ray.origin + camera_ray.direction * 20.),
                        120 * 4,
                    );
                    */
                    character_controller.grapple_position = Some((position, 2.0));

                    let velocity_along_direction =
                        camera_ray.direction.dot(rigid_body.velocity) * camera_ray.direction;
                    let velocity_not_along_direction =
                        rigid_body.velocity - velocity_along_direction;
                    rigid_body.velocity =
                        velocity_along_direction * 0.8 + velocity_not_along_direction * 0.7;
                    rigid_body.gravity_scale = 0.1;
                    rigid_body.mutated_velocity = true;
                }
            }

            if let Some((grapple_position, time_remaining)) =
                &mut character_controller.grapple_position
            {
                rigid_body.mutated_velocity = true;

                character_controller.extra_jumps = MAX_EXTRA_JUMPS;
                let diff = *grapple_position - transform.position;
                let dir_normalized = diff.normalized();
                rigid_body.velocity += dir_normalized * 0.8;
                rigid_body.velocity += camera_transform.forward() * 0.1;

                let max_grapple_velocity = 80.0;
                if rigid_body.velocity.length() > max_grapple_velocity {
                    rigid_body.velocity = rigid_body.velocity.normalized() * max_grapple_velocity;
                }
                *time_remaining -= time.delta_seconds_f64 as f32;

                cable.start = camera_transform.position
                    + camera_transform.right() * 0.3
                    + camera_transform.down() * 0.2;
                cable.end = *grapple_position;

                if diff.length() < 1.3 || jumped {
                    character_controller.grapple_position = None;
                    rigid_body.gravity_scale = 1.0;
                    cable.start = Vec3::ZERO;
                    cable.end = Vec3::ZERO;
                }
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct Cable {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
}

impl Cable {
    pub fn new() -> Self {
        Self {
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            radius: 0.04,
        }
    }
}

impl Cable {
    pub fn update_meshes_system(
        graphics: &mut Graphics,
        meshes: &mut Assets<Mesh>,
        mut cables: Query<(&mut Handle<Mesh>, &Cable)>,
    ) {
        for (mesh, cable) in cables.iter_mut() {
            *mesh = meshes.add(Mesh::new(
                graphics,
                cylinder(cable.start, cable.end, 6, cable.radius),
            ));
        }
    }
}
