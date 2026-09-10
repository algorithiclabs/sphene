//! Isolated libavif C-binding wrapper. Every `unsafe` call into `libavif_sys` lives in this
//! file; nothing outside it touches raw AVIF pointers. All libavif failures (null pointers,
//! non-OK `avifResult`s) convert to `SpheneError::IoError` — never a panic — so a bad AVIF
//! file or a misbehaving codec can't bring down the process.

use crate::{SpheneError, MAX_PIXELS};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use libavif_sys as sys;
use std::ffi::CString;
use std::os::raw::c_int;

/// Frees the decoder (and the `avifImage` it owns) on every exit path, including early returns.
struct DecoderGuard(*mut sys::avifDecoder);
impl Drop for DecoderGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { sys::avifDecoderDestroy(self.0) };
        }
    }
}

/// Frees an `avifImage*` we created ourselves (the encode path). Images owned by a decoder
/// are freed by `DecoderGuard` instead and must never be wrapped here.
struct ImageGuard(*mut sys::avifImage);
impl Drop for ImageGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { sys::avifImageDestroy(self.0) };
        }
    }
}

/// Frees the pixel buffer allocated by `avifRGBImageAllocatePixels`.
struct RgbPixelsGuard(sys::avifRGBImage);
impl Drop for RgbPixelsGuard {
    fn drop(&mut self) {
        unsafe { sys::avifRGBImageFreePixels(&mut self.0) };
    }
}

/// Frees the encoder.
struct EncoderGuard(*mut sys::avifEncoder);
impl Drop for EncoderGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { sys::avifEncoderDestroy(self.0) };
        }
    }
}

/// Frees the output buffer written by `avifEncoderWrite`.
struct RwDataGuard(sys::avifRWData);
impl Drop for RwDataGuard {
    fn drop(&mut self) {
        unsafe { sys::avifRWDataFree(&mut self.0) };
    }
}

fn avif_err(context: &str, result: sys::avifResult) -> SpheneError {
    let reason = unsafe {
        let ptr = sys::avifResultToString(result);
        if ptr.is_null() {
            "unknown libavif error".to_string()
        } else {
            std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    };
    SpheneError::IoError(format!(
        "libavif {context} failed: {reason} (code {result})"
    ))
}

/// Creates a decoder and parses an AVIF file's container/header (no pixel decode yet). Shared by
/// `read_dimensions` (header peek only) and `decode_avif` (which continues on to pixels).
fn open_and_parse(path: &str) -> Result<DecoderGuard, SpheneError> {
    unsafe {
        let decoder_ptr = sys::avifDecoderCreate();
        if decoder_ptr.is_null() {
            return Err(SpheneError::IoError(
                "libavif: avifDecoderCreate returned null".to_string(),
            ));
        }
        let decoder = DecoderGuard(decoder_ptr);
        (*decoder.0).imageSizeLimit = MAX_PIXELS as u32;

        let path = CString::new(path)
            .map_err(|e| SpheneError::IoError(format!("invalid AVIF path: {e}")))?;
        let res = sys::avifDecoderSetIOFile(decoder.0, path.as_ptr());
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifDecoderSetIOFile", res));
        }

        let res = sys::avifDecoderParse(decoder.0);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifDecoderParse", res));
        }

        Ok(decoder)
    }
}

/// Reads an AVIF file's pixel dimensions from its container header, without decoding pixel data.
/// Used to satisfy the DOS guard's "check before allocating" requirement for AVIF, which
/// `image::image_dimensions` can't peek (it doesn't know the AVIF format).
pub fn read_dimensions(path: &str) -> Result<(u32, u32), SpheneError> {
    let decoder = open_and_parse(path)?;

    unsafe {
        let avif_image = (*decoder.0).image;
        if avif_image.is_null() {
            return Err(SpheneError::IoError(
                "libavif: parse produced no image".to_string(),
            ));
        }

        let width = (*avif_image).width;
        let height = (*avif_image).height;
        if width == 0 || height == 0 {
            return Err(SpheneError::IoError(format!(
                "libavif: parsed image has invalid dimensions {width}x{height}"
            )));
        }

        Ok((width, height))
    }
}

/// Decodes an AVIF file into an RGBA image.
pub fn decode_avif(path: &str) -> Result<DynamicImage, SpheneError> {
    let decoder = open_and_parse(path)?;

    unsafe {
        let res = sys::avifDecoderNextImage(decoder.0);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifDecoderNextImage", res));
        }

        let avif_image = (*decoder.0).image;
        if avif_image.is_null() {
            return Err(SpheneError::IoError(
                "libavif: decoder produced no image".to_string(),
            ));
        }

        let width = (*avif_image).width;
        let height = (*avif_image).height;
        if width == 0 || height == 0 {
            return Err(SpheneError::IoError(format!(
                "libavif: decoded image has invalid dimensions {width}x{height}"
            )));
        }

        let mut rgb: sys::avifRGBImage = std::mem::zeroed();
        sys::avifRGBImageSetDefaults(&mut rgb, avif_image);
        rgb.format = sys::AVIF_RGB_FORMAT_RGBA;
        rgb.depth = 8;

        let res = sys::avifRGBImageAllocatePixels(&mut rgb);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifRGBImageAllocatePixels", res));
        }
        let mut pixels = RgbPixelsGuard(rgb);

        let res = sys::avifImageYUVToRGB(avif_image, &mut pixels.0);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifImageYUVToRGB", res));
        }
        if pixels.0.pixels.is_null() {
            return Err(SpheneError::IoError(
                "libavif: RGB conversion produced a null pixel buffer".to_string(),
            ));
        }

        let row_bytes = pixels.0.rowBytes as usize;
        let buffer_len = rgba_buffer_len(width, height, row_bytes)?;
        let src = std::slice::from_raw_parts(pixels.0.pixels, buffer_len);

        let out: RgbaImage = ImageBuffer::from_fn(width, height, |x, y| {
            let row_start = y as usize * row_bytes;
            let px_start = row_start + x as usize * 4;
            Rgba([
                src[px_start],
                src[px_start + 1],
                src[px_start + 2],
                src[px_start + 3],
            ])
        });

        Ok(DynamicImage::ImageRgba8(out))
    }
}

