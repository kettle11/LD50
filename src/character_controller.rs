use crate::*;

#[derive(Component, Clone)]
pub struct CharacterController;

impl CharacterController {
    pub fn fixed_update(
        input: &Input,
        rapier_physics: &mut RapierPhysicsManager,
        mut controlled: Query<(
            &mut Transform,
            &CharacterController,
            &mut RigidBody,
            &RapierCollider,
        )>,
    ) {
        for (transform, _controller, rigid_body, rapier_collider) in controlled.iter_mut() {
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
            let air_acceleration = 0.07;
            let acceleration = if grounded {
                ground_acceleration
            } else {
                air_acceleration
            };
            let mut forward = transform.forward();
            forward.y = 0.0;
            forward = forward.normalized();

            let mut right = transform.right();
            right.y = 0.0;
            right = right.normalized();
            if input.key(Key::W) {
                rigid_body.velocity += forward * acceleration;
            }
            if input.key(Key::S) {
                rigid_body.velocity -= forward * acceleration;
            }
            if input.key(Key::A) {
                rigid_body.velocity -= right * acceleration;
            }
            if input.key(Key::D) {
                rigid_body.velocity += right * acceleration;
            }

            let mut shape_velocity = rigid_body.velocity;
            shape_velocity.x = 0.0;
            shape_velocity.y = 0.0;
            //let shape_velocity: [f32; 3] = shape_velocity.into();

            use rapier3d::prelude::*;

            let ray = Ray::new(
                point![
                    transform.position.x,
                    transform.position.y,
                    transform.position.z
                ],
                vector![0.0, -1.0, 0.0],
            );

            if grounded || true {
                if input.key_down(Key::Space) {
                    rigid_body.velocity += Vec3::Y * 4.0;
                }
            }
        }
    }
}
