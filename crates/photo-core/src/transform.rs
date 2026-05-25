use image::DynamicImage;

pub fn rotate_left(image: &DynamicImage) -> DynamicImage {
    image.rotate270()
}

pub fn rotate_right(image: &DynamicImage) -> DynamicImage {
    image.rotate90()
}

pub fn flip_horizontal(image: &DynamicImage) -> DynamicImage {
    image.fliph()
}

pub fn flip_vertical(image: &DynamicImage) -> DynamicImage {
    image.flipv()
}

pub fn adjust_brightness(image: &DynamicImage, value: i32) -> DynamicImage {
    image.brighten(value)
}

pub fn adjust_contrast(image: &DynamicImage, value: f32) -> DynamicImage {
    image.adjust_contrast(value)
}

pub fn adjust_saturation(image: &DynamicImage, value: f32) -> DynamicImage {
    let factor = 1.0 + value / 100.0;
    let mut rgba = image.to_rgba8();

    for pixel in rgba.pixels_mut() {
        let red = f32::from(pixel[0]);
        let green = f32::from(pixel[1]);
        let blue = f32::from(pixel[2]);
        let luma = red * 0.2126 + green * 0.7152 + blue * 0.0722;

        pixel[0] = clamp_channel(luma + (red - luma) * factor);
        pixel[1] = clamp_channel(luma + (green - luma) * factor);
        pixel[2] = clamp_channel(luma + (blue - luma) * factor);
    }

    DynamicImage::ImageRgba8(rgba)
}

pub fn adjust_exposure(image: &DynamicImage, stops: f32) -> DynamicImage {
    let factor = 2.0_f32.powf(stops);
    let mut rgba = image.to_rgba8();

    for pixel in rgba.pixels_mut() {
        pixel[0] = clamp_channel(f32::from(pixel[0]) * factor);
        pixel[1] = clamp_channel(f32::from(pixel[1]) * factor);
        pixel[2] = clamp_channel(f32::from(pixel[2]) * factor);
    }

    DynamicImage::ImageRgba8(rgba)
}

pub fn blur(image: &DynamicImage, sigma: f32) -> DynamicImage {
    image.blur(sigma.max(0.0))
}

pub fn sharpen(image: &DynamicImage, sigma: f32, threshold: i32) -> DynamicImage {
    image.unsharpen(sigma.max(0.1), threshold.max(0))
}

pub fn grayscale(image: &DynamicImage) -> DynamicImage {
    image.grayscale()
}

pub fn invert(image: &DynamicImage) -> DynamicImage {
    let mut image = image.clone();
    image.invert();
    image
}

fn clamp_channel(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};

    use super::{adjust_exposure, adjust_saturation, invert};

    #[test]
    fn saturation_zero_preserves_alpha() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([200, 40, 40, 77])));
        let result = adjust_saturation(&image, -100.0).to_rgba8();
        assert_eq!(result.get_pixel(0, 0)[3], 77);
        assert_eq!(result.get_pixel(0, 0)[0], result.get_pixel(0, 0)[1]);
        assert_eq!(result.get_pixel(0, 0)[1], result.get_pixel(0, 0)[2]);
    }

    #[test]
    fn exposure_lifts_rgb_and_keeps_alpha() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([10, 20, 30, 99])));
        let result = adjust_exposure(&image, 1.0).to_rgba8();
        assert_eq!(result.get_pixel(0, 0).0, [20, 40, 60, 99]);
    }

    #[test]
    fn invert_flips_rgb() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([10, 20, 30, 255])));
        let result = invert(&image).to_rgba8();
        assert_eq!(result.get_pixel(0, 0).0, [245, 235, 225, 255]);
    }
}
