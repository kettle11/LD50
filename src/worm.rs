use crate::*;

#[derive(Component, Clone)]
struct Worm;

pub fn run_worm(world: &mut World) {
    let worm_count = world.query::<&Worm>().unwrap().iter().count();
    //println!("WORM COUNT: {:?}", worm_count);
    if worm_count == 0 {
        let mut worm_body = Handle::default();
        let mut worm_teeth = Handle::default();
        let mut worm_inner_teeth = Handle::default();
        (|meshes: &mut Assets<Mesh>, graphics: &mut Graphics| {
            let mut mesh_data = MeshData::new();

            create_worm_cylinder(
                &mut mesh_data,
                -Vec3::Y * 600.0,
                Vec3::ZERO,
                30,
                250.0,
                true,
            );
            worm_body = meshes.add(Mesh::new(graphics, mesh_data));

            let mut mesh_data = MeshData::new();

            let mut radius = 330.0;
            let mut y_offset = Vec3::ZERO;
            let mut resolution = 30;
            let mut twist = 0.25;
            for _ in 0..1 {
                create_worm_teeth(&mut mesh_data, Vec3::Y, y_offset, resolution, radius, twist);
                // radius *= 0.5;
                y_offset -= Vec3::Y * 150.0;
                resolution -= 5;
                twist += 0.05
            }

            worm_teeth = meshes.add(Mesh::new(graphics, mesh_data));
            let mut mesh_data = MeshData::new();

            create_worm_teeth(&mut mesh_data, Vec3::Y, y_offset, resolution, radius, twist);
            // radius *= 0.5;
            y_offset -= Vec3::Y * 150.0;
            resolution -= 5;
            twist += 0.05;

            worm_inner_teeth = meshes.add(Mesh::new(graphics, mesh_data));
        })
        .run(world);

        println!("SPAWNING");
        let body = world.spawn((
            worm_body,
            Transform::new().with_position(Vec3::Y * -100.0),
            Material::DEFAULT,
            Color::from_srgb_hex(0x4B0082, 1.0),
            Worm,
        ));

        let teeth = world.spawn((
            worm_teeth,
            Transform::new(),
            Material::DEFAULT,
            Color::from_srgb_hex(0x4B0082, 1.0),
        ));

        let inner_teeth = world.spawn((
            worm_inner_teeth,
            Transform::new(),
            Material::UNLIT,
            Color::interpolate(
                Color::RED.with_lightness(0.1).with_chroma(0.7),
                Color::from_srgb_hex(0x4B0082, 1.0),
                0.5,
            ),
        ));

        set_parent(world, Some(body), teeth);
        set_parent(world, Some(body), inner_teeth);
    }

    // Move the worm

    (|(transform, _worm): (&mut Transform, &Worm),
      player: (&GlobalTransform, &CharacterController)| {
        transform.position.y += 0.13;

        let player_diff = player.0.position.y - (transform.position.y - 120.0);

        println!("PLAYER DIFF: {:?}", player_diff);
        if player_diff < 0.0 {
            println!("PLAYER LOSES");
        }
    })
    .run(world);
}

pub fn create_worm_teeth(
    mesh_data: &mut MeshData,
    dir: Vec3,
    end: Vec3,
    resolution: u32,
    radius: f32,
    twist: f32,
) {
    let MeshData {
        positions,
        indices,
        normals,
        texture_coordinates,
        colors,
    } = mesh_data;

    let other_dir = if dir.abs() != Vec3::X {
        Vec3::X
    } else {
        Vec3::Z
    };
    let right = dir.cross(other_dir).normalized() * radius;
    let forward = dir.cross(right).normalized() * radius;

    let mut increment = std::f32::consts::PI * 2.0 / resolution as f32;
    let mut current_angle = 0.;

    /*
    if reverse {
        increment *= -1.0;
    }


    let mut top_positions = Vec::new();

    for _ in 0..resolution {
        current_angle += increment;
        let (sin, cos) = current_angle.sin_cos();
        top_positions.push(end + right * cos + forward * sin);
    }

    let mut last = *top_positions.last().unwrap();
    */

    let mut current_angle: f32 = 0.;

    for _ in 0..resolution {
        let (sin, cos) = (current_angle).sin_cos();
        let p0 = end + right * cos + forward * sin;

        let (sin, cos) = (current_angle + std::f32::consts::TAU * twist).sin_cos();
        let p1 = end + right * cos + forward * sin;

        let (sin, cos) = (current_angle + std::f32::consts::TAU * twist * 2.0).sin_cos();
        let p2 = end + right * cos + forward * sin;
        let p2 = (p2 - p1) / 2.0 + p1 + Vec3::Y * 40.0;

        let new_vertex = positions.len() as u32;
        positions.push(p0);
        positions.push(p1);
        positions.push(p2);

        normals.push(Vec3::Y);
        normals.push(Vec3::Y);
        normals.push(Vec3::Y);

        indices.push([new_vertex, new_vertex + 1, new_vertex + 2]);
        indices.push([new_vertex + 2, new_vertex + 1, new_vertex]);

        current_angle += increment;
    }
}

