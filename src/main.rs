use clap::Parser;
use image::imageops::flip_vertical_in_place;
use rand::Rng;
use std::path::PathBuf;

mod obj;
mod render;

use render::{Framebuffer, triangle};

#[derive(Parser)]
#[command(name = "rusty-tinyrenderer")]
#[command(about = "A tiny software rasterizer", long_about = None)]
struct Args {
    /// Path to the OBJ file to render
    #[arg(short, long)]
    obj: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    if let Some(obj_path) = args.obj {
        // Load and render OBJ file
        match obj::load_obj(&obj_path) {
            Ok((vertices, faces)) => {
                println!(
                    "Loaded {} vertices and {} faces",
                    vertices.len(),
                    faces.len()
                );

                // Render each face with random colors
                let mut rng = rand::rng();
                for face in faces {
                    // Get the three vertices of the triangle
                    let v0 = vertices[face[0]];
                    let v1 = vertices[face[1]];
                    let v2 = vertices[face[2]];

                    // Convert from [-1, 1] normalized coordinates to screen coordinates
                    let screen_x = |x: f32| ((x + 1.0) * WIDTH as f32 / 2.0) as i32;
                    let screen_y = |y: f32| ((y + 1.0) * HEIGHT as f32 / 2.0) as i32;
                    let screen_z = |z: f32| (z + 1.0) * 255.0 / 2.0;

                    let ax = screen_x(v0.0);
                    let ay = screen_y(v0.1);
                    let az = screen_z(v0.2);
                    let bx = screen_x(v1.0);
                    let by = screen_y(v1.1);
                    let bz = screen_z(v1.2);
                    let cx = screen_x(v2.0);
                    let cy = screen_y(v2.1);
                    let cz = screen_z(v2.2);

                    // Generate random color for this triangle
                    let color = [rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()];

                    triangle(ax, ay, az, bx, by, bz, cx, cy, cz, &mut framebuffer, color);
                }
            }
            Err(e) => {
                eprintln!("Error loading OBJ file: {}", e);
                return;
            }
        }
    } else {
        eprintln!("Error: No OBJ file specified. Use -o <path> to specify a model.");
        eprintln!("Usage: cargo run -- -o ./models/model.obj");
        return;
    }

    // Convert framebuffer to image and save
    let mut img = framebuffer.to_image();
    let mut zbuffer_img = framebuffer.to_zbuffer_image();

    // We have to flip the image, because the tutorial assumes the origin (0,0) is at the bottom-left corner
    flip_vertical_in_place(&mut img);
    flip_vertical_in_place(&mut zbuffer_img);

    img.save("output.png").unwrap();
    zbuffer_img.save("zbuffer.png").unwrap();
    println!("Rendered to output.png and zbuffer.png");
}
