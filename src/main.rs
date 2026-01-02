use clap::Parser;
use image::RgbImage;
use image::imageops::flip_vertical_in_place;
use rand::Rng;
use rayon::prelude::*;
use std::path::PathBuf;

mod obj;

type Color = [u8; 3];

const WHITE: Color = [255, 255, 255];
const GREEN: Color = [0, 255, 0];
const RED: Color = [255, 0, 0];
const BLUE: Color = [64, 128, 255];
const YELLOW: Color = [255, 200, 0];
const BLACK: Color = [0, 0, 0];

#[derive(Parser)]
#[command(name = "rusty-tinyrenderer")]
#[command(about = "A tiny software rasterizer", long_about = None)]
struct Args {
    /// Path to the OBJ file to render
    #[arg(short, long)]
    obj: Option<PathBuf>,
}

struct Framebuffer {
    width: usize,
    height: usize,
    data: Vec<Color>,
    zbuffer: Vec<u8>,
}

impl Framebuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![BLACK; width * height],
            zbuffer: vec![0; width * height],
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let index = (y as usize) * self.width + (x as usize);
            self.data[index] = color;
        }
    }

    fn to_image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                img.put_pixel(x as u32, y as u32, image::Rgb(self.data[index]));
            }
        }
        img
    }

    fn to_zbuffer_image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let z = self.zbuffer[index];
                img.put_pixel(x as u32, y as u32, image::Rgb([z, z, z]));
            }
        }
        img
    }
}

fn line(ax: i32, ay: i32, bx: i32, by: i32, img: &mut RgbImage, color: Color) {
    let mut ax = ax;
    let mut ay = ay;
    let mut bx = bx;
    let mut by = by;

    let steep = (ax - bx).abs() < (ay - by).abs();
    if steep {
        // if the line is steep, we transpose the image
        std::mem::swap(&mut ax, &mut ay);
        std::mem::swap(&mut bx, &mut by);
    }

    if ax > bx {
        // make it left-to-right
        std::mem::swap(&mut ax, &mut bx);
        std::mem::swap(&mut ay, &mut by);
    }

    for x in ax..=bx {
        let t = (x - ax) as f32 / (bx - ax) as f32;
        let y = (ay as f32 + (by - ay) as f32 * t).round() as i32;

        if steep {
            // if transposed, de-transpose
            img.put_pixel(y as u32, x as u32, image::Rgb(color));
        } else {
            img.put_pixel(x as u32, y as u32, image::Rgb(color));
        }
    }
}

fn signed_triangle_area(ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) -> f64 {
    0.5 * ((by - ay) * (bx + ax) + (cy - by) * (cx + bx) + (ay - cy) * (ax + cx)) as f64
}

fn triangle(
    ax: i32,
    ay: i32,
    az: f32,
    bx: i32,
    by: i32,
    bz: f32,
    cx: i32,
    cy: i32,
    cz: f32,
    framebuffer: &mut Framebuffer,
    color: Color,
) {
    // bounding box for the triangle
    let bbminx = ax.min(bx).min(cx);
    let bbminy = ay.min(by).min(cy);
    let bbmaxx = ax.max(bx).max(cx);
    let bbmaxy = ay.max(by).max(cy);

    let total_area = signed_triangle_area(ax, ay, bx, by, cx, cy);
    if total_area < 1.0 {
        return; // backface culling + discarding triangles that cover less than a pixel
    }

    let width = framebuffer.width;
    let height = framebuffer.height;
    let data_ptr = framebuffer.data.as_mut_ptr() as usize;
    let zbuffer_ptr = framebuffer.zbuffer.as_mut_ptr() as usize;

    // Process pixels in parallel and write directly to framebuffer
    // SAFETY: Each thread processes a unique x column, so there are no data races
    (bbminx..=bbmaxx).into_par_iter().for_each(|x| {
        for y in bbminy..=bbmaxy {
            let alpha = signed_triangle_area(x, y, bx, by, cx, cy) / total_area;
            let beta = signed_triangle_area(x, y, cx, cy, ax, ay) / total_area;
            let gamma = signed_triangle_area(x, y, ax, ay, bx, by) / total_area;

            // negative barycentric coordinate => the pixel is outside the triangle
            if alpha < 0.0 || beta < 0.0 || gamma < 0.0 {
                continue;
            }

            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let index = (y as usize) * width + (x as usize);

                // Interpolate z-depth using barycentric coordinates
                let z = (alpha * az as f64 + beta * bz as f64 + gamma * cz as f64) as u8;

                unsafe {
                    let zbuf_ptr = (zbuffer_ptr as *mut u8).add(index);

                    // Z-buffer test: only draw if this pixel is closer (higher z value)
                    if z <= *zbuf_ptr {
                        continue;
                    }

                    // Update z-buffer and framebuffer
                    *zbuf_ptr = z;
                    let color_ptr = (data_ptr as *mut Color).add(index);
                    *color_ptr = color;
                }
            }
        }
    });
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
