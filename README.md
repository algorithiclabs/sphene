# Sphene

Memory-safe, ImageMagick-compatible image conversion for Debian-based Linux systems using glibc.

Sphene handles common file-to-file conversions natively and delegates syntax it does not support to an installed ImageMagick 6 or 7 executable. The native pipeline keeps images in RGBA form, resizes in linear RGB, and applies the 250-megapixel allocation guard before decoding pixel data.

## Status

`0.1.0` is the first functional release. The interface is intentionally narrow: file paths in and out, with ImageMagick fallback for broader compatibility.

## Installation

```bash
cargo install sphene --version 0.1.0
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
