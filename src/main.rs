mod framebuffer;
mod color;
mod cube;
mod camera;
mod material;
mod stats;

use framebuffer::Framebuffer;
use color::Color;
use cube::Cube;
use camera::OrbitCamera;
use material::{Material, MaterialType};
use stats::RenderStats;
use nalgebra_glm::{Vec3, normalize, dot};
use minifb::{Key, Window, WindowOptions};
use image::open;
use std::f32::consts::PI;

const WIDTH: usize = 500;
const HEIGHT: usize = 400;
const MAX_DEPTH: u32 = 5;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light { position, color, intensity }
    }
}

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Plane {
            point,
            normal: nalgebra_glm::normalize(&normal),
            material,
        }
    }
    
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        let denom = nalgebra_glm::dot(&self.normal, ray_direction);
        if denom.abs() < 1e-6 { return None; }
        let t = nalgebra_glm::dot(&(self.point - ray_origin), &self.normal) / denom;
        if t > 0.001 { Some(t) } else { None }
    }
    
    pub fn get_normal(&self, _point: &Vec3) -> Vec3 {
        self.normal
    }
}

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let img = open(path)?;
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let data = rgb_img.into_raw();
        Ok(Texture { width, height, data })
    }
    
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = (u.fract() + 1.0).fract().clamp(0.0, 1.0);
        let v = (v.fract() + 1.0).fract().clamp(0.0, 1.0);
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        let index = ((y * self.width + x) * 3) as usize;
        
        if index + 2 < self.data.len() {
            Color::new(self.data[index], self.data[index + 1], self.data[index + 2])
        } else {
            Color::new(255, 0, 255)
        }
    }
    
    pub fn create_grass_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 17 + y * 13) % 16) as f32 / 16.0;
                let noise2 = ((x * 7 + y * 11) % 8) as f32 / 8.0;
                let combined_noise = (noise1 + noise2 * 0.3).clamp(0.0, 1.0);
                let base_green = 160 + (combined_noise * 60.0) as u8;
                let r = (25.0 + combined_noise * 35.0) as u8;
                let b = (25.0 + combined_noise * 30.0) as u8;
                data.extend_from_slice(&[r, base_green, b]);
            }
        }
        Texture { width: 32, height: 32, data }
    }

    pub fn create_cactus_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let stripe = if (x / 4) % 2 == 0 { 18 } else { 28 };
                let g = 140 + stripe;
                let r = 40 + (stripe / 2);
                let b = 40 + (stripe / 3);
                data.extend_from_slice(&[r as u8, g as u8, b as u8]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_stone_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 23 + y * 19) % 32) as f32 / 32.0;
                let noise2 = ((x * 7 + y * 13) % 16) as f32 / 16.0;
                let combined_noise = (noise1 + noise2 * 0.4).clamp(0.0, 1.0);
                let base_gray = (70.0 + combined_noise * 40.0) as u8;
                let variation = (combined_noise * 15.0) as u8;
                data.extend_from_slice(&[ base_gray + variation, base_gray + (variation / 2), base_gray ]);
            }
        }
        Texture { width: 32, height: 32, data }
    }
    
    pub fn create_dirt_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 13 + y * 17) % 24) as f32 / 24.0;
                let noise2 = ((x * 29 + y * 7) % 16) as f32 / 16.0;
                let combined_noise = (noise1 + noise2 * 0.5).clamp(0.0, 1.0);
                let brown_r = (140.0 + combined_noise * 50.0) as u8;
                let brown_g = (85.0 + combined_noise * 35.0) as u8;
                let brown_b = (35.0 + combined_noise * 25.0) as u8;
                data.extend_from_slice(&[brown_r, brown_g, brown_b]);
            }
        }
        Texture { width: 32, height: 32, data }
    }
    
     
     // agua más intensa
     pub fn create_water_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                
                let w1 = ((x as f32 * 0.35).sin() + (y as f32 * 0.25).sin()) * 0.35;
                let w2 = ((x as f32 * 0.18 + y as f32 * 0.22).sin()) * 0.25;
                let w = (w1 + w2).clamp(-0.6, 0.6);

            
                let r = (15.0 + w * 10.0).round().clamp(0.0, 40.0) as u8;
                let g = (90.0 + w * 25.0).round().clamp(70.0, 140.0) as u8;
                let b = (205.0 + w * 40.0).round().clamp(160.0, 255.0) as u8;

                data.extend_from_slice(&[r, g, b]);
            }
        }
        Texture { width: size, height: size, data }
    }


    
    pub fn create_lava_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 31 + y * 17) % 32) as f32 / 32.0;
                let noise2 = ((x * 13 + y * 29) % 16) as f32 / 16.0;
                let intensity = (noise1 + noise2 * 0.6).clamp(0.0, 1.0);
                if intensity > 0.7 {
                    data.extend_from_slice(&[255, 255, (150.0 + intensity * 105.0) as u8]);
                } else if intensity > 0.4 {
                    data.extend_from_slice(&[255, (120.0 + intensity * 135.0) as u8, 30]);
                } else {
                    data.extend_from_slice(&[(180.0 + intensity * 75.0) as u8, 20, 0]);
                }
            }
        }
        Texture { width: 32, height: 32, data }
    }
    
    pub fn create_obsidian_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 43 + y * 23) % 16) as f32 / 16.0;
                let noise2 = ((x * 17 + y * 31) % 8) as f32 / 8.0;
                let combined_noise = (noise1 + noise2 * 0.2).clamp(0.0, 1.0);
                
                let base_intensity = 15.0 + combined_noise * 25.0;
                let purple_tint = if combined_noise > 0.8 { 20.0 } else { 5.0 };
                
                let r = (base_intensity + purple_tint * 0.6) as u8;
                let g = base_intensity as u8;
                let b = (base_intensity + purple_tint) as u8;
                
                data.extend_from_slice(&[r, g, b]);
            }
        }
        Texture { width: 32, height: 32, data }
    }

    // new for tree
    pub fn create_sand_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size*size*3) as usize);
        for y in 0..size {
            for x in 0..size {
                let n1 = ((x * 17 + y * 9) % 16) as f32 / 16.0;
                let n2 = ((x * 5 + y * 13) % 8) as f32 / 8.0;
                let t = (0.6 + 0.4*(n1*0.7 + n2*0.3)).clamp(0.0,1.0);
                let r = (210.0 + 40.0*t) as u8;
                let g = (190.0 + 35.0*t) as u8;
                let b = (140.0 + 25.0*t) as u8;
                data.extend_from_slice(&[r,g,b]);
            }
        }
        Texture { width: size, height: size, data }
    }

    pub fn create_wood_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size*size*3) as usize);
        for y in 0..size {
            for x in 0..size {
                let ring = (((x as f32 - 16.0).hypot(y as f32 - 16.0) * 0.4).sin()*0.5+0.5).clamp(0.0,1.0);
                let r = (110.0 + 60.0*ring) as u8;
                let g = (75.0 + 40.0*ring) as u8;
                let b = (45.0 + 25.0*ring) as u8;
                data.extend_from_slice(&[r,g,b]);
            }
        }
        Texture { width: size, height: size, data }
    }

    pub fn create_leaves_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size*size*3) as usize);
        for y in 0..size {
            for x in 0..size {
                let n = ((x*23 + y*31) % 32) as f32 / 32.0;
                let r = (30.0 + 40.0*n) as u8;
                let g = (120.0 + 100.0*n) as u8;
                let b = (30.0 + 35.0*n) as u8;
                data.extend_from_slice(&[r,g,b]);
            }
        }
        Texture { width: size, height: size, data }
    }

}

