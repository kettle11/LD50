use std::collections::HashMap;

use crate::*;

pub struct Terrain {
    pub scale: f32,
    size_xz: usize,
    size_y: usize,
    values: Vec<f32>,
    mesh_normal_calculator: MeshNormalCalculator,
    chunk_size: usize,
    chunks: HashMap<Vec3u, Entity>,
    world_offset: Vec3,
}

fn sample_with_octaves<const LANES: usize>(
    noise: &mut clatter::Simplex3d,
    persistence: f32,
    x: f32,
    y: f32,
    z: f32,
) -> f32
where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    let mut amplitudes: [f32; LANES] = [0.0; LANES];
    let mut frequencies: [f32; LANES] = [0.0; LANES];

    for i in 0..LANES {
        amplitudes[i] = amplitude;
        frequencies[i] = frequency;

        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    let amplitudes = core::simd::Simd::<f32, LANES>::from_array(amplitudes);
    let frequencies = core::simd::Simd::<f32, LANES>::from_array(frequencies);
    let sample = noise.sample([
        core::simd::Simd::<f32, LANES>::splat(x) * frequencies,
        core::simd::Simd::<f32, LANES>::splat(y) * frequencies,
        core::simd::Simd::<f32, LANES>::splat(z) * frequencies,
    ]) * amplitudes;

    sample.value.reduce_sum() / max_value
}

impl Terrain {
    pub fn new(size_xz: usize, size_y: usize) -> Self {
        let scale = 450.;
        let mut terrain = Self {
            scale,
            size_xz,
            size_y,
            values: vec![0.0; size_xz * size_xz * size_y],
            mesh_normal_calculator: MeshNormalCalculator::new(),
            chunk_size: 32,
            chunks: HashMap::new(),
            world_offset: -Vec3::Y * 50.0 - Vec3::XZ * scale / 2.0,
        };
        terrain.generate_height_data();
        terrain
    }
    pub fn generate_height_data(&mut self) {
        let scale = self.scale;

        // let noise = noise::Perlin::new();
        let mut noise = clatter::Simplex3d::new();

        let radius_squared = (scale / 2.0) * (scale / 2.0);
        let center = Vec3::fill(scale) / 2.0;

        let offset = Vec3::ZERO;

        let size_per_tile = scale / self.size_xz as f32;
        let mut index = 0;

        let top = size_per_tile * self.size_y as f32;

        for i in 0..self.size_xz {
            for j in 0..self.size_y {
                for k in 0..self.size_xz {
                    let p = Vec3::new(i as f32, j as f32, k as f32) * size_per_tile + offset;
                    let persistence = 0.5;

                    let mut sample = 0.0;
                    {
                        let p = p / 300.0 + Vec3::fill(2000.);

                        sample = (sample_with_octaves::<8>(&mut noise, persistence, p.x, p.y, p.z)
                            * 1.3) as f32;

                        // println!("SAMPLE: {:?}", sample);
                        /*
                        for _ in 0..5 {
                            let p = p * frequency;
                            sample += noise.get([p.x as f64, p.y as f64, p.z as f64]) * amplitude;

                            max_value += amplitude;
                            amplitude *= persistence;
                            frequency *= 2.0;
                        }*/
                    }

                    // println!("SAMPLE: {:?}", sample);
                    // println!("P: {:?}", p);

                    let p = p - center;
                    let distance_from_center = p.xz().length_squared();
                    //  println!("DISTANCE FROM CENTER: {:?}", distance_from_center);

                    let v = distance_from_center / radius_squared;
                    let scale_factor = if v > 0.7 {
                        // println!("V: {:?}", v);
                        let clamp = ((v - 0.7) / 0.3).clamp(0.0, 1.0) as f64;

                        // println!("CLAMP: {:?}", clamp);
                        clamp
                    } else {
                        0.0
                    };

                    // Height density
                    let j_asf32 = j as f32;
                    let percent_to_top = j_asf32 / top;
                    let density = ((1.0 - percent_to_top)
                        * (percent_to_top * std::f32::consts::TAU * 4.0).sin())
                    .max(0.4);

                    if (j as f32 * size_per_tile) > top * 0.8 {
                        sample = -1.0;
                        if v < 0.003 && (j as f32 * size_per_tile) < top * 0.9 {
                            sample = 1.0;
                        }
                    }

                    let sample = sample * density;

                    //  println!("SCALE FACTOR: {:?}", scale_factor);
                    let v = sample - scale_factor as f32 - 0.02;
                    /*
                    if p.x > 100.0 {
                        sample = 1.0
                    } else {
                        sample = -1.0;
                    }
                    */

                    self.values[index] = v as f32;
                    // values[i * (size * size) + j * size + k] = v;
                    index += 1;
                }
            }
        }
        //let values = calculate_values(scale, offset, size, center, radius_squared, &noise);
    }