/// Encodes `img` to AVIF bytes. `quality` is 0 (worst) - 100 (best), matching libavif's own scale.
pub fn encode_avif(img: &RgbaImage, quality: u8) -> Result<Vec<u8>, SpheneError> {
    let width = img.width();
    let height = img.height();
    if width == 0 || height == 0 {
        return Err(SpheneError::IoError(
            "libavif: cannot encode a zero-sized image".to_string(),
        ));
    }

    unsafe {
        let image_ptr = sys::avifImageCreate(width, height, 8, sys::AVIF_PIXEL_FORMAT_YUV444);
        if image_ptr.is_null() {
            return Err(SpheneError::IoError(
                "libavif: avifImageCreate returned null".to_string(),
            ));
        }
        let avif_image = ImageGuard(image_ptr);

        let mut rgb: sys::avifRGBImage = std::mem::zeroed();
        sys::avifRGBImageSetDefaults(&mut rgb, avif_image.0);
        rgb.format = sys::AVIF_RGB_FORMAT_RGBA;
        rgb.depth = 8;

        let res = sys::avifRGBImageAllocatePixels(&mut rgb);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifRGBImageAllocatePixels", res));
        }
        let pixels = RgbPixelsGuard(rgb);
        if pixels.0.pixels.is_null() {
            return Err(SpheneError::IoError(
                "libavif: pixel allocation produced a null buffer".to_string(),
            ));
        }

        let row_bytes = pixels.0.rowBytes as usize;
        let buffer_len = rgba_buffer_len(width, height, row_bytes)?;
        let dst = std::slice::from_raw_parts_mut(pixels.0.pixels, buffer_len);
        for y in 0..height {
            let row_start = y as usize * row_bytes;
            for x in 0..width {
                let px = img.get_pixel(x, y);
                let px_start = row_start + x as usize * 4;
                dst[px_start] = px[0];
                dst[px_start + 1] = px[1];
                dst[px_start + 2] = px[2];
                dst[px_start + 3] = px[3];
            }
        }

        let res = sys::avifImageRGBToYUV(avif_image.0, &pixels.0);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifImageRGBToYUV", res));
        }
        drop(pixels);

        let encoder_ptr = sys::avifEncoderCreate();
        if encoder_ptr.is_null() {
            return Err(SpheneError::IoError(
                "libavif: avifEncoderCreate returned null".to_string(),
            ));
        }
        let encoder = EncoderGuard(encoder_ptr);
        (*encoder.0).quality = quality as c_int;
        (*encoder.0).qualityAlpha = quality as c_int;

        let mut output = RwDataGuard(std::mem::zeroed());
        let res = sys::avifEncoderWrite(encoder.0, avif_image.0, &mut output.0);
        if res != sys::AVIF_RESULT_OK {
            return Err(avif_err("avifEncoderWrite", res));
        }
        if output.0.data.is_null() {
            return Err(SpheneError::IoError(
                "libavif: encoder produced a null output buffer".to_string(),
            ));
        }

        Ok(std::slice::from_raw_parts(output.0.data, output.0.size).to_vec())
    }
}

fn rgba_buffer_len(width: u32, height: u32, row_bytes: usize) -> Result<usize, SpheneError> {
    let minimum_row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| SpheneError::IoError("libavif: RGBA row size overflow".to_string()))?;
    if row_bytes < minimum_row_bytes {
        return Err(SpheneError::IoError(format!(
            "libavif: RGBA row stride {row_bytes} is shorter than {minimum_row_bytes}"
        )));
    }
    row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| SpheneError::IoError("libavif: RGBA buffer size overflow".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_dimensions_color_type_and_alpha() {
        let src: RgbaImage = ImageBuffer::from_fn(6, 6, |x, y| {
            Rgba([(x * 40) as u8, (y * 40) as u8, 128, 200])
        });

        let encoded = encode_avif(&src, 80).expect("encode_avif should succeed");
        assert!(!encoded.is_empty());

        let path =
            std::env::temp_dir().join(format!("sphene_avif_roundtrip_{}.avif", std::process::id()));
        std::fs::write(&path, &encoded).unwrap();

        let decoded = decode_avif(path.to_str().unwrap()).expect("decode_avif should succeed");
        assert_eq!(decoded.width(), 6);
        assert_eq!(decoded.height(), 6);
        assert_eq!(decoded.color(), image::ColorType::Rgba8);

        let decoded_rgba = decoded.to_rgba8();
        let avg_alpha: f64 = decoded_rgba.pixels().map(|p| f64::from(p[3])).sum::<f64>()
            / f64::from(decoded_rgba.width() * decoded_rgba.height());
        assert!(
            avg_alpha < 250.0,
            "alpha channel appears to have been dropped during AVIF round-trip (avg={avg_alpha})"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn decode_rejects_invalid_data_cleanly() {
        let err = decode_avif("/nonexistent/sphene-invalid.avif").unwrap_err();
        assert!(matches!(err, SpheneError::IoError(_)));
    }

    #[test]
    fn encode_rejects_zero_sized_image_cleanly() {
        let empty: RgbaImage = ImageBuffer::new(0, 0);
        let err = encode_avif(&empty, 80).unwrap_err();
        assert!(matches!(err, SpheneError::IoError(_)));
    }
}