#[derive(Clone)]
pub struct Skybox {
    pub px: Texture,
    pub nx: Texture,
    pub py: Texture,
    pub ny: Texture,
    pub pz: Texture,
    pub nz: Texture,
}

impl Skybox {
    pub fn create_procedural_sky() -> Self {
        Skybox {
            px: Self::create_sky_texture_right(),
            nx: Self::create_sky_texture_left(), 
            py: Self::create_sky_texture_top(),
            ny: Self::create_sky_texture_bottom(),
            pz: Self::create_sky_texture_front(),
            nz: Self::create_sky_texture_back(),
        }
    }


    pub fn load_from_files() -> Result<Self, Box<dyn std::error::Error>> {
        match Self::try_load_from_files() {
            Ok(skybox) => {
                println!("Skybox loaded, all good!!");
                Ok(skybox)
            },
            Err(e) => {
                println!("Failed skybox: {}", e);
                println!("Procedural skybox ...");
                Ok(Self::create_procedural_sky())
            }
        }
    }

    fn make_vertical_gradient(size: usize, top: (u8,u8,u8), bottom: (u8,u8,u8)) -> Texture {
        let mut data = Vec::with_capacity(size * size * 3);
        for y in 0..size {
            let t = y as f32 / (size as f32 - 1.0);          // 0 arriba, 1 abajo
            let r = (top.0 as f32*(1.0 - t) + bottom.0 as f32*t) as u8;
            let g = (top.1 as f32*(1.0 - t) + bottom.1 as f32*t) as u8;
            let b = (top.2 as f32*(1.0 - t) + bottom.2 as f32*t) as u8;
            for _x in 0..size {
                data.extend_from_slice(&[r,g,b]);
            }
        }
        Texture { width: size as u32, height: size as u32, data }
    }
    
