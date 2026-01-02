use image::RgbImage;

const WHITE: [u8; 3] = [255, 255, 255];
const GREEN: [u8; 3] = [0, 255, 0];
const RED: [u8; 3] = [0, 0, 255];
const BLUE: [u8; 3] = [64, 128, 255];
const YELLOW: [u8; 3] = [255, 200, 0];

fn main() {
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 64;

    let mut img = RgbImage::new(WIDTH, HEIGHT);

    let (ax, ay) = (7, 3);
    let (bx, by) = (12, 37);
    let (cx, cy) = (62, 53);

    img.put_pixel(ax, ay, image::Rgb(RED));
    img.put_pixel(bx, by, image::Rgb(GREEN));
    img.put_pixel(cx, cy, image::Rgb(BLUE));

    img.save("output.png").unwrap();
}
