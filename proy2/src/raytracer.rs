use crate::math_utils::{Vec3, Color, Ray};
use crate::cube::{Scene, HitRecord};
use crate::materials::{Material, TextureManager};
use crate::skybox::Skybox;
use crate::camera::Camera;
use rand::Rng;

pub struct Raytracer {
    pub scene: Scene,
    pub materials: Vec<Material>,
    pub texture_manager: TextureManager,
    pub skybox: Skybox,
    pub max_depth: u32,
    pub samples_per_pixel: u32,
}

impl Raytracer {
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),
            materials: Vec::new(),
            texture_manager: TextureManager::new(),
            skybox: Skybox::new(),
            max_depth: 10,
            samples_per_pixel: 4,
        }
    }
    
    pub fn add_material(&mut self, material: Material) -> usize {
        self.materials.push(material);
        self.materials.len() - 1
    }
    
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.texture_manager.load_texture(id, path)
    }
    
    fn ray_color(&self, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::zeros();
        }
        
        if let Some(hit) = self.scene.hit(ray, 0.001, f32::INFINITY) {
            if hit.material_index < self.materials.len() {
                let material = &self.materials[hit.material_index];
                
                if let Some(scatter_result) = material.scatter(
                    ray, 
                    hit.point, 
                    hit.normal, 
                    &self.texture_manager, 
                    hit.u, 
                    hit.v
                ) {
                    let scattered_color = self.ray_color(&scatter_result.scattered_ray, depth - 1);
                    return scatter_result.attenuation.component_mul(&scattered_color);
                }
            }
            return Color::zeros();
        }
        
        // Background color from skybox
        self.skybox.sample(ray.direction)
    }
    
    pub fn render_pixel(&self, camera: &Camera, x: u32, y: u32, width: u32, height: u32) -> Color {
        let mut color = Color::zeros();
        let mut rng = rand::thread_rng();
        
        for _ in 0..self.samples_per_pixel {
            let u = (x as f32 + rng.r#gen::<f32>()) / width as f32;
            let v = (y as f32 + rng.r#gen::<f32>()) / height as f32;
            
            let ray = camera.get_ray(u, 1.0 - v); // Flip V coordinate
            color += self.ray_color(&ray, self.max_depth);
        }
        
        color / self.samples_per_pixel as f32
    }
    
    pub fn render(&self, camera: &Camera, width: u32, height: u32) -> Vec<Color> {
        let mut pixels = vec![Color::zeros(); (width * height) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                pixels[index] = self.render_pixel(camera, x, y, width, height);
            }
            
            // Print progress
            if y % 10 == 0 {
                println!("Rendering line {} of {}", y, height);
            }
        }
        
        pixels
    }
}
