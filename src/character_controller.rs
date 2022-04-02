use crate::*;

#[derive(Component, Clone)]
pub struct CharacterController {
    grapple_target: Entity,
    grapple_position: Option<(Vec3, f32)>,
}

#[derive(Component, Clone)]
pub struct GrappleTarget;

#[derive(Component, Clone)]
pub struct CharacterControllerCamera;

impl CharacterController {
    pub fn new(world: &mut World) -> Self {
        Self {
            grapple_target: world.spawn((
                Mesh::SPHERE,
                Transform::new(),
                Material::DEFAULT,
                Color::AZURE,
                GrappleTarget,
            )),
            grapple_position: None,
        }
    }

    pub fn fixed_update(
        input: &Input,
        rapier_physics: &mut RapierPhysicsManager,
        (camera_transform, camera, _): (&GlobalTransform, &Camera, &CharacterControllerCamera),
        mut controlled: Query<(
            &mut Transform,
            &mut CharacterController,
            &mut RigidBody,
            &RapierCollider,
        )>,
        time: &Time,
        (grapple_target_transform, _): (&mut Transform, &GrappleTarget),
        immediate_drawer: &mut ImmediateDrawer,
    ) {
        for (transform, character_controller, rigid_body, rapier_collider) in controlled.iter_mut()
        {
            rapier_physics.query_pipeline.update(
                &rapier_physics.island_manager,
                &rapier_physics.rigid_body_set,
                &rapier_physics.collider_set,
            );
            let grounded = rapier_physics
                .query_pipeline
                .cast_shape(
                    &rapier_physics.collider_set,
                    &Isometry::translation(
                        transform.position.x,
                        transform.position.y,
                        transform.position.z,
                    ),
                    &[0.0, -1.0, 0.0].into(),
                    &rapier3d::prelude::Ball::new(0.3),
                    0.3,
                    rapier3d::prelude::InteractionGroups::all(),
                    Some(&|c| c != rapier_collider.0),
                )
                .is_some();

            let ground_acceleration = 0.2;
            let air_acceleration = 0.1;
            let acceleration = if grounded {
                ground_acceleration
            } else {
                air_acceleration
            };
            let mut forward = camera_transform.forward();
            forward.y = 0.0;
            forward = forward.normalized();

            let mut right = camera_transform.right();
            right.y = 0.0;
            right = right.normalized();

            let horizontal_velocity = Vec3::new(rigid_body.velocity.x, 0.0, rigid_body.velocity.z);
            let max = 6.0;

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

            let mut shape_velocity = rigid_body.velocity;
            shape_velocity.x = 0.0;
            shape_velocity.y = 0.0;
            //let shape_velocity: [f32; 3] = shape_velocity.into();

            use rapier3d::prelude::*;

            if grounded || true {
                if input.key_down(Key::Space) {
                    rigid_body.velocity += Vec3::Y * 10.0;
                }
            }

            if input.pointer_button_down(PointerButton::Primary) {
                println!("CASTING RAY!");
                let (x, y) = input.pointer_position();
                let camera_ray = camera.view_to_ray(&camera_transform, x as f32, y as f32);
                let origin: [f32; 3] = camera_ray.origin.into();
                let direction: [f32; 3] = camera_ray.direction.into();

                let ray = Ray::new(origin.into(), direction.into());
                rapier_physics.query_pipeline.update(
                    &rapier_physics.island_manager,
                    &rapier_physics.rigid_body_set,
                    &rapier_physics.collider_set,
                );
                let result = rapier_physics.query_pipeline.cast_ray(
                    &rapier_physics.collider_set,
                    &ray,
                    300.0,
                    false,
                    rapier3d::prelude::InteractionGroups::all(),
                    Some(&|c| c != rapier_collider.0),
                );

                if let Some(result) = result {
                    let position = camera_ray.get_point(result.1);
                    println!(
                        "DISTANCE FROM CAMERA TO BODY: {:?}",
                        (camera_transform.position - transform.position).length()
                    );
                    println!("GRAPPLING");
                    grapple_target_transform.position = position;

                    immediate_drawer.set_color(Color::YELLOW);
                    immediate_drawer.draw_sphere_for_n_frames(
                        Transform::new().with_position(camera_ray.origin),
                        120 * 4,
                    );
                    //character_controller.grapple_position = Some((position, 2.0));
                }
            }

            if let Some((grapple_position, time_remaining)) =
                &mut character_controller.grapple_position
            {
                rigid_body.velocity += (*grapple_position - transform.position).normalized() * 0.2;
                *time_remaining -= time.delta_seconds_f64 as f32;
                if *time_remaining <= 0.0 {
                    character_controller.grapple_position = None;
                }
            }
        }
    }
}
