//! Blackbox parity harness. File hashes aren't comparable against ImageMagick
//! (encoders differ), so these assert observable properties instead: exit
//! code, output extension, and pixel dimensions.

use assert_cmd::Command;
use image::{ImageFormat, RgbImage};
use std::path::{Path, PathBuf};

fn unique_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "sphene_test_{}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        name
    ))
}

/// Writes a minimal 10x10 JPEG fixture to `path`.
fn write_dummy_jpeg(path: &Path) {
    let img = RgbImage::from_pixel(10, 10, image::Rgb([128, 128, 128]));
    img.save_with_format(path, ImageFormat::Jpeg)
        .expect("failed to write dummy JPEG fixture");
}

/// Asserts the CLI produced a real output file with the expected extension and dimensions.
/// Does not compare bytes/hashes against ImageMagick output — encoders differ.
fn assert_output_properties(path: &Path, expected_ext: &str, expected_w: u32, expected_h: u32) {
    assert_eq!(
        path.extension().and_then(|e| e.to_str()),
        Some(expected_ext),
        "output file missing expected extension"
    );
    let (w, h) = image::image_dimensions(path)
        .unwrap_or_else(|e| panic!("output file at {:?} unreadable: {e}", path));
    assert_eq!((w, h), (expected_w, expected_h), "output dimensions mismatch");
}

#[test]
fn convert_dummy_jpeg_produces_expected_output() {
    let input = unique_path("input.jpg");
    let output = unique_path("output.jpg");
    write_dummy_jpeg(&input);

    let mut cmd = Command::cargo_bin("sphene").expect("sphene binary not built");
    cmd.arg("convert").arg(&input).arg(&output);
    cmd.assert().success();

    assert_output_properties(&output, "jpg", 10, 10);

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}
