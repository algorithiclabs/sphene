//! Blackbox parity harness. File hashes aren't comparable against ImageMagick
//! (encoders differ), so these assert observable properties instead: exit
//! code, output extension, and pixel dimensions.

use assert_cmd::Command;
use image::{ImageFormat, RgbImage};
use std::os::unix::process::CommandExt;
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
    assert_eq!(
        (w, h),
        (expected_w, expected_h),
        "output dimensions mismatch"
    );
}

fn assert_native_syntax(args: &[&str], output: &Path) {
    let input = unique_path("input.jpg");
    write_dummy_jpeg(&input);

    let mut cmd = Command::cargo_bin("sphene").expect("sphene binary not built");
    cmd.args(args).arg(&input).arg(output);
    cmd.assert().success();

    assert_output_properties(&output, "jpg", 10, 10);

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(output);
}

#[test]
fn im_v6_and_v7_native_syntaxes_produce_expected_output() {
    let direct_output = unique_path("direct-output.jpg");
    assert_native_syntax(&[], &direct_output);

    let convert_output = unique_path("convert-output.jpg");
    assert_native_syntax(&["convert"], &convert_output);

    let input = unique_path("magick-convert-input.jpg");
    let output = unique_path("magick-convert-output.jpg");
    write_dummy_jpeg(&input);

    let status = std::process::Command::new(assert_cmd::cargo::cargo_bin("sphene"))
        .arg0("magick")
        .arg("convert")
        .arg(&input)
        .arg(&output)
        .status()
        .expect("failed to launch sphene");
    assert!(status.success());
    assert_output_properties(&output, "jpg", 10, 10);

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}

#[test]
fn sphene_owns_help_and_version_flags() {
    for flag in ["--help", "--version"] {
        Command::cargo_bin("sphene")
            .expect("sphene binary not built")
            .arg(flag)
            .assert()
            .success()
            .stdout(predicates::str::contains("sphene"));
    }
}