    fn try_load_from_files() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Skybox {
            px: Texture::load_from_file("px.png")?,
            nx: Texture::load_from_file("nx.png")?,
            py: Texture::load_from_file("py.png")?,
            ny: Texture::load_from_file("ny.png")?,
            pz: Texture::load_from_file("pz.png")?,
            nz: Texture::load_from_file("nz.png")?,
        })
    }
    
    fn create_sky_texture_top() -> Texture {
        
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (60, 130, 255),   // arriba (más oscuro)
            (170, 210, 255),  // abajo (más claro)
        )
    }
        
    fn create_sky_texture_bottom() -> Texture {
        
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (80, 150, 255),
            (180, 220, 255),
        )
    }

    
    fn create_sky_texture_front() -> Texture {
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (70, 140, 255),
            (185, 220, 255),
        )
    }
    
    fn create_sky_texture_back() -> Texture {
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (70, 140, 255),
            (185, 220, 255),
        )
    }
    
    fn create_sky_texture_left() -> Texture {
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (70, 140, 255),
            (185, 220, 255),
        )
    }

    
    fn create_sky_texture_right() -> Texture {
        let size = 256;
        Self::make_vertical_gradient(
            size,
            (70, 140, 255),
            (185, 220, 255),
        )
    }
    
    pub fn sample(&self, direction: &Vec3) -> Color {
         let dir = nalgebra_glm::normalize(direction);

       
        let adjusted_dir = dir;

        let abs_x = adjusted_dir.x.abs();
        let abs_y = adjusted_dir.y.abs();
        let abs_z = adjusted_dir.z.abs();
            
        let (texture, u, v) = if abs_x >= abs_y && abs_x >= abs_z {
            if adjusted_dir.x > 0.0 {
                let u = (-adjusted_dir.z / abs_x + 1.0) * 0.5;
                let v = (-adjusted_dir.y / abs_x + 1.0) * 0.5;
                (&self.px, u, v)
            } else {
                let u = (adjusted_dir.z / abs_x + 1.0) * 0.5;
                let v = (-adjusted_dir.y / abs_x + 1.0) * 0.5;
                (&self.nx, u, v)
            }
        } else if abs_y >= abs_x && abs_y >= abs_z {
            if adjusted_dir.y > 0.0 {
                let u = (adjusted_dir.x / abs_y + 1.0) * 0.5;
                let v = (adjusted_dir.z / abs_y + 1.0) * 0.5;
                (&self.py, u, v)
            } else {
                let u = (adjusted_dir.x / abs_y + 1.0) * 0.5;
                let v = (-adjusted_dir.z / abs_y + 1.0) * 0.5;
                (&self.ny, u, v)
            }
        } else {
            if adjusted_dir.z > 0.0 {
                let u = (adjusted_dir.x / abs_z + 1.0) * 0.5;
                let v = (-adjusted_dir.y / abs_z + 1.0) * 0.5;
                (&self.pz, u, v)
            } else {
                let u = (-adjusted_dir.x / abs_z + 1.0) * 0.5;
                let v = (-adjusted_dir.y / abs_z + 1.0) * 0.5;
                (&self.nz, u, v)
            }
        };
        
        texture.sample(u, v)
    }
}

pub struct OptimizedDiorama {
    pub cubes: Vec<Cube>,
    pub water_planes: Vec<Plane>,
    pub lava_planes: Vec<Plane>,
    pub bounding_box_min: Vec3,
    pub bounding_box_max: Vec3,
}