/*
pub fn create_worm_teeth(
    mesh_data: &mut MeshData,
    start: Vec3,
    end: Vec3,
    resolution: u32,
    radius: f32,
    reverse: bool,
) {
    let MeshData {
        positions,
        indices,
        normals,
        texture_coordinates,
        colors,
    } = mesh_data;

    let dir = (start - end).normalized();
    let other_dir = if dir.abs() != Vec3::X {
        Vec3::X
    } else {
        Vec3::Z
    };
    let right = dir.cross(other_dir).normalized() * radius;
    let forward = dir.cross(right).normalized() * radius;

    let mut increment = std::f32::consts::PI * 2.0 / resolution as f32;
    let mut current_angle = 0.;

    if reverse {
        increment *= -1.0;
    }

    let mut top_positions = Vec::new();

    for _ in 0..resolution {
        current_angle += increment;
        let (sin, cos) = current_angle.sin_cos();
        top_positions.push(end + right * cos + forward * sin);
    }

    let mut last = *top_positions.last().unwrap();

    for current in top_positions {
        let new_vertex = positions.len() as u32;
        positions.push(last);
        positions.push(current);
        positions.push((current - last) / 2.0 + last + Vec3::Y * 50.0);

        normals.push(Vec3::Y);
        normals.push(Vec3::Y);
        normals.push(Vec3::Y);

        indices.push([new_vertex, new_vertex + 1, new_vertex + 2]);
        indices.push([new_vertex + 2, new_vertex + 1, new_vertex]);

        last = current;
    }
}*/

pub fn create_worm_cylinder(
    mesh_data: &mut MeshData,
    start: Vec3,
    end: Vec3,
    resolution: u32,
    radius: f32,
    reverse: bool,
) {
    let MeshData {
        positions,
        indices,
        normals,
        texture_coordinates,
        colors,
    } = mesh_data;

    let center = start;
    let dir = (start - end).normalized();
    let other_dir = if dir.abs() != Vec3::X {
        Vec3::X
    } else {
        Vec3::Z
    };
    let right = dir.cross(other_dir).normalized() * radius;
    let forward = dir.cross(right).normalized() * radius;

    let mut increment = std::f32::consts::PI * 2.0 / resolution as f32;
    let mut current_angle = 0.;

    if reverse {
        increment *= -1.0;
    }

    let start_index = positions.len() as u32;
    for _ in 0..resolution {
        current_angle += increment;

        let new_vertex = positions.len() as u32;
        let (sin, cos) = current_angle.sin_cos();
        let offset = right * cos + forward * sin;
        positions.push(center + right * cos + forward * sin);
        positions.push(end + right * cos + forward * sin);

        normals.push(offset.normalized());
        normals.push(offset.normalized());

        indices.push([new_vertex, new_vertex + 1, new_vertex + 2]);
        indices.push([new_vertex + 2, new_vertex + 1, new_vertex + 3]);
    }

    let new_vertex = (positions.len() - 2) as u32;
    indices.pop();
    indices.pop();
    indices.push([new_vertex, new_vertex + 1, start_index]);
    indices.push([start_index, new_vertex + 1, start_index + 1]);
}
