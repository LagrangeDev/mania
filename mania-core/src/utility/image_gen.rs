use image::{ImageBuffer, ImageFormat, ImageResult, Rgb};
use noise::{NoiseFn, Perlin};
use std::io::{Seek, Write};

pub trait ImageOpWriteSeek: Write + Seek {}
impl<T: Write + Seek> ImageOpWriteSeek for T {}

// TODO: make it more configurable
pub fn gen_thumbnail<W: ImageOpWriteSeek>(output: &mut W) -> ImageResult<()> {
    let (width, height, scale) = (1024, 768, 500.0);
    let (min_color, max_color) = ((40, 20, 20), (60, 40, 40));
    let perlin = Perlin::new(114514);
    let lerp =
        |min: u8, max: u8, t: f64| -> u8 { (min as f64 + t * (max as f64 - min as f64)) as u8 };
    let mut img = ImageBuffer::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let nx = x as f64 / scale;
        let ny = y as f64 / scale;
        let t = (perlin.get([nx, ny]) + 1.0) / 2.0;
        *pixel = Rgb([
            lerp(min_color.0, max_color.0, t),
            lerp(min_color.1, max_color.1, t),
            lerp(min_color.2, max_color.2, t),
        ]);
    }
    img.write_to(output, ImageFormat::Jpeg)?;
    Ok(())
}
