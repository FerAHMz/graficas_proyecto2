use nalgebra::{Vector3, Point3};

pub type Vec3 = Vector3<f32>;
pub type Point3f = Point3<f32>;
pub type Color = Vec3;

pub const EPSILON: f32 = 1e-6;

pub fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(&normal) * normal
}

pub fn refract(incident: Vec3, normal: Vec3, eta: f32) -> Option<Vec3> {
    let cos_i = -incident.dot(&normal);
    let sin_t2 = eta * eta * (1.0 - cos_i * cos_i);
    
    if sin_t2 >= 1.0 {
        None // Total internal reflection
    } else {
        let cos_t = (1.0 - sin_t2).sqrt();
        Some(eta * incident + (eta * cos_i - cos_t) * normal)
    }
}

pub fn fresnel(cos_i: f32, eta: f32) -> f32 {
    let sin_t2 = eta * eta * (1.0 - cos_i * cos_i);
    if sin_t2 >= 1.0 {
        return 1.0; // Total internal reflection
    }
    
    let cos_t = (1.0 - sin_t2).sqrt();
    let r_parallel = (eta * cos_i - cos_t) / (eta * cos_i + cos_t);
    let r_perpendicular = (cos_i - eta * cos_t) / (cos_i + eta * cos_t);
    
    0.5 * (r_parallel * r_parallel + r_perpendicular * r_perpendicular)
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3f,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3f, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    pub fn at(&self, t: f32) -> Point3f {
        self.origin + t * self.direction
    }
}

pub fn random_in_unit_sphere() -> Vec3 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    loop {
        let p = Vec3::new(
            rng.r#gen_range(-1.0..1.0),
            rng.r#gen_range(-1.0..1.0),
            rng.r#gen_range(-1.0..1.0),
        );
        if p.magnitude_squared() < 1.0 {
            return p;
        }
    }
}

pub trait Lerp {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }
}
