use image::RgbImage;
// use image::imageops::flip_vertical_in_place;

type Color = [u8; 3];

const WHITE: Color = [255, 255, 255];
const GREEN: Color = [0, 255, 0];
const RED: Color = [255, 0, 0];
const BLUE: Color = [64, 128, 255];
const YELLOW: Color = [255, 200, 0];

fn line(ax: u32, ay: u32, bx: u32, by: u32, img: &mut RgbImage, color: Color) {
    let mut ax = ax;
    let mut ay = ay;
    let mut bx = bx;
    let mut by = by;

    let steep = (ax as i32 - bx as i32).abs() < (ay as i32 - by as i32).abs();
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
        let y = (ay as f32 + (by as f32 - ay as f32) * t).round() as u32;

        if steep {
            // if transposed, de-transpose
            img.put_pixel(y, x, image::Rgb(color));
        } else {
            img.put_pixel(x, y, image::Rgb(color));
        }
    }
}

fn triangle(
    ax: u32,
    ay: u32,
    bx: u32,
    by: u32,
    cx: u32,
    cy: u32,
    img: &mut RgbImage,
    color: Color,
) {
    line(ax, ay, bx, by, img, color);
    line(bx, by, cx, cy, img, color);
    line(cx, cy, ax, ay, img, color);
}

fn main() {
    const WIDTH: u32 = 128;
    const HEIGHT: u32 = 128;

    let mut img = RgbImage::new(WIDTH, HEIGHT);

    let (ax, ay) = (7, 61);
    let (bx, by) = (12, 27);
    let (cx, cy) = (62, 11);

    triangle(7, 45, 35, 100, 45, 60, &mut img, RED);

    triangle(120, 35, 90, 5, 45, 110, &mut img, WHITE);

    triangle(115, 83, 80, 90, 85, 120, &mut img, GREEN);
    // We have to flip the image, because the tutorial assumes the origin (0,0) is at the bottom-left corner
    // flip_vertical_in_place(&mut img);

    img.save("output.png").unwrap();
}
