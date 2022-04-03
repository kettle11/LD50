use noise::NoiseFn;
use noise::Perlin;

use crate::*;

pub fn generate_chunk(offset: Vec3) -> MeshData {
    let scale = 200.;
    let noise = noise::Perlin::new();
    let terrain_sampler = TerrainSampler {
        noise,
        scale,
        radius_squared: (scale / 2.0) * (scale / 2.0),
        center: Vec3::fill(scale) / 2.0,
        offset,
    };
    let mut chunk = isosurface::MarchingCubes::new(64);
    let sampler = isosurface::sampler::Sampler::new(&terrain_sampler);

    let mut extractor = Extractor::new(scale, (scale / 64.0) * offset);

    chunk.extract(&sampler, &mut extractor);
    let mut mesh_normal_calculator = MeshNormalCalculator::new();
    let mut mesh_data = extractor.mesh_data;
    // println!("MESH DATA: {:#?}", mesh_data);
    mesh_normal_calculator.calculate_normals(&mut mesh_data);
    mesh_data
}

struct TerrainSampler {
    noise: Perlin,
    scale: f32,
    radius_squared: f32,
    center: Vec3,
    offset: Vec3,
}

impl isosurface::source::ScalarSource for TerrainSampler {
    fn sample_scalar(&self, p: isosurface::math::Vec3) -> isosurface::distance::Signed {
        let persistence = 0.5;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        let mut sample = 0.0;
        let p = Vec3::new(p.x, p.y, p.z) + self.offset;
        for _ in 0..6 {
            let p = p * 4.0 + Vec3::fill(2000.);
            let p = p * frequency;
            sample += self.noise.get([p.x as f64, p.y as f64, p.z as f64]) * amplitude;

            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        let p = p * self.scale - self.center;

        let distance_from_center = p.xz().length_squared();
        //  println!("DISTANCE FROM CENTER: {:?}", distance_from_center);

        let scale_factor = (distance_from_center / self.radius_squared).clamp(0.0, 1.0) as f64;
        //  println!("SCALE FACTOR: {:?}", scale_factor);
        let sample = (sample / max_value + 0.1) - scale_factor;

        isosurface::distance::Signed(sample as f32)
    }
}

struct Extractor {
    mesh_data: MeshData,
    indices: [u32; 3],
    index: usize,
    scale: f32,
    offset: Vec3,
}
impl Extractor {
    pub fn new(scale: f32, offset: Vec3) -> Self {
        Self {
            mesh_data: MeshData::new(),
            indices: [0, 0, 0],
            index: 0,
            scale,
            offset,
        }
    }
}

impl isosurface::extractor::Extractor for Extractor {
    fn extract_vertex(&mut self, vertex: isosurface::math::Vec3) {
        self.mesh_data
            .positions
            .push(Vec3::new(vertex.x, vertex.y, vertex.z) * self.scale + self.offset);
    }

    fn extract_index(&mut self, index: usize) {
        self.indices[self.index] = index as u32;
        self.index += 1;
        if self.index == 3 {
            self.mesh_data.indices.push(self.indices);
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
