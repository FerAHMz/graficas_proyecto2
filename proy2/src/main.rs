mod math_utils;
mod materials;
mod cube;
mod camera;
mod skybox;
mod raytracer;

use raylib::prelude::*;
use math_utils::{Vec3, Color, Point3f};
use materials::{create_materials, TextureManager};
use cube::{Cube, Scene};
use camera::Camera;
use raytracer::Raytracer;
use nalgebra::Point3;

fn create_scene() -> (Scene, Vec<materials::Material>) {
    let mut scene = Scene::new();
    let materials = create_materials();
    
    // Create a small island diorama
    let grass_material = 0;
    let glass_material = 1;
    let iron_material = 2;
    let diamond_material = 3;
    let water_material = 4;
    
    // Ground layer (grass blocks)
    for x in -3..4 {
        for z in -3..4 {
            let distance_from_center = ((x * x + z * z) as f32).sqrt();
            if distance_from_center <= 3.5 {
                scene.add_cube(Cube::new(
                    Point3::new(x as f32, -1.0, z as f32),
                    Point3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                    grass_material,
                ));
            }
        }
    }
    
    // Water pool in the center
    scene.add_cube(Cube::new(
        Point3::new(-1.0, 0.0, -1.0),
        Point3::new(2.0, 0.5, 2.0),
        water_material,
    ));
    
    // Iron structure
    scene.add_cube(Cube::new(
        Point3::new(-3.0, 0.0, -2.0),
        Point3::new(-2.0, 2.0, -1.0),
        iron_material,
    ));
    
    // Diamond decoration
    scene.add_cube(Cube::new(
        Point3::new(2.5, 0.0, 2.5),
        Point3::new(3.5, 1.0, 3.5),
        diamond_material,
    ));
    
    // Glass windows/barriers
    scene.add_cube(Cube::new(
        Point3::new(-1.0, 0.5, 2.0),
        Point3::new(2.0, 1.5, 2.1),
        glass_material,
    ));
    
    scene.add_cube(Cube::new(
        Point3::new(2.0, 0.5, -1.0),
        Point3::new(2.1, 1.5, 2.0),
        glass_material,
    ));
    
    (scene, materials)
}

fn setup_raytracer() -> Result<Raytracer, Box<dyn std::error::Error>> {
    let mut raytracer = Raytracer::new();
    
    // Load textures
    raytracer.load_texture("grass_side", "assets/textures/grass_side_carried.png")?;
    raytracer.load_texture("glass", "assets/textures/glass.png")?;
    raytracer.load_texture("iron_block", "assets/textures/iron_block.png")?;
    raytracer.load_texture("diamond_block", "assets/textures/diamond_block.png")?;
    raytracer.load_texture("water_still", "assets/textures/water_still.png")?;
    
    // Create scene and materials
    let (scene, materials) = create_scene();
    
    // Add materials to raytracer
    for material in materials {
        raytracer.add_material(material);
    }
    
    raytracer.scene = scene;
    raytracer.samples_per_pixel = 2; // Lower for real-time performance
    raytracer.max_depth = 5;
    
    Ok(raytracer)
}

fn color_to_raylib_color(color: Color) -> raylib::prelude::Color {
    let r = (color.x.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    let g = (color.y.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    let b = (color.z.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    raylib::prelude::Color::new(r, g, b, 255)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let screen_width = 800;
    let screen_height = 600;
    
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Minecraft Diorama - Raytracer")
        .build();
        
    rl.set_target_fps(30);
    
    // Setup raytracer
    let raytracer = setup_raytracer()?;
    
    // Setup camera
    let mut camera = Camera::new(
        Point3::new(0.0, 0.0, 0.0),  // Target center
        10.0,                        // Distance
        45.0,                        // FOV
        screen_width as f32 / screen_height as f32, // Aspect ratio
    );
    
    // Create texture for rendered image
    let render_width = 200;  // Lower resolution for real-time performance
    let render_height = 150;
    let mut render_image = Image::gen_image_color(render_width, render_height, raylib::prelude::Color::BLACK);
    let mut render_texture = rl.load_texture_from_image(&thread, &render_image).unwrap();
    
    let mut last_render_time = std::time::Instant::now();
    let mut auto_rotate = true;
    
    println!("Controls:");
    println!("- Mouse: Rotate camera");
    println!("- Mouse Wheel: Zoom in/out");
    println!("- SPACE: Toggle auto-rotation");
    println!("- R: Re-render scene");
    println!("- ESC: Exit");
    
    while !rl.window_should_close() {
        // Handle input
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            auto_rotate = !auto_rotate;
        }
        
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            last_render_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
        }
        
        // Camera controls
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_delta = rl.get_mouse_delta();
            camera.rotate(
                mouse_delta.x * 0.01,
                mouse_delta.y * 0.01,
            );
            last_render_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
        }
        
        let wheel_move = rl.get_mouse_wheel_move();
        if wheel_move != 0.0 {
            camera.zoom(-wheel_move * 0.5);
            last_render_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
        }
        
        // Auto rotation
        if auto_rotate {
            camera.rotate(0.005, 0.0);
            last_render_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
        }
        
        // Re-render if needed (every second or when camera moves)
        if last_render_time.elapsed().as_secs() >= 1 {
            println!("Rendering frame...");
            let start_time = std::time::Instant::now();
            
            let pixels = raytracer.render(&camera, render_width as u32, render_height as u32);
            
            // Update texture
            for y in 0..render_height {
                for x in 0..render_width {
                    let index = (y * render_width + x) as usize;
                    let color = color_to_raylib_color(pixels[index]);
                    unsafe {
                        render_image.draw_pixel(x as i32, y as i32, color);
                    }
                }
            }
            
            render_texture = rl.load_texture_from_image(&thread, &render_image).unwrap();
            
            let render_time = start_time.elapsed();
            println!("Render completed in {:.2}s", render_time.as_secs_f32());
            
            last_render_time = std::time::Instant::now();
        }
        
        // Drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(raylib::prelude::Color::BLACK);
        
        // Draw rendered image scaled to screen
        d.draw_texture_ex(
            &render_texture,
            Vector2::zero(),
            0.0,
            screen_width as f32 / render_width as f32,
            raylib::prelude::Color::WHITE,
        );
        
        // Draw UI
        d.draw_text(
            &format!("Camera Distance: {:.1}", camera.distance),
            10, 10, 20, raylib::prelude::Color::WHITE
        );
        
        d.draw_text(
            &format!("Auto-rotate: {}", if auto_rotate { "ON" } else { "OFF" }),
            10, 35, 20, raylib::prelude::Color::WHITE
        );
        
        d.draw_text(
            "Press SPACE to toggle rotation, R to re-render",
            10, screen_height - 25, 16, raylib::prelude::Color::LIGHTGRAY
        );
    }
    
    Ok(())
}
