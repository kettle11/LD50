use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use koi::*;
use rapier3d::{
    math::Isometry,
    na::UnitQuaternion,
    prelude::{ColliderHandle, QueryPipeline, SharedShape},
};

#[derive(Component, Clone)]
struct Controlled;

#[derive(Clone)]
pub struct RigidBodyInner {
    pub kinematic: bool,
    pub velocity: Vec3,
    pub can_rotate: (bool, bool, bool),
    pub gravity_scale: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
}
#[derive(Component, Clone)]
pub struct RigidBody {
    rigid_body_inner: RigidBodyInner,
    mutated: bool,
    pub mutated_velocity: bool,
    pub mutated_position: bool,
}

impl RigidBody {
    pub fn new(inner: RigidBodyInner) -> Self {
        Self {
            rigid_body_inner: inner,
            mutated: true,
            mutated_velocity: true,
            mutated_position: true,
        }
    }
}

impl Deref for RigidBody {
    type Target = RigidBodyInner;
    fn deref(&self) -> &Self::Target {
        &self.rigid_body_inner
    }
}

impl DerefMut for RigidBody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mutated = true;
        &mut self.rigid_body_inner
    }
}
impl Default for RigidBodyInner {
    fn default() -> Self {
        Self {
            kinematic: false,
            velocity: Vec3::ZERO,
            can_rotate: (true, true, true),
            gravity_scale: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
        }
    }
}

#[derive(Component, Clone)]
pub enum Collider {
    Cuboid(Vec3),
    Sphere(f32),
    AttachedMesh,
    /// Use this for convex meshes.
    AttachedMeshConvex,
    /// Use this for complex concave meshes that will be moving.
    /// This takes a bit to calculate and can sometimes crash Rapier. :(
    AttachedMeshConvexDecomposition,
}

#[derive(Component, Clone)]
pub struct RapierRigidBody(pub rapier3d::prelude::RigidBodyHandle);

#[derive(Component, Clone)]
pub struct RapierCollider(pub rapier3d::prelude::ColliderHandle);

#[derive(NotCloneComponent)]
pub struct RapierPhysicsManager {
    pub gravity: Vec3,
    pub rigid_body_set: rapier3d::prelude::RigidBodySet,
    pub collider_set: rapier3d::prelude::ColliderSet,
    pub integration_parameters: rapier3d::prelude::IntegrationParameters,
    pub physics_pipeline: rapier3d::prelude::PhysicsPipeline,
    pub island_manager: rapier3d::prelude::IslandManager,
    pub broad_phase: rapier3d::prelude::BroadPhase,
    pub narrow_phase: rapier3d::prelude::NarrowPhase,
    pub joint_set: rapier3d::prelude::JointSet,
    pub ccd_solver: rapier3d::prelude::CCDSolver,
    pub cached_mesh_colliders: HashMap<Handle<Mesh>, SharedShape>,
    pub query_pipeline: QueryPipeline,
    user_data_to_entity: Vec<Entity>,
}

