use image::RgbImage;
use rayon::prelude::*;

pub type Color = [u8; 3];

pub const BLACK: Color = [0, 0, 0];

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Color>,
    pub zbuffer: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![BLACK; width * height],
            zbuffer: vec![0; width * height],
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let index = (y as usize) * self.width + (x as usize);
            self.data[index] = color;
        }
    }

    pub fn to_image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                img.put_pixel(x as u32, y as u32, image::Rgb(self.data[index]));
            }
        }
        img
    }

    pub fn to_zbuffer_image(&self) -> RgbImage {
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

pub fn line(ax: i32, ay: i32, bx: i32, by: i32, framebuffer: &mut Framebuffer, color: Color) {
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
            framebuffer.set_pixel(y, x, color);
        } else {
            framebuffer.set_pixel(x, y, color);
        }
    }
}

fn signed_triangle_area(ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) -> f64 {
    0.5 * ((by - ay) * (bx + ax) + (cy - by) * (cx + bx) + (ay - cy) * (ax + cx)) as f64
}

pub fn triangle(
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
