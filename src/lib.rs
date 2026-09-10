mod avif;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::fs::File;
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpheneError {
    #[error("Failed to read image file from path: {0}")]
    IoError(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("Unknown error occurred during image processing")]
    Unknown,
    #[error("Requested dimensions {0}x{1} exceed the 250-megapixel safety limit")]
    DimensionsTooLarge(u32, u32),
}

/// Hard DOS-protection ceiling: total pixel count must stay under this before any buffer allocation.
// ponytail: 250MP is the v0.1 compatibility ceiling. Chunked processing deferred to v0.6.
pub(crate) const MAX_PIXELS: u64 = 250_000_000;

/// Rejects dimensions whose pixel count would hit or exceed `MAX_PIXELS`.
/// Must be called before any pixel buffer is allocated.
pub fn check_dimensions(width: u32, height: u32) -> Result<(), SpheneError> {
    if (width as u64) * (height as u64) >= MAX_PIXELS {
        return Err(SpheneError::DimensionsTooLarge(width, height));
    }
    Ok(())
}

/// Flags the native engine parses itself. Anything else routes to the ImageMagick fallback.
const NATIVE_FLAGS: [&str; 2] = ["-resize", "-quality"];
/// ImageMagick resize modifiers (`!`, `>`, `<`, `^`) that the native resize path doesn't implement.
const RESIZE_MODIFIERS: [char; 4] = ['!', '>', '<', '^'];
const NATIVE_EXTENSIONS: [&str; 7] = ["avif", "heic", "heif", "jpeg", "jpg", "png", "webp"];

fn has_native_extension(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            NATIVE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

fn native_input_output(args: &[String]) -> Option<(&str, &str)> {
    let input = args.get(1)?.as_str();
    let mut index = 2;

    while let Some(arg) = args.get(index) {
        if NATIVE_FLAGS.contains(&arg.as_str()) {
            index += 2;
            continue;
        }
        if arg.starts_with('-') || index + 1 != args.len() {
            return None;
        }
        return Some((input, arg));
    }

    None
}

/// Strangler Fig gate: scans raw CLI args for syntax the native engine can't handle
/// (STDIN `-`, resize modifiers, or any flag outside `NATIVE_FLAGS`) so parsing can
/// halt before clap ever touches them, per the PRD's <5ms handoff requirement.
pub fn needs_fallback(args: &[String]) -> bool {
    if let Some((input, output)) = native_input_output(args) {
        if !has_native_extension(input) || !has_native_extension(output) {
            return true;
        }
    }

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-" {
            return true;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            if !NATIVE_FLAGS.contains(&arg.as_str()) {
                return true;
            }
            if args
                .get(i + 1)
                .is_some_and(|v| v.contains(RESIZE_MODIFIERS))
            {
                return true;
            }
            i += 1;
        }
        i += 1;
    }
    false
}

/// Spawns ImageMagick with raw CLI arguments minus Sphene's binary name.
/// Prefers ImageMagick 7's `magick`, then falls back to ImageMagick 6's `convert`.
pub fn spawn_fallback(raw_args: &[String]) -> Result<i32, SpheneError> {
    let args = raw_args.get(1..).unwrap_or_default();
    let status = match std::process::Command::new("magick").args(args).status() {
        Ok(status) => status,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::process::Command::new("convert")
                .args(args)
                .status()
                .map_err(|e| SpheneError::IoError(e.to_string()))?
        }
        Err(error) => return Err(SpheneError::IoError(error.to_string())),
    };
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn native_convert_syntax_does_not_fall_back() {
        assert!(!needs_fallback(&args(&["convert", "in.jpg", "out.png"])));
        assert!(!needs_fallback(&args(&[
            "convert", "in.jpg", "-resize", "800x600", "-quality", "80", "out.png"
        ])));
    }

    #[test]
    fn stdin_flag_falls_back() {
        assert!(needs_fallback(&args(&["convert", "-", "out.png"])));
    }

    #[test]
    fn resize_modifier_falls_back() {
        assert!(needs_fallback(&args(&[
            "convert", "in.jpg", "-resize", "800x600^", "out.png"
        ])));
    }

    #[test]
    fn unknown_flag_falls_back() {
        assert!(needs_fallback(&args(&[
            "convert",
            "in.jpg",
            "-polaroid",
            "out.png"
        ])));
    }

    #[test]
    fn unknown_extensions_fall_back() {
        assert!(needs_fallback(&args(&["convert", "in.unknown", "out.png"])));
        assert!(needs_fallback(&args(&["convert", "in.png", "out.unknown"])));
    }

    #[test]
    fn malformed_native_flag_stays_native() {
        assert!(!needs_fallback(&args(&[
            "convert", "in.jpg", "-resize", "garbage"
        ])));
    }
}

#[cfg(test)]
mod dos_guard_tests {
    use super::*;