impl RapierPhysicsManager {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::Y * -9.81,
            rigid_body_set: rapier3d::prelude::RigidBodySet::new(),
            collider_set: rapier3d::prelude::ColliderSet::new(),
            integration_parameters: rapier3d::prelude::IntegrationParameters::default(),
            physics_pipeline: rapier3d::prelude::PhysicsPipeline::default(),
            island_manager: rapier3d::prelude::IslandManager::new(),
            broad_phase: rapier3d::prelude::BroadPhase::new(),
            narrow_phase: rapier3d::prelude::NarrowPhase::new(),
            joint_set: rapier3d::prelude::JointSet::new(),
            ccd_solver: rapier3d::prelude::CCDSolver::new(),
            cached_mesh_colliders: HashMap::new(),
            query_pipeline: QueryPipeline::new(),
            user_data_to_entity: Vec::new(),
        }
    }

    pub fn fixed_update(world: &mut World) {
        (Self::add_rapier_rigid_bodies).run(world);
        apply_commands(world);
        (Self::add_rapier_colliders).run(world);
        apply_commands(world);
        (Self::step).run(world);
    }

    pub fn add_rapier_rigid_bodies(
        &mut self,
        commands: &mut Commands,
        needs_rigid_body_query: Query<(&Transform, &RigidBody), Without<RapierRigidBody>>,
    ) {
        // Add rigid bodies to entities that need them.
        for (entity, (transform, rigid_body)) in needs_rigid_body_query.entities_and_components() {
            let position = transform.position;
            let position: [f32; 3] = position.into();

            let mut new_rigid_body = if rigid_body.kinematic {
                rapier3d::prelude::RigidBodyBuilder::new_kinematic_position_based()
                    .translation(position.into())
                    .build()
            } else {
                rapier3d::prelude::RigidBodyBuilder::new_dynamic()
                    .translation(position.into())
                    .build()
            };

            // This doesn't work in the Rapier on crates.io. It was fixed in September.
            /*
            new_rigid_body.restrict_rotations(
                rigid_body.can_rotate.0,
                rigid_body.can_rotate.1,
                rigid_body.can_rotate.2,
                false,
            );
            */
            new_rigid_body.set_linear_damping(rigid_body.linear_damping);
            new_rigid_body.set_angular_damping(rigid_body.angular_damping);

            if !rigid_body.can_rotate.0 || !rigid_body.can_rotate.1 || !rigid_body.can_rotate.2 {
                new_rigid_body.lock_rotations(true, false);
            }

            new_rigid_body.user_data = self.user_data_to_entity.len() as u128;
            self.user_data_to_entity.push(*entity);

            commands.add_component(
                *entity,
                RapierRigidBody(self.rigid_body_set.insert(new_rigid_body)),
            )
        }
    }

    pub fn add_rapier_colliders(
        &mut self,
        meshes: &Assets<Mesh>,
        commands: &mut Commands,
        needs_collider_query: Query<
            (
                &GlobalTransform,
                &Collider,
                Option<&RapierRigidBody>,
                Option<&Handle<Mesh>>,
            ),
            (Without<RapierCollider>, With<Transform>),
        >,
    ) {
        // Add colliders to entities that need them.
        for (entity, (transform, collider, rapier_rigid_body, mesh_handle)) in
            needs_collider_query.entities_and_components()
        {
            let scale = transform.scale;
            let mut collider = match collider {
                Collider::Cuboid(extents) => rapier3d::prelude::ColliderBuilder::cuboid(
                    extents.x * scale.x,
                    extents.y * scale.y,
                    extents.z * scale.z,
                )
                .build(),
                Collider::Sphere(radius) => {
                    rapier3d::prelude::ColliderBuilder::ball(*radius * scale.x).build()
                }
                Collider::AttachedMeshConvex => {
                    let mesh_handle = mesh_handle.unwrap();
                    let entry = self.cached_mesh_colliders.entry(mesh_handle.clone());
                    let shared_shape = entry.or_insert_with(|| {
                        let mesh = meshes.get(mesh_handle);
                        let mesh_data = mesh.mesh_data.as_ref().unwrap();

                        // Rapier does not support scaling mesh colliders, so create a new one each time.
                        let mut vertex_positions = Vec::new();
                        for vertex in mesh_data.positions.iter() {
                            let p = transform.scale.mul_by_component(*vertex);
                            let p: [f32; 3] = p.into();
                            vertex_positions.push(p.into());
                        }
                        SharedShape::convex_hull(
                            // This is safe as long as rapier3d is in f32 mode.
                            &vertex_positions,
                        )
                        .unwrap()
                    });
                    rapier3d::prelude::ColliderBuilder::new(shared_shape.clone()).build()
                }
                Collider::AttachedMesh => {
                    let mesh_handle = mesh_handle.unwrap();
                    let entry = self.cached_mesh_colliders.entry(mesh_handle.clone());
                    let shared_shape = entry.or_insert_with(|| {
                        let mesh = meshes.get(mesh_handle);
                        let mesh_data = mesh.mesh_data.as_ref().unwrap();

                        // Rapier does not support scaling mesh colliders, so create a new one each time.
                        let mut vertex_positions = Vec::new();
                        for vertex in mesh_data.positions.iter() {
                            let p = transform.scale.mul_by_component(*vertex);
                            let p: [f32; 3] = p.into();
                            vertex_positions.push(p.into());
                        }
                        SharedShape::trimesh(
                            // This is safe as long as rapier3d is in f32 mode.
                            vertex_positions,
                            mesh_data.indices.clone(),
                        )
                    });
                    rapier3d::prelude::ColliderBuilder::new(shared_shape.clone()).build()
                }
                Collider::AttachedMeshConvexDecomposition => {
                    let mesh_handle = mesh_handle.unwrap();
                    let entry = self.cached_mesh_colliders.entry(mesh_handle.clone());
                    let shared_shape = entry.or_insert_with(|| {
                        let mesh = meshes.get(mesh_handle);
                        let mesh_data = mesh.mesh_data.as_ref().unwrap();

                        // Rapier does not support scaling mesh colliders, so create a new one each time.
                        let mut vertex_positions = Vec::new();
                        for vertex in mesh_data.positions.iter() {
                            let p = transform.scale.mul_by_component(*vertex);
                            let p: [f32; 3] = p.into();
                            vertex_positions.push(p.into());
                        }

                        SharedShape::convex_decomposition(
                            // This is safe as long as rapier3d is in f32 mode.
                            &vertex_positions,
                            &mesh_data.indices.clone(),
                        )
                    });
                    rapier3d::prelude::ColliderBuilder::new(shared_shape.clone()).build()
                }
            };
            collider.user_data = self.user_data_to_entity.len() as u128;
            self.user_data_to_entity.push(*entity);

            let collider_handle = if let Some(rapier_rigid_body) = rapier_rigid_body {
                self.collider_set.insert_with_parent(
                    collider.clone(),
                    rapier_rigid_body.0,
                    &mut self.rigid_body_set,
                )
            } else {
                // This is a standalone collider without a position.
                let p: [f32; 3] = transform.position.into();
                let rotation: [f32; 4] = transform.rotation.into();
                collider.set_position(Isometry::from_parts(
                    p.into(),
                    UnitQuaternion::from_quaternion(rotation.into()),
                ));

                self.collider_set.insert(collider)
            };

            commands.add_component(*entity, RapierCollider(collider_handle))
        }
    }

    /*
    pub fn sync_with_rapier(
        &mut self,
        rigid_bodies: &mut Query<(&mut Transform, &RapierRigidBody, &mut RigidBody)>,
        rapier_rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    ) {
        let rigid_body_ref = self.rigid_body_set.get(rapier_rigid_body_handle).unwrap();
        let entity = self.user_data_to_entity[rigid_body_ref.user_data as usize];
        let (transform, _, r) = rigid_bodies.get_entity_components_mut(entity).unwrap();

        let current_position: [f32; 3] = rigid_body_ref
            .position()
            .transform_point(&rapier3d::prelude::nalgebra::Point3::new(0.0, 0.0, 0.0))
            .into();
        let current_rotation: [f32; 4] = rigid_body_ref.rotation().coords.into();
        transform.position = current_position.into();

        if r.can_rotate.0 || r.can_rotate.1 || r.can_rotate.2 {
            transform.rotation = Quat::from_xyzw(
                current_rotation[0],
                current_rotation[1],
                current_rotation[2],
                current_rotation[3],
            );
        }
        let linvel: [f32; 3] = (*rigid_body_ref.linvel()).into();
        r.velocity = linvel.into();
    }
    */

    pub fn step(
        &mut self,
        mut rigid_body_query: Query<(&mut Transform, &RapierRigidBody, &mut RigidBody)>,
    ) {
        // Update the transform of rigid bodies that have moved.
        for (transform, rigid_body, rigid_body_koi) in rigid_body_query.iter_mut() {
            if self.rigid_body_set.contains(rigid_body.0) {
                if transform.position.length() > 1000.0 {
                    // println!("DESPAWNING");
                    self.rigid_body_set.remove(
                        rigid_body.0,
                        &mut self.island_manager,
                        &mut self.collider_set,
                        &mut self.joint_set,
                    );
                    continue;
                }

                let velocity = rigid_body_koi.velocity;
                let velocity: [f32; 3] = velocity.into();

                let position = transform.position;
                let position: [f32; 3] = position.into();

                let rigid_body_ref = self.rigid_body_set.get_mut(rigid_body.0).unwrap();

                if rigid_body_koi.mutated_velocity {
                    rigid_body_ref.set_linvel(velocity.into(), true);
                }
                if rigid_body_koi.mutated_position {
                    let [x, y, z, w] = transform.rotation.as_array();
                    let q = rapier3d::prelude::nalgebra::Unit::<
                        rapier3d::prelude::nalgebra::Quaternion<f32>,
                    >::from_quaternion(
                        rapier3d::prelude::nalgebra::Quaternion::from_parts(
                            w,
                            rapier3d::prelude::nalgebra::Vector3::new(x, y, z),
                        ),
                    );
                    rigid_body_ref.set_position(position.into(), true);
                    rigid_body_ref.set_rotation(q.vector().into(), true);
                }

                // if current_position != position
                //     || current_rotation != rotation
                //     || current_velocity != velocity
                //     || rigid_body_koi.gravity_scale != rigid_body_ref.gravity_scale()
                if rigid_body_koi.mutated {
                    // to_angle_axis might not work correctly.
                    // it definitely fails for the identity rotation.
                    rigid_body_ref.set_gravity_scale(rigid_body_koi.gravity_scale, true);
                }
            }
        }

        let gravity: [f32; 3] = self.gravity.into();
        let gravity = gravity.into();
        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );

        for (transform, rigid_body, r) in rigid_body_query.iter_mut() {
            // Don't update the position of kinematic rigid bodies.
            if r.kinematic {
                continue;
            }

            if let Some(rigid_body_ref) = self.rigid_body_set.get(rigid_body.0) {
                let current_position: [f32; 3] = rigid_body_ref
                    .position()
                    .transform_point(&rapier3d::prelude::nalgebra::Point3::new(0.0, 0.0, 0.0))
                    .into();
                let current_rotation: [f32; 4] = rigid_body_ref.rotation().coords.into();
                transform.position = current_position.into();

                if r.can_rotate.0 || r.can_rotate.1 || r.can_rotate.2 {
                    transform.rotation = Quat::from_xyzw(
                        current_rotation[0],
                        current_rotation[1],
                        current_rotation[2],
                        current_rotation[3],
                    );
                }
                let linvel: [f32; 3] = (*rigid_body_ref.linvel()).into();
                r.velocity = linvel.into();
                r.mutated = false;
                r.mutated_velocity = false;
                r.mutated_position = false;
            }
        }
    }
}
