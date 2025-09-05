use crate::math_utils::{Vec3, Point3f, Ray};
use nalgebra::Point3;

pub struct Camera {
    pub position: Point3f,
    pub target: Point3f,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    
    // Orbital controls
    pub distance: f32,
    pub theta: f32,  // Horizontal angle
    pub phi: f32,    // Vertical angle
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Camera {
    pub fn new(target: Point3f, distance: f32, fov: f32, aspect_ratio: f32) -> Self {
        let mut camera = Self {
            position: Point3::new(0.0, 0.0, 0.0),
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            fov,
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
            distance,
            theta: 0.0,
            phi: std::f32::consts::PI * 0.25, // 45 degrees
            min_distance: 2.0,
            max_distance: 50.0,
        };
        camera.update_position();
        camera
    }
    
    pub fn update_position(&mut self) {
        // Convert spherical coordinates to cartesian
        let x = self.distance * self.phi.sin() * self.theta.cos();
        let y = self.distance * self.phi.cos();
        let z = self.distance * self.phi.sin() * self.theta.sin();
        
        self.position = self.target + Vec3::new(x, y, z);
    }
    
    pub fn rotate(&mut self, delta_theta: f32, delta_phi: f32) {
        self.theta += delta_theta;
        self.phi += delta_phi;
        
        // Clamp phi to avoid gimbal lock
        self.phi = self.phi.clamp(0.1, std::f32::consts::PI - 0.1);
        
        self.update_position();
    }
    
    pub fn zoom(&mut self, delta: f32) {
        self.distance += delta;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
        self.update_position();
    }
    
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        // Convert screen coordinates to world ray
        let theta = self.fov * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = self.aspect_ratio * half_height;
        
        let w = (self.position - self.target).normalize();
        let u_vec = self.up.cross(&w).normalize();
        let v_vec = w.cross(&u_vec);
        
        let lower_left_corner = self.position.coords 
            - half_width * u_vec 
            - half_height * v_vec 
            - w;
            
        let horizontal = 2.0 * half_width * u_vec;
        let vertical = 2.0 * half_height * v_vec;
        
        let direction = lower_left_corner + u * horizontal + v * vertical - self.position.coords;
        
        Ray::new(self.position, direction.normalize())
    }
    
    pub fn get_view_matrix(&self) -> nalgebra::Matrix4<f32> {
        nalgebra::Matrix4::look_at_rh(
            &self.position,
            &self.target,
            &self.up
        )
    }
}