    #[test]
    fn dimensions_at_or_over_limit_abort_cleanly() {
        // 20_000 * 20_000 = 400_000_000, over the 250MP ceiling.
        let err = check_dimensions(20_000, 20_000).unwrap_err();
        assert!(matches!(
            err,
            SpheneError::DimensionsTooLarge(20_000, 20_000)
        ));
    }

    #[test]
    fn dimensions_at_exact_limit_abort_cleanly() {
        let err = check_dimensions(25_000, 10_000).unwrap_err();
        assert!(matches!(
            err,
            SpheneError::DimensionsTooLarge(25_000, 10_000)
        ));
    }

    #[test]
    fn dimensions_that_would_overflow_u32_abort_cleanly() {
        // 65_536 * 65_536 overflows u32::MAX; must not panic on the multiply.
        let err = check_dimensions(65_536, 65_536).unwrap_err();
        assert!(matches!(err, SpheneError::DimensionsTooLarge(_, _)));
    }

    #[test]
    fn dimensions_within_limit_are_accepted() {
        assert!(check_dimensions(5_000, 5_000).is_ok()); // 25_000_000 < limit
    }

    #[test]
    fn resize_image_rejects_oversized_dimensions_before_touching_pixels() {
        let err = resize_image("in.jpg", "out.jpg", 20_000, 20_000).unwrap_err();
        assert!(matches!(err, SpheneError::DimensionsTooLarge(_, _)));
    }
}

#[cfg(test)]
mod alpha_tests {
    use super::*;

    fn unique_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("sphene_alpha_test_{}_{name}", std::process::id()))
    }

    #[test]
    fn resize_preserves_alpha_channel() {
        // Uniform semi-transparent image: a Lanczos3 resize of a constant field should
        // reproduce the same constant, so this isolates alpha survival from filter ringing
        // (which a hard transparent/opaque edge would otherwise introduce).
        let src: RgbaImage = ImageBuffer::from_pixel(4, 4, Rgba([10u8, 20, 30, 128]));

        let input = unique_path("input.png");
        let output = unique_path("output.png");
        src.save(&input).unwrap();

        resize_image(input.to_str().unwrap(), output.to_str().unwrap(), 2, 2).unwrap();

        let decoded = image::open(&output).unwrap();
        assert_eq!(decoded.color(), image::ColorType::Rgba8);

        let rgba = decoded.to_rgba8();
        for pixel in rgba.pixels() {
            assert_eq!(
                pixel[3], 128,
                "alpha channel should survive resize untouched"
            );
        }

        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
    }
}

/// sRGB electro-optical transfer function inverse: gamma-encoded [0,1] -> linear light.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB opto-electronic transfer function: linear light -> gamma-encoded [0,1].
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Resizes `img` to exactly `width`x`height` (no aspect-ratio adjustment). The R, G, B
/// channels are resized in linear-light space (sRGB -> linear -> Lanczos3 -> sRGB) to avoid
/// the dark-artifact bias of filtering directly in gamma-encoded space; the A channel is
/// carried through the transfer-function conversions untouched (it's a coverage value, not
/// a light intensity) but is still resampled by the Lanczos3 filter along with the color data.
fn resize_in_linear_space(img: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    let srgb_f32 = img.to_rgba32f();
    let linear = ImageBuffer::from_fn(srgb_f32.width(), srgb_f32.height(), |x, y| {
        let p = srgb_f32.get_pixel(x, y);
        Rgba([
            srgb_to_linear(p[0]),
            srgb_to_linear(p[1]),
            srgb_to_linear(p[2]),
            p[3],
        ])
    });

    let resized_linear = image::imageops::resize(&linear, width, height, FilterType::Lanczos3);

    ImageBuffer::from_fn(width, height, |x, y| {
        let p = resized_linear.get_pixel(x, y);
        Rgba([
            (linear_to_srgb(p[0]).clamp(0.0, 1.0) * 255.0).round() as u8,
            (linear_to_srgb(p[1]).clamp(0.0, 1.0) * 255.0).round() as u8,
            (linear_to_srgb(p[2]).clamp(0.0, 1.0) * 255.0).round() as u8,
            (p[3].clamp(0.0, 1.0) * 255.0).round() as u8,
        ])
    })
}

/// Peeks `input`'s pixel dimensions without decoding it, for the DOS guard. AVIF isn't
/// supported by `image`'s own format sniffing, so it's routed to the libavif wrapper instead.
fn peek_dimensions(input: &str, data: &[u8]) -> Result<(u32, u32), SpheneError> {
    if matches!(
        image::ImageFormat::from_path(input),
        Ok(image::ImageFormat::Avif)
    ) {
        return avif::read_dimensions(data);
    }
    image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| SpheneError::IoError(e.to_string()))?
        .into_dimensions()
        .map_err(|e| SpheneError::IoError(e.to_string()))
}

