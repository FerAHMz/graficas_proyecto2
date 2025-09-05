use crate::math_utils::{Vec3, Point3f, Ray, EPSILON};
use crate::materials::Material;

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub material_index: usize,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.direction.dot(&outward_normal) < 0.0;
        self.normal = if self.front_face { outward_normal } else { -outward_normal };
    }
}

#[derive(Debug, Clone)]
pub struct Cube {
    pub min: Point3f,
    pub max: Point3f,
    pub material_index: usize,
}

impl Cube {
    pub fn new(min: Point3f, max: Point3f, material_index: usize) -> Self {
        Self { min, max, material_index }
    }
    
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut t_near = t_min;
        let mut t_far = t_max;
        let mut hit_face = 0; // 0=x, 1=y, 2=z, with sign indicating direction
        
        // Check intersection with each pair of parallel planes
        for axis in 0..3 {
            let ray_dir = ray.direction[axis];
            let ray_orig = ray.origin[axis];
            let min_val = self.min[axis];
            let max_val = self.max[axis];
            
            if ray_dir.abs() < EPSILON {
                // Ray is parallel to the planes
                if ray_orig < min_val || ray_orig > max_val {
                    return None;
                }
            } else {
                let t1 = (min_val - ray_orig) / ray_dir;
                let t2 = (max_val - ray_orig) / ray_dir;
                
                let (t_min_axis, t_max_axis) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
                
                if t_min_axis > t_near {
                    t_near = t_min_axis;
                    hit_face = if t1 < t2 { -(axis as i32 + 1) } else { axis as i32 + 1 };
                }
                
                if t_max_axis < t_far {
                    t_far = t_max_axis;
                }
                
                if t_near > t_far {
                    return None;
                }
            }
        }
        
        if t_near > t_max || t_far < t_min {
            return None;
        }
        
        let t = if t_near > t_min { t_near } else { t_far };
        if t < t_min || t > t_max {
            return None;
        }
        
        let hit_point = ray.at(t);
        let (normal, u, v) = self.get_face_normal_and_uv(hit_point.coords, hit_face);
        
        let mut hit_record = HitRecord {
            point: hit_point.coords,
            normal: Vec3::zeros(),
            t,
            u,
            v,
            material_index: self.material_index,
            front_face: false,
        };
        
        hit_record.set_face_normal(ray, normal);
        Some(hit_record)
    }
    
    fn get_face_normal_and_uv(&self, point: Vec3, face: i32) -> (Vec3, f32, f32) {
        let size = self.max - self.min;
        let relative = point - self.min.coords;
        
        match face.abs() {
            1 => { // X face
                let normal = if face > 0 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(-1.0, 0.0, 0.0) };
                let u = relative.z / size.z;
                let v = relative.y / size.y;
                (normal, u, v)
            },
            2 => { // Y face  
                let normal = if face > 0 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(0.0, -1.0, 0.0) };
                let u = relative.x / size.x;
                let v = relative.z / size.z;
                (normal, u, v)
            },
            3 => { // Z face
                let normal = if face > 0 { Vec3::new(0.0, 0.0, 1.0) } else { Vec3::new(0.0, 0.0, -1.0) };
                let u = relative.x / size.x;
                let v = relative.y / size.y;
                (normal, u, v)
            },
            _ => (Vec3::new(0.0, 1.0, 0.0), 0.0, 0.0),
        }
    }
}

pub struct Scene {
    pub cubes: Vec<Cube>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cubes: Vec::new(),
        }
    }
    
    pub fn add_cube(&mut self, cube: Cube) {
        self.cubes.push(cube);
    }
    
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        let mut closest_t = t_max;
        
        for cube in &self.cubes {
            if let Some(hit) = cube.hit(ray, t_min, closest_t) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }
        
        closest_hit
    }
}
