# Sphene

Sphene is a memory-safe, ImageMagick-compatible image conversion CLI for cloud and backend workloads. It keeps native processing bounded before decoding image pixels, helping protect workers from oversized or malicious image inputs.

### Decompression bomb protection

A 107 KB solid-color PNG can declare dimensions of 30,000 × 30,000 pixels: 900 million pixels in a tiny upload.

**ImageMagick (v7.1.2)**

- **Peak memory:** 15.1 GB
- **Processing time:** 16.36s
- **Risk:** On a memory-constrained cloud worker, an allocation of this size could trigger OOM termination and take down the worker.

**Sphene (v0.1.1)**

- **Peak memory:** 2.7 MB
- **Processing time:** < 0.01s
- **Result:** Rejects the image before decoding with `DimensionsTooLarge(30000, 30000)`.

Sphene enforces this 250-megapixel limit on its native processing path. Unsupported flags and formats are forwarded to an installed ImageMagick `magick` or `convert` executable for compatibility.

## Why teams use Sphene

- **Protect workers at the image boundary.** Oversized images are rejected before pixel buffers are allocated, turning a potential memory event into a bounded request failure.
- **Keep familiar workflows.** Common file-to-file conversions run natively; unsupported ImageMagick syntax is forwarded to the existing `magick` or `convert` installation.
- **Get a fast native path.** Native resizing uses linear-light RGB processing and format-aware quality controls while preserving the existing CLI shape.
- **See compatibility decisions.** Set `RUST_LOG=warn` to see why a command was delegated to ImageMagick.

### Everyday conversion benchmark

On an Apple Silicon arm64 system, Sphene v0.1.1 converted a 6000 × 4000 gradient PNG to a 1200 × 800 WebP at quality 75 in **669.2 ms** on average across five warm runs. ImageMagick v7.1.2 averaged **835.3 ms** for the same command: Sphene was **1.25× faster** in this controlled run. Both outputs were 4,984 bytes and produced RMSE 0 when compared.

This is one reproducible fixture, not a universal speed claim. Measure representative images and workloads before making capacity or latency commitments.

## Status

`0.1.1` is the current functional release. The interface is intentionally narrow: file paths in and out, with ImageMagick fallback for broader compatibility.

## Installation

```bash
cargo install sphene --version 0.1.1
```

AVIF builds require the native dependencies documented by `libavif-sys`.

## Usage

```bash
sphene input.jpg output.png
sphene convert input.jpg -resize 800x600 output.webp
sphene convert input.jpg -quality 80 output.jpg
sphene convert input.jpg -resize 800x600 -quality 80 output.webp
```

Native operations support JPEG, PNG, WebP, and AVIF. HEIC/HEIF is delegated to ImageMagick. Output format is inferred from the destination extension. `-quality` applies to JPEG, WebP, and AVIF; other formats ignore it. The resize form preserves aspect ratio inside the requested bounding box and uses Lanczos3 filtering.

Unsupported flags, resize modifiers, unknown extensions, and `-` trigger the ImageMagick fallback. Sphene prefers `magick` and falls back to `convert`, passing the original arguments and exit status through. Install ImageMagick if you need that path.

## Limitations

- File paths only. Standard input and output streams are not supported natively.
- Color-profile management, GUI operations, and ImageMagick's full feature set use the fallback and require ImageMagick.
- The native path rejects dimensions at or above 250,000,000 pixels before allocating an image buffer.
- Native input files are limited to 50 MiB before decoding.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md). Release history is in [CHANGELOG.md](CHANGELOG.md).

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