    pub fn create_chunk_mesh(
        &mut self,
        offset: Vec3u,
        samples: usize,
        world_offset: Vec3,
    ) -> MeshData {
        //println!("SIZE Y: {:?}", self.size_y);
        let terrain_sampler = TerrainSampler {
            offset,
            values: &self.values,
            size_xz: self.size_xz,
            size_y: self.size_y,
            samples,
        };

        let mut chunk = isosurface::MarchingCubes::new(samples);
        let sampler = isosurface::sampler::Sampler::new(&terrain_sampler);

        let scale = (samples as f32 / self.size_xz as f32) * self.scale;
        let mut extractor = Extractor::new(scale, Vec3::ZERO, world_offset);

        chunk.extract(&sampler, &mut extractor);
        let mesh_data = extractor.mesh_data;

        // println!("NORMALS LEN: {:?}", mesh_data.normals.len());
        // println!("PSOITIONS LEN: {:?}", mesh_data.positions.len());

        // println!("MESH DATA: {:#?}", mesh_data);
        // self.mesh_normal_calculator
        //     .calculate_normals(&mut mesh_data);
        mesh_data
    }

    pub fn regenerate_chunk(&mut self, world: &mut World, chunk: Vec3u) {
        let mut to_spawn = Vec::new();

        (|graphics: &mut Graphics, meshes: &mut Assets<Mesh>| {
            let world_offset = (chunk.as_f32() * self.chunk_size as f32) / self.size_xz as f32
                * self.scale
                + self.world_offset;
            let mesh_data =
                self.create_chunk_mesh(chunk * self.chunk_size, self.chunk_size, world_offset);
            let has_a_tri = !mesh_data.indices.is_empty();
            let mesh = meshes.add(Mesh::new(graphics, mesh_data));

            if has_a_tri {
                to_spawn.push((
                    chunk,
                    (
                        mesh,
                        Material::DEFAULT,
                        Transform::new().with_position(world_offset),
                        Collider::AttachedMesh,
                    ),
                ));
            }
        })
        .run(world);

        for (key, to_spawn) in to_spawn {
            if let Some(replacing) = self.chunks.insert(key, world.spawn(to_spawn)) {
                let _ = world.despawn(replacing);
            }
        }
    }

    pub fn create_chunks(&mut self, world: &mut World) {
        println!("CHUNKS: {:?}", self.size_y as f32 / self.chunk_size as f32);
        let chunks_y = self.size_y / self.chunk_size;
        let chunks_xz = self.size_xz / self.chunk_size;

        for i in 0..chunks_xz {
            for j in 0..chunks_y {
                for k in 0..chunks_xz {
                    let chunk = Vec3u::new(i, j, k);
                    self.regenerate_chunk(world, chunk);
                }
            }
        }
    }
}

/*
pub fn generate_chunk(offset_y: usize) -> MeshData {
    let scale = 200.;
    let noise = noise::Perlin::new();
    let radius_squared = (scale / 2.0) * (scale / 2.0);
    let center = Vec3::fill(scale) / 2.0;
    /* let terrain_sampler = TerrainSampler {
        noise,
        scale,
        radius_squared: (scale / 2.0) * (scale / 2.0),
        center: Vec3::fill(scale) / 2.0,
        offset,
    };
    */
    let offset = Vec3::Y * offset_y as f32 * scale;
    let size = 128;
    let values = calculate_values(scale, offset, size, center, radius_squared, &noise);
    let terrain_sampler = TerrainSampler {
        values: &values,
        size,
    };

    let mut chunk = isosurface::MarchingCubes::new(128);
    let sampler = isosurface::sampler::Sampler::new(&terrain_sampler);

    let mut extractor = Extractor::new(scale, offset);

    chunk.extract(&sampler, &mut extractor);
    let mut mesh_normal_calculator = MeshNormalCalculator::new();
    let mut mesh_data = extractor.mesh_data;
    // println!("MESH DATA: {:#?}", mesh_data);
    mesh_normal_calculator.calculate_normals(&mut mesh_data);
    mesh_data
}
*/

struct TerrainSampler<'a> {
    size_xz: usize,
    size_y: usize,
    offset: Vec3u,
    values: &'a [f32],
    samples: usize,
}

impl<'a> isosurface::source::ScalarSource for TerrainSampler<'a> {
    fn sample_scalar(&self, p: isosurface::math::Vec3) -> isosurface::distance::Signed {
        let i = (p.x * (self.samples) as f32) as usize + self.offset.x;
        let j = (p.y * (self.samples) as f32) as usize + self.offset.y;
        let k = (p.z * (self.samples) as f32) as usize + self.offset.z;

        let index = i * self.size_xz * self.size_y + j * (self.size_xz) + k;
        if index > self.values.len() - 1 {
            return isosurface::distance::Signed(-1.0);

            //  println!("P: {:?}", p);
        }

        /*let v = if self.values[i * (self.size * self.size) + j * self.size + k] == 0 {
            -1.0
        } else {
            1.0
        };*/
        let v = self.values[index];
        //  println!("V: {:?}", v);
        isosurface::distance::Signed(v)
    }
}

