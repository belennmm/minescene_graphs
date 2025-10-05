use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2],
    pub refractive_index: f32,
    pub has_texture: bool,
    pub material_type: MaterialType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialType {
    Grass,
    Dirt,
    Stone,
    Water,
    Lava,
    Wood,
    Glass,
    Metal,
    Obsidian,  
    Sand, 
    Leaves,
    Crystal,  
    Cactus,  
}

impl Material {
    pub fn new(diffuse: Color) -> Self {
        Material {
            diffuse,
            specular: 0.0,
            albedo: [1.0, 0.0],
            refractive_index: 1.0,
            has_texture: false,
            material_type: MaterialType::Stone,
        }
    }
    
    pub fn with_texture(diffuse: Color, material_type: MaterialType) -> Self {
        Material {
            diffuse,
            specular: 0.0,
            albedo: [1.0, 0.0],
            refractive_index: 1.0,
            has_texture: true,
            material_type,
        }
    }
    
    pub fn grass_top() -> Self {
        Material {
            diffuse: Color::new(50, 180, 50),
            specular: 8.0,
            albedo: [0.85, 0.15],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Grass,
        }
    }

    pub fn dirt_layer() -> Self {
        Material {
            diffuse: Color::new(160, 100, 50),
            specular: 3.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Dirt,
        }
    }

    pub fn stone_layer() -> Self {
        Material {
            diffuse: Color::new(90, 90, 95),
            specular: 15.0,
            albedo: [0.7, 0.3],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Stone,
        }
    }
    
    pub fn water_surface() -> Self {
        Material {
            //  blue más bueno
            diffuse: Color::new(30, 110, 220),
            specular: 100.0,       
            
            albedo: [0.6, 0.4],    // 60% de  color, 40%  de transparencia 
            refractive_index: 1.33,
            has_texture: true,
            material_type: MaterialType::Water,
        }
    }
    pub fn lava_surface() -> Self {
        Material {
            diffuse: Color::new(255, 80, 0), // Naranja más intenso
            specular: 15.0,
            albedo: [0.8, 0.2], 
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Lava,
        }
    }
    
    // NUEVO: Material de obsidiana
    pub fn obsidian_block() -> Self {
        Material {
            diffuse: Color::new(20, 18, 30), // Negro-púrpura oscuro
            specular: 50.0, // Bastante reflectante
            albedo: [0.8, 0.2], 
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Obsidian,
        }
    }

    pub fn stone_wall() -> Self {
        Material {
            diffuse: Color::new(105, 105, 105),
            specular: 5.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Stone,
        }
    }
    
    pub fn wood_planks() -> Self {
        Material {
            diffuse: Color::new(139, 90, 43),
            specular: 10.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Wood,
        }
    }


    // 
     pub fn leaves_block() -> Self {
        Self {
            diffuse: Color::new(60, 160, 70),
            albedo: [0.9, 0.1],
            specular: 10.0,
            refractive_index: 1.0,
            material_type: MaterialType::Leaves,
            has_texture: true,
        }
    }

    pub fn wood_block() -> Self {
        Self {
            diffuse: Color::new(140, 100, 60),
             albedo: [0.9, 0.1],
            specular: 10.0,
            refractive_index: 1.0,
            material_type: MaterialType::Wood,
            has_texture: true,
        }
    }
    
    //
     pub fn sand_top() -> Self {
        Self {
            diffuse: Color::new(235, 220, 170),
            albedo: [0.9, 0.1],
            specular: 10.0,
            refractive_index: 1.0,
            material_type: MaterialType::Sand,
            has_texture: true,
        }
    }

    pub fn crystal_block() -> Self {
        Material {
            diffuse: Color::new(180, 220, 255), // azulito claro
            specular: 110.0,               
            albedo: [0.2, 0.8],                 
            refractive_index: 1.45,            
            has_texture: true,
            material_type: MaterialType::Crystal,
        }
    }


    pub fn clear_glass() -> Self {
        Material {
            diffuse: Color::new(255, 255, 255),
            specular: 125.0,
            albedo: [0.1, 0.9],
            refractive_index: 1.5,
            has_texture: false,
            material_type: MaterialType::Glass,
        }
    }

    pub fn cactus_block() -> Self {
        Self {
            diffuse: Color::new(80, 170, 80),
            specular: 5.0,
            albedo: [0.95, 0.05],
            refractive_index: 1.0,
            has_texture: true,
            material_type: MaterialType::Cactus,
        }
    }


    
    pub fn metal_surface() -> Self {
        Material {
            diffuse: Color::new(192, 192, 192),
            specular: 100.0,
            albedo: [0.4, 0.6],
            refractive_index: 1.0,
            has_texture: false,
            material_type: MaterialType::Metal,
        }
    }
    
    pub fn is_emissive(&self) -> bool {
        matches!(self.material_type, MaterialType::Lava)
    }
    
    pub fn is_transparent(&self) -> bool {
        matches!(self.material_type, MaterialType::Glass | MaterialType::Water)
    }
    
    pub fn is_reflective(&self) -> bool {
        self.specular > 50.0 || matches!(self.material_type, MaterialType::Water | MaterialType::Metal | MaterialType::Obsidian)
    }
    
    pub fn emission_intensity(&self) -> f32 {
        match self.material_type {
            MaterialType::Lava => 0.4,
            _ => 0.0,
        }
    }
    
    pub fn emission_color(&self) -> Color {
        match self.material_type {
            MaterialType::Lava => Color::new(255, 150, 50),
            _ => Color::black(),
        }
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.material_type == other.material_type && self.diffuse == other.diffuse
    }
}