/// Decodes `input`, routing AVIF through the libavif wrapper and everything else through `image`.
fn open_image(input: &str, data: &[u8]) -> Result<DynamicImage, SpheneError> {
    if matches!(
        image::ImageFormat::from_path(input),
        Ok(image::ImageFormat::Avif)
    ) {
        return avif::decode_avif(data);
    }
    image::load_from_memory(data).map_err(|e| SpheneError::IoError(e.to_string()))
}

/// Computes dimensions that fit within `target_w`x`target_h` while preserving aspect ratio.
/// This is ImageMagick's default `WxH` sizing behavior; modifiers (`^`, `!`, `>`, `<`) that
/// change this behavior are routed to the fallback by `needs_fallback` before reaching here.
fn fit_within(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32) {
    let ratio = f64::min(
        target_w as f64 / src_w as f64,
        target_h as f64 / src_h as f64,
    );
    let new_w = ((src_w as f64 * ratio).round() as u32).max(1);
    let new_h = ((src_h as f64 * ratio).round() as u32).max(1);
    (new_w, new_h)
}

/// Default WebP quality (0.0-100.0) used when `-quality` isn't given; matches `webp`'s own default.
const DEFAULT_WEBP_QUALITY: f32 = 75.0;
/// Default AVIF quality (0-100, libavif's own scale) used when `-quality` isn't given.
const DEFAULT_AVIF_QUALITY: u8 = 75;

/// Writes `img` to `output`, inferring format from its extension. Honors `quality` for JPEG
/// (lossy) and WEBP (lossy, via libwebp). PNG has no quality concept and ignores it.
fn encode_output(img: RgbaImage, output: &str, quality: Option<u8>) -> Result<(), SpheneError> {
    let format = image::ImageFormat::from_path(output)
        .map_err(|e| SpheneError::UnsupportedFormat(e.to_string()))?;

    match format {
        image::ImageFormat::Jpeg => {
            // JPEG has no alpha channel; flatten before encoding.
            let rgb = DynamicImage::ImageRgba8(img).to_rgb8();
            if let Some(q) = quality {
                let mut file =
                    File::create(output).map_err(|e| SpheneError::IoError(e.to_string()))?;
                let encoder = JpegEncoder::new_with_quality(&mut file, q);
                rgb.write_with_encoder(encoder)
                    .map_err(|e| SpheneError::IoError(e.to_string()))
            } else {
                rgb.save_with_format(output, format)
                    .map_err(|e| SpheneError::IoError(e.to_string()))
            }
        }
        image::ImageFormat::WebP => {
            let quality = quality.map(f32::from).unwrap_or(DEFAULT_WEBP_QUALITY);
            let dynamic = DynamicImage::ImageRgba8(img);
            let encoder = webp::Encoder::from_image(&dynamic)
                .map_err(|e| SpheneError::UnsupportedFormat(e.to_string()))?;
            let encoded = encoder.encode(quality);
            std::fs::write(output, &*encoded).map_err(|e| SpheneError::IoError(e.to_string()))
        }
        image::ImageFormat::Avif => {
            let quality = quality.unwrap_or(DEFAULT_AVIF_QUALITY);
            let encoded = avif::encode_avif(&img, quality)?;
            std::fs::write(output, &encoded).map_err(|e| SpheneError::IoError(e.to_string()))
        }
        _ => img
            .save_with_format(output, format)
            .map_err(|e| SpheneError::IoError(e.to_string())),
    }
}

/// Resizes `input` to exactly `width`x`height` (no aspect-ratio adjustment) and writes it to `output`.
pub fn resize_image(input: &str, output: &str, width: u32, height: u32) -> Result<(), SpheneError> {
    check_dimensions(width, height)?;

    let data = std::fs::read(input).map_err(|e| SpheneError::IoError(e.to_string()))?;
    let (src_w, src_h) = peek_dimensions(input, &data)?;
    check_dimensions(src_w, src_h)?;

    let img = open_image(input, &data)?;
    let resized = resize_in_linear_space(&img, width, height);
    encode_output(resized, output, None)
}

/// Reads `input`, optionally resizes it (preserving aspect ratio to fit within `resize`),
/// and writes the result to `output`, honoring `quality` where the output format supports it.
pub fn convert_image(
    input: &str,
    output: &str,
    resize: Option<(u32, u32)>,
    quality: Option<u8>,
) -> Result<(), SpheneError> {
    if let Some((target_w, target_h)) = resize {
        check_dimensions(target_w, target_h)?;
    }

    let data = std::fs::read(input).map_err(|e| SpheneError::IoError(e.to_string()))?;
    let (src_w, src_h) = peek_dimensions(input, &data)?;
    check_dimensions(src_w, src_h)?;

    let img = open_image(input, &data)?;

    let final_img = match resize {
        Some((target_w, target_h)) => {
            let (new_w, new_h) = fit_within(src_w, src_h, target_w, target_h);
            check_dimensions(new_w, new_h)?;
            resize_in_linear_space(&img, new_w, new_h)
        }
        None => img.to_rgba8(),
    };

    encode_output(final_img, output, quality)
}
