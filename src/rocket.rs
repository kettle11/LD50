use rapier3d::math::Isometry;

use crate::*;

#[derive(Component, Clone)]
pub struct Rocket {
    velocity: Vec3,
}

pub fn spawn_rocket(commands: &mut Commands, start: Vec3, direction: Vec3) {
    commands.spawn((
        Mesh::SPHERE,
        // RigidBody::new(RigidBodyInner {
        //     gravity_scale: 0.0,
        //     velocity: direction * 20.0,
        //     ..Default::default()
        // }),
        Material::UNLIT,
        Color::YELLOW,
        Transform::new()
            .with_position(start + direction * 2.0)
            .with_scale(Vec3::fill(0.2)),
        Collider::Sphere(0.5),
        Rocket {
            velocity: direction * 40.0,
        },
    ))
}

pub fn check_rocket_collisions_system(
    commands: &mut Commands,
    rapier_physics: &RapierPhysicsManager,
    explosion_manager: &mut ExplosionManager,
    mut rockets: Query<(&mut Transform, &Rocket, &RapierCollider)>,
    time: &Time,
) {
    for (entity, (transform, rocket, rapier_collider)) in rockets.entities_and_components_mut() {
        transform.position += rocket.velocity * time.fixed_time_step as f32;
        // rigid_body.mutated_position = true;

        /*
        for contact in rapier_physics.narrow_phase.contacts_with(rapier_collider.0) {
            println!("COLLIDER: {:?}", rapier_collider.0);

            println!("CONTACT: {:?}", (contact.collider1, contact.collider2));
            commands.add_component(*entity, ToDespawn);
            explosion_manager.new_explosion(transform.position, 5.0);
            break;
        }
        */

        let velocity: [f32; 3] = rocket.velocity.into();

        let v = velocity.into();
        let hit_something = rapier_physics
            .query_pipeline
            .cast_shape(
                &rapier_physics.collider_set,
                &Isometry::translation(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ),
                &v,
                &rapier3d::prelude::Ball::new(transform.scale.x * 0.5),
                0.1,
                rapier3d::prelude::InteractionGroups::all(),
                Some(&|c| c != rapier_collider.0),
            )
            .is_some();
        if hit_something {
            commands.add_component(*entity, ToDespawn);
            explosion_manager.new_explosion(transform.position, 5.0);
        }
    }
}
