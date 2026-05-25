use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{DynamicImage, RgbaImage};
use photo_core::gpano_xmp::contains_gpano_marker;
use photo_core::{
    ConvertOptions, PanoramaMode, PanoramaOptions, SupportedFormat, convert_image, inspect_image,
    write_panorama_jpeg,
};

#[test]
fn converts_png_to_jpeg() {
    let dir = temp_dir("convert_png_to_jpeg");
    fs::create_dir_all(&dir).unwrap();
    let input = dir.join("input.png");
    let output = dir.join("output.jpg");
    DynamicImage::ImageRgba8(RgbaImage::new(16, 16))
        .save(&input)
        .unwrap();

    let result = convert_image(
        &input,
        &output,
        ConvertOptions {
            format: SupportedFormat::Jpeg,
            quality: 90,
            background: [255, 255, 255, 255],
        },
    )
    .unwrap();

    assert_eq!(result.format, SupportedFormat::Jpeg);
    assert_eq!((result.width, result.height), (16, 16));
    assert!(output.exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn writes_decodable_gpano_jpeg() {
    let dir = temp_dir("writes_decodable_gpano_jpeg");
    fs::create_dir_all(&dir).unwrap();
    let input = dir.join("input.png");
    let output = dir.join("output_360.jpg");
    DynamicImage::ImageRgba8(RgbaImage::new(80, 60))
        .save(&input)
        .unwrap();

    let result = write_panorama_jpeg(
        &input,
        &output,
        PanoramaOptions {
            mode: PanoramaMode::Pad,
            target_width: Some(100),
            quality: 90,
            background: [0, 0, 0, 255],
        },
    )
    .unwrap();

    assert_eq!((result.width, result.height), (100, 50));
    let bytes = fs::read(&output).unwrap();
    assert!(contains_gpano_marker(&bytes));

    let info = inspect_image(&output).unwrap();
    assert_eq!((info.width, info.height), (100, 50));

    let _ = fs::remove_dir_all(dir);
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("photo_tool_{name}_{nanos}"))
}
