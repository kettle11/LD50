use crate::*;

#[derive(Component, Clone)]
pub struct ExplosionManager {
    explosion_mesh: Handle<Mesh>,
    explosions_queue: Vec<ExplosionData>,
    colors: Vec<Color>,
    colliders_intersected: Vec<rapier3d::prelude::ColliderHandle>,
}

#[derive(Clone)]
pub struct ExplosionData {
    pub center: Vec3,
    pub scale: f32,
}

#[derive(Clone, Component)]
pub struct ExplosionPiece {
    spawned_pieces: u32,
}

impl ExplosionManager {
    pub fn setup_system(world: &mut World) {
        let low_poly_uv_sphere = (|meshes: &mut Assets<Mesh>, graphics: &mut Graphics| {
            meshes.add(Mesh::new(graphics, uv_sphere(4, 4, Vec2::ONE)))
        })
        .run(world);

        world.spawn(Self {
            explosion_mesh: low_poly_uv_sphere,
            explosions_queue: Vec::new(),
            colors: vec![
                Color::new(212. / 255., 39. / 255., 15. / 255., 1.0),
                Color::new(239. / 255., 249. / 255., 126. / 255., 1.0),
                Color::new(204. / 255., 137. / 255., 75. / 255., 1.0),
            ],
            colliders_intersected: Vec::new(),
        });
    }

    pub fn fixed_update_system(
        &mut self,
        commands: &mut Commands,
        mut explosion_pieces: Query<(&Transform, &mut ExplosionPiece)>,
        mut rigid_bodies: Query<(&mut Transform, &RapierRigidBody, &mut RigidBody)>,
        rapier_physics_manager: &mut RapierPhysicsManager,
    ) {
        fn spawn_piece(
            colors: &[Color],
            spawned_pieces: u32,
            center: Vec3,
            scale: f32,
            random: &mut Random,
            mesh: Handle<Mesh>,
            commands: &mut Commands,
        ) {
            commands.spawn((
                *random.select_from_slice(colors),
                Temporary(5 * 1),
                mesh,
                Material::UNLIT,
                Transform::new()
                    .with_position(center)
                    .with_scale(Vec3::fill(scale)),
                ExplosionPiece { spawned_pieces },
            ))
        }
        let mut random = Random::new();

        for ExplosionData { center, scale } in self.explosions_queue.drain(..) {
            for _ in 0..random.range_u32(2..4) {
                spawn_piece(
                    &self.colors,
                    random.range_u32(3..30),
                    center,
                    scale,
                    &mut random,
                    self.explosion_mesh.clone(),
                    commands,
                );

                let falloff = scale * 5.0;
                self.colliders_intersected.clear();
                rapier_physics_manager
                    .query_pipeline
                    .intersections_with_shape(
                        &rapier_physics_manager.collider_set,
                        &rapier3d::math::Isometry::translation(center.x, center.y, center.z),
                        &rapier3d::prelude::Ball::new(falloff),
                        rapier3d::prelude::InteractionGroups::all(),
                        None,
                        |handle| {
                            self.colliders_intersected.push(handle);
                            true
                        },
                    );

                for collider_handle in self.colliders_intersected.iter() {
                    let collider = rapier_physics_manager
                        .collider_set
                        .get(*collider_handle)
                        .unwrap();
                    let collider_center = collider.position();

                    let center: [f32; 3] = center.into();
                    let center: rapier3d::prelude::nalgebra::Point3<f32> = center.into();
                    /*/
                    let ray = rapier3d::prelude::Ray::new(
                        center,
                        &rapier3d::prelude::nalgebra::Point3::new(
                            collider_center.translation.x,
                            collider_center.translation.y,
                            collider_center.translation.z,
                        ) - center,
                    );

                    rapier_physics_manager.query_pipeline.cast_ray(
                        &[collider],
                        &ray,
                        100.,
                        true,
                        rapier3d::prelude::InteractionGroups::all(),
                        None,
                    );
                    */

                    let dir = &rapier3d::prelude::nalgebra::Point3::new(
                        collider_center.translation.x,
                        collider_center.translation.y,
                        collider_center.translation.z,
                    ) - center;
                    let max_force = 4.0 * scale;

                    if let Some(parent) = rapier_physics_manager
                        .collider_set
                        .get(*collider_handle)
                        .unwrap()
                        .parent()
                    {
                        let mag = dir.magnitude();
                        let falloff = 1.0 - (mag / falloff);
                        let falloff = (falloff * falloff).min(1.0);
                        let mut impulse = dir.normalize() * falloff * max_force;

                        if impulse.y > 0.0 {
                            impulse.y *= 2.0;
                        }
                        rapier_physics_manager
                            .rigid_body_set
                            .get_mut(parent)
                            .unwrap()
                            .apply_impulse(impulse, true);

                        rapier_physics_manager
                            .rigid_body_set
                            .get_mut(parent)
                            .unwrap()
                            .apply_torque_impulse(impulse / 3., true);
                    }
                }
            }
        }

        for (entity, piece) in explosion_pieces.entities_and_components_mut() {
            if piece.1.spawned_pieces > 0 && random.f32() < 0.8 {
                piece.1.spawned_pieces -= 1;
                spawn_piece(
                    &self.colors,
                    random.range_u32(0..piece.1.spawned_pieces),
                    piece.0.position
                        + random.point_in_unit_sphere() * piece.0.scale.x * random.f32() * 2.0,
                    piece.0.scale.x * random.f32(),
                    &mut random,
                    self.explosion_mesh.clone(),
                    commands,
                );
            }
        }
    }

    pub fn new_explosion(&mut self, center: Vec3, scale: f32) {
        self.explosions_queue.push(ExplosionData { center, scale })
    }
}
