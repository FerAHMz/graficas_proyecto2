use crate::math_utils::{Vec3, Color, Ray, reflect, refract, fresnel, random_in_unit_sphere};
use image::{DynamicImage, RgbaImage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub albedo: Color,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub roughness: f32,
    pub texture_id: Option<String>,
}

impl Material {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            albedo: Color::new(0.8, 0.8, 0.8),
            specular: 0.1,
            transparency: 0.0,
            reflectivity: 0.1,
            refractive_index: 1.0,
            roughness: 0.5,
            texture_id: None,
        }
    }
    
    pub fn with_texture(mut self, texture_id: &str) -> Self {
        self.texture_id = Some(texture_id.to_string());
        self
    }
    
    pub fn with_albedo(mut self, r: f32, g: f32, b: f32) -> Self {
        self.albedo = Color::new(r, g, b);
        self
    }
    
    pub fn with_properties(mut self, specular: f32, transparency: f32, reflectivity: f32, refractive_index: f32) -> Self {
        self.specular = specular;
        self.transparency = transparency;
        self.reflectivity = reflectivity;
        self.refractive_index = refractive_index;
        self
    }
}

pub struct TextureManager {
    textures: HashMap<String, RgbaImage>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
    
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        let rgba_img = img.to_rgba8();
        self.textures.insert(id.to_string(), rgba_img);
        Ok(())
    }
    
    pub fn sample_texture(&self, texture_id: &str, u: f32, v: f32) -> Color {
        if let Some(texture) = self.textures.get(texture_id) {
            let width = texture.width() as f32;
            let height = texture.height() as f32;
            
            let x = ((u.fract() * width) as u32).min(texture.width() - 1);
            let y = ((v.fract() * height) as u32).min(texture.height() - 1);
            
            let pixel = texture.get_pixel(x, y);
            Color::new(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            )
        } else {
            Color::new(1.0, 0.0, 1.0) // Magenta for missing texture
        }
    }
}

pub fn create_materials() -> Vec<Material> {
    vec![
        // Grass block
        Material::new("grass")
            .with_texture("grass_side")
            .with_albedo(0.4, 0.8, 0.2)
            .with_properties(0.1, 0.0, 0.05, 1.0),
            
        // Glass
        Material::new("glass")
            .with_texture("glass")
            .with_albedo(0.9, 0.9, 1.0)
            .with_properties(0.9, 0.9, 0.1, 1.52),
            
        // Iron
        Material::new("iron")
            .with_texture("iron_block")
            .with_albedo(0.7, 0.7, 0.8)
            .with_properties(0.8, 0.0, 0.9, 1.0),
            
        // Diamond
        Material::new("diamond")
            .with_texture("diamond_block")
            .with_albedo(0.8, 0.9, 1.0)
            .with_properties(0.95, 0.3, 0.8, 2.42),
            
        // Water
        Material::new("water")
            .with_texture("water_still")
            .with_albedo(0.2, 0.4, 0.8)
            .with_properties(0.7, 0.8, 0.3, 1.33),
    ]
}

#[derive(Debug, Clone)]
pub struct ScatterResult {
    pub scattered_ray: Ray,
    pub attenuation: Color,
    pub pdf: f32,
}

impl Material {
    pub fn scatter(&self, ray: &Ray, hit_point: Vec3, normal: Vec3, texture_manager: &TextureManager, u: f32, v: f32) -> Option<ScatterResult> {
        let mut base_color = self.albedo;
        
        // Apply texture if available
        if let Some(texture_id) = &self.texture_id {
            let texture_color = texture_manager.sample_texture(texture_id, u, v);
            base_color = base_color.component_mul(&texture_color);
        }
        
        let mut attenuation = base_color;
        let incident = ray.direction;
        let cos_i = -incident.dot(&normal);
        
        // Handle reflection and refraction
        if self.transparency > 0.0 && self.refractive_index != 1.0 {
            let eta = if cos_i > 0.0 { 1.0 / self.refractive_index } else { self.refractive_index };
            let fresnel_factor = fresnel(cos_i.abs(), eta);
            
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            if rng.r#gen::<f32>() < fresnel_factor * (1.0 - self.transparency) + self.reflectivity {
                // Reflection
                let reflected = reflect(incident, normal);
                let scattered_direction = if self.roughness > 0.0 {
                    (reflected + self.roughness * random_in_unit_sphere()).normalize()
                } else {
                    reflected
                };
                
                Some(ScatterResult {
                    scattered_ray: Ray::new(hit_point.into(), scattered_direction),
                    attenuation,
                    pdf: 1.0,
                })
            } else if let Some(refracted) = refract(incident, normal, eta) {
                // Refraction
                attenuation *= self.transparency;
                Some(ScatterResult {
                    scattered_ray: Ray::new(hit_point.into(), refracted),
                    attenuation,
                    pdf: 1.0,
                })
            } else {
                // Total internal reflection
                let reflected = reflect(incident, normal);
                Some(ScatterResult {
                    scattered_ray: Ray::new(hit_point.into(), reflected),
                    attenuation,
                    pdf: 1.0,
                })
            }
        } else if self.reflectivity > 0.0 {
            // Pure reflection
            let reflected = reflect(incident, normal);
            let scattered_direction = if self.roughness > 0.0 {
                (reflected + self.roughness * random_in_unit_sphere()).normalize()
            } else {
                reflected
            };
            
            attenuation *= self.reflectivity;
            Some(ScatterResult {
                scattered_ray: Ray::new(hit_point.into(), scattered_direction),
                attenuation,
                pdf: 1.0,
            })
        } else {
            // Diffuse scattering
            let scattered_direction = (normal + random_in_unit_sphere()).normalize();
            Some(ScatterResult {
                scattered_ray: Ray::new(hit_point.into(), scattered_direction),
                attenuation,
                pdf: 1.0,
            })
        }
    }
}