impl OptimizedDiorama {

    
   
    
    pub fn new(center: Vec3, cube_size: f32) -> Self {
        let mut cubes = Vec::new();
        let mut water_planes = Vec::new();
        let mut lava_planes = Vec::new();
        
        let grid_size = 18;
        let spacing = cube_size;
        let offset = (grid_size as f32 * spacing) / 2.0 - spacing / 2.0;
        
        let terrain_heights = Self::generate_terrain_heights(grid_size);
        
        let mut min_pos = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max_pos = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let height = terrain_heights[z][x];
                
                for y_level in 0..=height {
                    let pos = Vec3::new(
                        center.x + x as f32 * spacing - offset,
                        center.y + (y_level as f32) * spacing,
                        center.z + z as f32 * spacing - offset,
                    );
                    
                    min_pos = Vec3::new(min_pos.x.min(pos.x), min_pos.y.min(pos.y), min_pos.z.min(pos.z));
                    max_pos = Vec3::new(max_pos.x.max(pos.x), max_pos.y.max(pos.y), max_pos.z.max(pos.z));
                    
                    let material = Self::determine_material(x, z, y_level, height);
                    
                    if Self::should_place_cube(x, z, y_level, height, grid_size) {
                        cubes.push(Cube::new(pos, cube_size, material));
                    }
                }
            }
        }
        
        Self::add_water_areas(&mut water_planes, &terrain_heights, center, cube_size, spacing, offset);
        Self::add_lava_areas(&mut lava_planes, &terrain_heights, center, cube_size, spacing, offset);

        Self::place_tree(&mut cubes, center, cube_size, spacing, offset);
        //Self::place_forest_corner_details(&mut cubes, center, cube_size, spacing, offset);
        Self::place_crystal_details(&mut cubes, center, cube_size, spacing, offset);
        Self::place_overhang_roof(&mut cubes, center, cube_size, spacing, offset);

        // cactus
        Self::place_cactus(&mut cubes, center, cube_size, spacing, offset);

      

         


        
        OptimizedDiorama { 
            cubes, 
            water_planes, 
            lava_planes,
            bounding_box_min: min_pos - Vec3::new(2.0, 2.0, 2.0),
            bounding_box_max: max_pos + Vec3::new(2.0, 2.0, 2.0),
        }


    }

    // fn fores

    fn place_forest_corner_details(
        cubes: &mut Vec<Cube>, center: Vec3, cube_size: f32, spacing: f32, offset: f32
    ) {
        
        for &(bx, bz) in &[(13,13), (14,12), (16,13)] {
            for by in 0..2 {
                let pos = Vec3::new(
                    center.x + (bx as f32) * spacing - offset,
                    center.y + (5 + by) as f32 * spacing,    
                    center.z + (bz as f32) * spacing - offset,
                );
                cubes.push(Cube::new(pos, cube_size, Material::stone_layer()));
            }
        }

        
        for x in 15..=16 {
            for z in 16..=17 {
                for y in (4..=6).rev() { // para abajo
                    let pos = Vec3::new(
                        center.x + (x as f32) * spacing - offset,
                        center.y + (y as f32) * spacing,
                        center.z + (z as f32) * spacing - offset,
                    );
                    cubes.push(Cube::new(pos, cube_size, Material::leaves_block()));
                }
            }
        }
    }

   

        fn place_tree(cubes: &mut Vec<Cube>, center: Vec3, cube_size: f32,
                        spacing: f32, offset: f32) {
                // los cálculos del arbol
               let tx: i32 = 14;
                let tz: i32 = 13;
                let base_y: i32 = 5;  
                let trunk_h: i32 = 4;

                // tronco
                for i in 0..trunk_h {
                    let pos = Vec3::new(
                        center.x + (tx as f32) * spacing - offset,
                        center.y + ((base_y + i) as f32) * spacing,
                        center.z + (tz as f32) * spacing - offset,
                    );
                    
                    cubes.push(Cube::new(pos, cube_size, Material::wood_block()));
                    
                }

                // copa 3x3x3 con redondeo de Manhattan
                let top_y = base_y + trunk_h;
                for dz in -1i32..=1 {
                    for dx in -1i32..=1 {
                        for dy in 0i32..=2 {
                            let manhattan: i32 = dx.abs() + dy + dz.abs();
                            if manhattan <= 3 {
                                let pos = Vec3::new(
                                    center.x + ((tx + dx) as f32) * spacing - offset,
                                    center.y + ((top_y + dy) as f32) * spacing,
                                    center.z + ((tz + dz) as f32) * spacing - offset,
                                );
                                cubes.push(Cube::new(pos, cube_size, Material::leaves_block()));
                        }
                    }
                }
            }
        }

        fn in_oasis(x: usize, z: usize) -> bool {
            // oasis de agua
            let grid_back = 17;
            (x >= 8 && x <= 10) && (z >= grid_back - 3 && z <= grid_back - 1)
        }

        //cactus
        fn place_cactus(
            cubes: &mut Vec<Cube>, center: Vec3, cube_size: f32, spacing: f32, offset: f32) {
            
            let cx = 9usize;    
            let cz = 8usize;    

           
            let base_y = 4usize;
            for y in base_y..=base_y + 1 {
                let pos = Vec3::new(
                    center.x + (cx as f32) * spacing - offset,
                    center.y + (y as f32) * spacing,
                    center.z + (cz as f32) * spacing - offset,
                );
                cubes.push(Cube::new(pos, cube_size, Material::cactus_block()));
            }
        }


        // cave
        fn place_crystal_details(
            cubes: &mut Vec<Cube>, center: Vec3, cube_size: f32, spacing: f32, offset: f32) {
            

           
            for &(cx, cz) in &[(2,8), (3,11)] {
                for y in 3..=4 {
                    let pos = Vec3::new(
                        center.x + (cx as f32) * spacing - offset,
                        center.y + (y as f32) * spacing,
                        center.z + (cz as f32) * spacing - offset,
                    );
                    cubes.push(Cube::new(pos, cube_size, Material::crystal_block()));
                }
            }
        }

        const ROOF_X0: usize = 0;   
        const ROOF_Z0: usize = 0;  
        const ROOF_W:  usize = 6;   
        const ROOF_H:  usize = 7;   
        const ROOF_Y:  usize = 7;  

        fn in_roof_plate(x: usize, z: usize) -> bool {
            (Self::ROOF_X0..Self::ROOF_X0 + Self::ROOF_W).contains(&x) &&
            (Self::ROOF_Z0..Self::ROOF_Z0 + Self::ROOF_H).contains(&z)
        }

        


    
    
    const CORNER_X: usize = 17;  
    const CORNER_Z: usize = 12; 
    // terreno heights
    fn generate_terrain_heights(grid_size: usize) -> Vec<Vec<usize>> {
        
        let mut heights = vec![vec![1; grid_size]; grid_size];

        for z in 0..grid_size {
            for x in 0..grid_size {
                let h = if x < 6 {
                    // cave  
                    let mut h = if x == 0 || z == 0 { 6 } 
                    else if x == 1 || z == 1 { 5 }        
                    else if x == 2 || z == 2 { 4 }        
                    else { 3 };                            

                   

                    h

                } else if x < 12 {
                    // oasis 
                    if Self::in_oasis(x, z) { 2 } else { 3 }
                } else {
                    // forest 

                        if x == 12 || x == 13 {
                            3
                        } else {
                            //para orilla 
                            let mut base = 4 + ((x + 2 * z) % 2) as usize; // 4–5

                        
                            let dx = (x as isize - Self::CORNER_X as isize).abs() as f32;
                            let dz = (z as isize - Self::CORNER_Z as isize).abs() as f32;
                            let dist = (dx * dx + dz * dz).sqrt();

                            let bump = if dist < 2.5 { 2 } else if dist < 5.5 { 1 } else { 0 };
                            base + bump
                        }
                   
                };
                heights[z][x] = h;
            }
        } 

        heights
    }

    
    // para la cave bioma 
    fn in_lava_pond_a(x: usize, z: usize) -> bool {  
        x >= 2 && x <= 4 && z >= 2 && z <= 4
    }
    fn in_lava_pond_b(x: usize, z: usize) -> bool {  
        x >= 1 && x <= 3 && z >= 4 && z <= 6.min(5)  
    }
    fn in_lava_pond_c(x: usize, z: usize, grid_size: usize) -> bool {
        //del lado opuesto
        x >= 2 && x <= 4 && z >= grid_size - 4 && z <= grid_size - 2
    }

    fn in_any_lava_pond(x: usize, z: usize, grid_size: usize) -> bool {
        Self::in_lava_pond_a(x, z)
            || Self::in_lava_pond_b(x, z)
            || Self::in_lava_pond_c(x, z, grid_size)
    }
    

    fn place_overhang_roof(
        cubes: &mut Vec<Cube>, center: Vec3, cube_size: f32, spacing: f32, offset: f32
    ) {
        for x in Self::ROOF_X0..Self::ROOF_X0 + Self::ROOF_W {
            for z in Self::ROOF_Z0..Self::ROOF_Z0 + Self::ROOF_H {
                let pos = Vec3::new(
                    center.x + (x as f32) * spacing - offset,
                    center.y + (Self::ROOF_Y as f32) * spacing,
                    center.z + (z as f32) * spacing - offset,
                );
                cubes.push(Cube::new(pos, cube_size, Material::stone_layer()));
            }
        }
    }


    
        
    // determinar el material , AGUa etc
    fn determine_material(x: usize, z: usize, y_level: usize, max_height: usize) -> Material {
        let lava_zone = x < 6;
        let sand_zone = x >= 6 && x < 12;
        let grass_zone = x >= 12;
        let forest_zone = x >= 12;

        if lava_zone {
            
            if y_level == 1 {
                return Material::lava_surface();
            }

          // para la lava 
            if Self::in_any_lava_pond(x, z, 18) && y_level == max_height {
                return Material::lava_surface();
            }

            // obsidiana

            if (x <= 2 || z <= 2) && ((x + 2*z + y_level) % 5 == 0 || (3*x + z) % 7 == 0) {
                return Material::obsidian_block();                         
            }

            return Material::stone_layer();
        }

        if sand_zone {
            //  oasis superficie
            if Self::in_oasis(x, z) && y_level == 1 || y_level == 2 {
                return Material::water_surface();
            }
            // la sand
            if y_level == max_height { return Material::sand_top(); }
            return Material::stone_layer();
        }

        if grass_zone {
            // grama
            if y_level == max_height { return Material::grass_top(); }
            if y_level >= max_height - 1 { return Material::dirt_layer(); }
            return Material::stone_layer();
        }

        if forest_zone {
            
            if y_level == max_height { return Material::grass_top(); }
            if y_level >= max_height - 1 { return Material::dirt_layer(); }
            return Material::stone_layer();
        }

        
        Material::stone_layer()
    }

    
    fn should_place_cube(x: usize, z: usize, y_level: usize, max_height: usize, grid_size: usize) -> bool {
       

        if x < 6 {
           
           if y_level == 1 { return true; }
           // huecoo
            


            return true; 
        }


        // Para el oasis 
        if x >= 6 && x < 12 && Self::in_oasis(x, z) {
            if y_level == 1  || y_level == 2  { return true; } 
            if y_level >= 2 { return false; }
        }

        // para el forest - lo quité porque no me gustó 
        //if x >= 12 && Self::in_grotto_cut(x, z) && (y_level == 6 || y_level == 5) {
          //  return false;
        //}

        true
    }
    
    fn add_water_areas(_water_planes: &mut Vec<Plane>, _heights: &Vec<Vec<usize>>, _center: Vec3, _cube_size: f32, _spacing: f32, _offset: f32) {
    }
    
    fn add_lava_areas(_lava_planes: &mut Vec<Plane>, _heights: &Vec<Vec<usize>>, _center: Vec3, _cube_size: f32, _spacing: f32, _offset: f32) {
    }


    
    

    
    pub fn ray_intersect_fast(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<(usize, f32, u8)> {
        if !self.ray_intersects_bbox(ray_origin, ray_direction) {
            return None;
        }
        
        let mut closest_distance = f32::INFINITY;
        let mut closest_index = None;
        
        for (i, cube) in self.cubes.iter().enumerate() {
            if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                    if distance < 0.1 { break; }
                }
            }
        }
        
        closest_index.map(|idx| (idx, closest_distance, 1))
    }
    
    fn ray_intersects_bbox(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> bool {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            if ray_direction[i].abs() < 1e-6 {
                if ray_origin[i] < self.bounding_box_min[i] || ray_origin[i] > self.bounding_box_max[i] {
                    return false;
                }
            } else {
                let t1 = (self.bounding_box_min[i] - ray_origin[i]) / ray_direction[i];
                let t2 = (self.bounding_box_max[i] - ray_origin[i]) / ray_direction[i];
                
                let t_near = t1.min(t2);
                let t_far = t1.max(t2);
                
                t_min = t_min.max(t_near);
                t_max = t_max.min(t_far);
                
                if t_min > t_max {
                    return false;
                }
            }
        }
        
        t_max > 0.0
    }
    
    pub fn ray_intersect_shadow_fast(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        for (i, cube) in self.cubes.iter().enumerate() {
            if i % 2 == 0 {
                if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                    if distance > 0.001 && distance < max_distance {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn reflect(dir: &Vec3, normal: &Vec3) -> Vec3 { *dir - *normal * 2.0 * dot(dir, normal) }

fn refract(incident: &Vec3, normal: &Vec3, eta: f32) -> Option<Vec3> {
    let mut n = *normal;
    let mut cosi = dot(incident, &n).clamp(-1.0, 1.0);
    let mut etai = 1.0;
    let mut etat = eta;
    if cosi < 0.0 { cosi = -cosi; } else { std::mem::swap(&mut etai, &mut etat); n = -n; }
    let eta_ratio = etai / etat;
    let k = 1.0 - eta_ratio * eta_ratio * (1.0 - cosi * cosi);
    if k < 0.0 { None } else { Some(*incident * eta_ratio + n * (eta_ratio * cosi - k.sqrt())) }
}

fn fresnel(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    let mut cosi = dot(incident, normal).clamp(-1.0, 1.0);
    let etai = 1.0;
    let etat = ior;
    if cosi > 0.0 {
        let r0 = ((etat - etai) / (etat + etai)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosi).powi(5)
    } else {
        let cosi_abs = -cosi;
        let r0 = ((etat - etai) / (etat + etai)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosi_abs).powi(5)
    }
}

fn sample_sky(skybox: &Option<Skybox>, dir: &Vec3) -> Color {
    if let Some(sb) = skybox {
        let closer_dir = Vec3::new(dir.x * 0.3, dir.y * 0.7, dir.z * 0.3);
        sb.sample(&closer_dir)
    } else {
        if dir.y > 0.1 {
            let t = ((dir.y - 0.1) / 0.9).clamp(0.0, 1.0);
            Color::new((100.0 + t * 80.0) as u8, (180.0 + t * 50.0) as u8, 255)
        } else {
            Color::new(120, 160, 200)
        }
    }
}

fn cast_ray_optimized_recursive(ray_origin: &Vec3, ray_direction: &Vec3, diorama: &OptimizedDiorama, floor: &Plane, 
                                lights: &[Light], grass_texture: &Texture, dirt_texture: &Texture, stone_texture: &Texture, 
                                water_texture: &Texture, lava_texture: &Texture, obsidian_texture: &Texture,   sand_texture: &Texture, leaves_texture: &Texture, wood_texture: &Texture,   crystal_texture: &Texture,  cactus_texture: &Texture,   
                                skybox: &Option<Skybox>, stats: &mut RenderStats, depth: u32) -> Color {
    if depth == 0 {
        return sample_sky(skybox, ray_direction);
    }

    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0;
    let mut hit_cube: Option<&Cube> = None;

    stats.rays_cast += 1;

    if let Some((object_index, distance, object_type)) = diorama.ray_intersect_fast(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance && object_type == 1 {
            closest_distance = distance;
            let cube = &diorama.cubes[object_index];
            hit_material = Some(cube.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = cube.get_normal(&hit_point);
            hit_cube = Some(cube);
            hit_object = 1;
            stats.hits += 1;
        }
    }

    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            hit_material = Some(floor.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = floor.get_normal(&hit_point);
            closest_distance = distance;
            hit_object = 5;
            stats.hits += 1;
        }
    }

    if hit_object == 0 {
        stats.misses += 1;
        return sample_sky(skybox, ray_direction);
    }

    if let Some(material) = hit_material {
        let base_color = if hit_object == 1 && material.has_texture && hit_cube.is_some() {
            let cube = hit_cube.unwrap();
            let (u, v) = cube.get_uv_coordinates(&hit_point);
            match material.material_type {
                MaterialType::Grass    => grass_texture.sample(u, v),
                MaterialType::Dirt     => dirt_texture.sample(u, v),
                MaterialType::Stone    => stone_texture.sample(u, v),
                MaterialType::Water    => water_texture.sample(u, v),
                MaterialType::Lava     => lava_texture.sample(u, v),
                MaterialType::Obsidian => obsidian_texture.sample(u, v),
                MaterialType::Sand     => sand_texture.sample(u, v),
                MaterialType::Wood     => wood_texture.sample(u, v),
                MaterialType::Leaves   => leaves_texture.sample(u, v),
                MaterialType::Crystal => crystal_texture.sample(u, v),
                MaterialType::Glass | MaterialType::Metal => material.diffuse,
                MaterialType::Cactus   => cactus_texture.sample(u, v),


                _ => material.diffuse,
            }
        } else {
            material.diffuse
        };

        let ambient_strength = match material.material_type {
            MaterialType::Grass => 0.5,
            MaterialType::Stone => 0.25,
            MaterialType::Dirt => 0.35,
            MaterialType::Water => 0.15,
            MaterialType::Lava => 0.8,
            // ambient_strength
            MaterialType::Sand => 0.45,
            MaterialType::Leaves => 0.55,
            MaterialType::Obsidian => 0.2,
            _ => 0.3,
        };

        let mut total_r = base_color.r as f32 * ambient_strength;
        let mut total_g = base_color.g as f32 * ambient_strength;
        let mut total_b = base_color.b as f32 * ambient_strength;

        if material.is_emissive() {
            let ec = material.emission_color();
            let ei = material.emission_intensity();
            total_r += ec.r as f32 * ei * 2.0;
            total_g += ec.g as f32 * ei * 2.0;
            total_b += ec.b as f32 * ei * 2.0;
        }

        for (i, light) in lights.iter().enumerate() {
            let light_dir = normalize(&(light.position - hit_point));
            let light_distance = nalgebra_glm::distance(&light.position, &hit_point);

            let mut in_shadow = false;
            if i == 0 && material.material_type != MaterialType::Water {
                let shadow_origin = hit_point + hit_normal * 0.001;
                in_shadow = diorama.ray_intersect_shadow_fast(&shadow_origin, &light_dir, light_distance);
            }

            if !in_shadow {
                let diff = nalgebra_glm::dot(&hit_normal, &light_dir).max(0.0);
                let attenuation = 1.0 / (1.0 + 0.015 * light_distance + 0.0008 * light_distance * light_distance);

                let surface_multiplier = match material.material_type {
                    MaterialType::Grass => 1.4,
                    MaterialType::Stone => 0.8,
                    MaterialType::Dirt => 1.0,
                    MaterialType::Water => 2.0,
                    MaterialType::Lava => 0.3,
                     
                    MaterialType::Sand => 1.2,
                    MaterialType::Leaves => 1.6,
                    MaterialType::Obsidian => 1.1,
                    _ => 1.0,
                };

                let light_contribution = diff * light.intensity * attenuation * surface_multiplier;

                total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
                total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
                total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
            }
        }

        let mut final_color = Color::new(
            total_r.min(255.0) as u8,
            total_g.min(255.0) as u8,
            total_b.min(255.0) as u8,
        );

        let mut reflect_color = Color::black();
        if material.is_reflective() {
            let refl_dir = reflect(ray_direction, &hit_normal);
            let refl_origin = hit_point + hit_normal * 0.001;
            
            reflect_color = cast_ray_optimized_recursive(
                &refl_origin, &refl_dir, diorama, floor, lights,
                grass_texture, dirt_texture, stone_texture,
                water_texture, lava_texture, obsidian_texture,
                sand_texture,  leaves_texture,     wood_texture, crystal_texture, cactus_texture,
        
                skybox, stats, depth - 1
            );

        }

        let mut refract_color = Color::black();
        if material.is_transparent() {
            if let Some(refr_dir) = refract(ray_direction, &hit_normal, material.refractive_index) {
                let refr_origin = hit_point - hit_normal * 0.001;
               
               refract_color = cast_ray_optimized_recursive(
                    &refr_origin, &refr_dir, diorama, floor, lights,
                    grass_texture, dirt_texture, stone_texture,
                    water_texture, lava_texture, obsidian_texture,
                    sand_texture, leaves_texture,    wood_texture, crystal_texture,cactus_texture,
                    skybox, stats, depth - 1
                );
            }
        }

        if material.is_transparent() || material.is_reflective() {
            let kr = fresnel(ray_direction, &hit_normal, material.refractive_index).clamp(0.0, 1.0);
            if material.is_transparent() {
                let t = material.albedo[1];
                let reflected_part = reflect_color.to_vec3() * kr;
                let refracted_part = refract_color.to_vec3() * (1.0 - kr) * t;
                let base_part = final_color.to_vec3() * (1.0 - t);
                let mixed = base_part + reflected_part + refracted_part;
                return Color::from_vec3(mixed).clamp();
            } else {
                let mixed = final_color.to_vec3() * (1.0 - kr) + reflect_color.to_vec3() * kr;
                return Color::from_vec3(mixed).clamp();
            }
        }

        final_color.clamp()
    } else {
        sample_sky(skybox, ray_direction)
    }
}

trait ColorVec3 {
    fn to_vec3(&self) -> Vec3;
    fn from_vec3(v: Vec3) -> Self;
    fn clamp(self) -> Self;
    fn black() -> Self;
}

impl ColorVec3 for Color {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.r as f32, self.g as f32, self.b as f32)
    }
    fn from_vec3(v: Vec3) -> Self {
        Color::new(v.x as u8, v.y as u8, v.z as u8)
    }
    fn clamp(self) -> Self {
        Color::new(self.r.min(255), self.g.min(255), self.b.min(255))
    }
    fn black() -> Self {
        Color::new(0, 0, 0)
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let grass_texture = match Texture::load_from_file("grass.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_grass_texture()
    };
    let dirt_texture = match Texture::load_from_file("dirt.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_dirt_texture()
    };
    let stone_texture = match Texture::load_from_file("stone.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_stone_texture()
    };
    let water_texture = match Texture::load_from_file("water.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_water_texture()
    };
    let lava_texture = match Texture::load_from_file("lava.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_lava_texture()
    };

    let obsidian_texture = match Texture::load_from_file("obsidian.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_obsidian_texture()
    };

    let leaves_texture = match Texture::load_from_file("leaves.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_leaves_texture(), 
    };

    
    let wood_texture = match Texture::load_from_file("wood.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_wood_texture(), 
    };

    let cactus_texture = match Texture::load_from_file("cactus.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_cactus_texture(),
    };

    let crystal_texture = match Texture::load_from_file("crystal.png") {
        Ok(tex) => tex,
        Err(_) => {
            // fallback muy simple si no hay PNG
            let mut t = Texture { width: 32, height: 32, data: vec![] };
            for _ in 0..(32*32) { t.data.extend_from_slice(&[170, 210, 255]); }
            t
        }
    };

    let sand_texture   = Texture::create_sand_texture();
 
    

    let skybox = Some(Skybox::create_procedural_sky());

    let mut camera = OrbitCamera::new(Vec3::new(0.0, 2.0, 0.0), 10.0);
    camera.orbit(0.8, 0.4);

    let diorama = OptimizedDiorama::new(Vec3::new(0.0, 0.0, 0.0), 0.8);
    let floor = Plane::new(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Material::stone_wall());

    let lights = vec![
        Light::new(Vec3::new(-4.0, 8.0, -2.0), Color::new(255, 220, 180), 1.1),
        Light::new(Vec3::new(6.0, 6.0, 3.0), Color::new(180, 200, 255), 0.7),
    ];

    let mut window = Window::new("Belén Diorama", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(30);

    let mut stats = RenderStats::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit_speed = if window.is_key_down(Key::LeftShift) { 0.1 } else { 0.05 };
        let zoom_speed = if window.is_key_down(Key::LeftShift) { 1.2 } else { 0.6 };
        if window.is_key_down(Key::Left) { camera.orbit(-orbit_speed, 0.0); }
        if window.is_key_down(Key::Right) { camera.orbit(orbit_speed, 0.0); }
        if window.is_key_down(Key::Up) { camera.orbit(0.0, orbit_speed); }
        if window.is_key_down(Key::Down) { camera.orbit(0.0, -orbit_speed); }
        if window.is_key_down(Key::W) { camera.zoom(-zoom_speed); }
        if window.is_key_down(Key::S) { camera.zoom(zoom_speed); }
        if window.is_key_down(Key::Space) {
            camera = OrbitCamera::new(Vec3::new(0.0, 2.0, 0.0), 10.0);
            camera.orbit(0.8, 0.4);
        }

        stats.reset();
       render_optimized_recursive(
            &mut framebuffer, &diorama, &floor, &lights, &camera,
            &grass_texture, &dirt_texture, &stone_texture, &water_texture,
            &lava_texture, &obsidian_texture,
            &sand_texture, &wood_texture, &leaves_texture,  &crystal_texture, &cactus_texture,
            &skybox, &mut stats
        );

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn render_optimized_recursive(
        framebuffer: &mut Framebuffer, diorama: &OptimizedDiorama, floor: &Plane,
        lights: &[Light], camera: &OrbitCamera,
        grass_texture: &Texture, dirt_texture: &Texture, stone_texture: &Texture,
        water_texture: &Texture, lava_texture: &Texture, obsidian_texture: &Texture,
        sand_texture: &Texture, wood_texture: &Texture, leaves_texture: &Texture,  
        crystal_texture: &Texture, cactus_texture: &Texture,
        skybox: &Option<Skybox>, stats: &mut RenderStats
    ) {
    
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    framebuffer.clear();
    
    let skip = 1;
    for y in (0..framebuffer.height).step_by(skip) {
        for x in (0..framebuffer.width).step_by(skip) {
            let mut screen_x = (2.0 * x as f32) / width - 1.0;
            let mut screen_y = -(2.0 * y as f32) / height + 1.0;
            screen_x *= aspect_ratio;
            
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray_optimized_recursive(
                   &camera.eye, &ray_direction, diorama, floor, lights,
                grass_texture, dirt_texture, stone_texture,
                water_texture, lava_texture, obsidian_texture,
                sand_texture, leaves_texture, wood_texture,   crystal_texture, cactus_texture,
                skybox, stats, MAX_DEPTH
                );
            
            framebuffer.set_current_color(pixel_color);
            for dy in 0..skip {
                for dx in 0..skip {
                    if x + dx < framebuffer.width && y + dy < framebuffer.height {
                        framebuffer.point(x + dx, y + dy);
                    }
                }
            }
        }
    }
}