struct Extractor {
    mesh_data: MeshData,
    indices: [u32; 3],
    positions: Vec<Vec3>,
    index: usize,
    scale: f32,
    offset: Vec3,
    world_offset: Vec3,
}
impl Extractor {
    pub fn new(scale: f32, offset: Vec3, world_offset: Vec3) -> Self {
        Self {
            positions: Vec::new(),
            mesh_data: MeshData::new(),
            indices: [0, 0, 0],
            index: 0,
            scale,
            offset,
            world_offset,
        }
    }
}

impl isosurface::extractor::Extractor for Extractor {
    fn extract_vertex(&mut self, vertex: isosurface::math::Vec3) {
        self.positions
            .push(Vec3::new(vertex.x, vertex.y, vertex.z) * self.scale + self.offset);
    }

    fn extract_index(&mut self, index: usize) {
        self.indices[self.index] = index as u32;
        self.index += 1;
        if self.index == 3 {
            // Make the mesh flat shaded
            // Normals could be calculated here as
            let offset = self.mesh_data.positions.len() as u32;

            let p0 = self.positions[self.indices[0] as usize];
            let p1 = self.positions[self.indices[1] as usize];
            let p2 = self.positions[self.indices[2] as usize];

            self.mesh_data.positions.push(p0);
            self.mesh_data.positions.push(p1);
            self.mesh_data.positions.push(p2);

            let normal = ((p1 - p0).cross(p2 - p1)).normalized();

            self.mesh_data.normals.push(normal);
            self.mesh_data.normals.push(normal);
            self.mesh_data.normals.push(normal);

            let mut color;

            let p0 = p0 + self.world_offset;
            if p0.y > 2800. {
                if normal.y > 0.5 {
                    color = Color::WHITE.to_linear_srgb()
                } else {
                    color = Color::from_srgb_hex(0xFFD700, 1.0).to_linear_srgb();
                }
            } else if p0.y > 2000. {
                if normal.y > 0.5 {
                    color = Color::OCHRE.to_linear_srgb()
                } else {
                    color = Color::PURPLE.with_lightness(0.8).to_linear_srgb()
                }
            } else if p0.y > 1000. {
                if normal.y > 0.5 {
                    color = Color::MINT.to_linear_srgb();
                } else {
                    color = Color::YELLOW
                        .with_chroma(0.2)
                        .with_lightness(0.8)
                        .to_linear_srgb()
                }
            } else {
                if normal.y > 0.5 {
                    color = Color::from_srgb_hex(0x51c43f, 1.0).to_linear_srgb();
                } else {
                    color = Color::interpolate(Color::OCHRE, Color::BROWN, 0.7)
                        .with_chroma(0.1)
                        .with_lightness(0.7)
                        .to_linear_srgb()
                }
            }

            self.mesh_data.colors.push(color);
            self.mesh_data.colors.push(color);
            self.mesh_data.colors.push(color);

            self.mesh_data
                .indices
                .push([offset, offset + 1, offset + 2]);
            self.index = 0;
        }
    }
}

struct MeshNormalCalculator {
    normal_use_count: Vec<i32>,
}

impl MeshNormalCalculator {
    pub fn new() -> Self {
        Self {
            normal_use_count: Vec::new(),
        }
    }
    pub fn calculate_normals(&mut self, mesh_data: &mut MeshData) {
        self.normal_use_count.clear();
        self.normal_use_count.resize(mesh_data.positions.len(), 0);

        mesh_data.normals.clear();
        mesh_data
            .normals
            .resize(mesh_data.positions.len(), Vec3::ZERO);
        for [p0, p1, p2] in mesh_data.indices.iter().cloned() {
            let dir0 = mesh_data.positions[p1 as usize] - mesh_data.positions[p0 as usize];
            let dir1 = mesh_data.positions[p2 as usize] - mesh_data.positions[p1 as usize];
            let normal = dir0.cross(dir1).normalized();
            self.normal_use_count[p0 as usize] += 1;
            self.normal_use_count[p1 as usize] += 1;
            self.normal_use_count[p2 as usize] += 1;

            mesh_data.normals[p0 as usize] += normal;
            mesh_data.normals[p1 as usize] += normal;
            mesh_data.normals[p2 as usize] += normal;
        }

        for (normal, &normal_use_count) in mesh_data
            .normals
            .iter_mut()
            .zip(self.normal_use_count.iter())
        {
            *normal = *normal / normal_use_count as f32;
        }
    }
}
