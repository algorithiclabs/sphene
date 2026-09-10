# sphene

Memory-safe-core ImageMagick-compatible image conversion for Debian-based Linux (glibc). Native operations keep RGBA data through the pipeline and resize RGB in linear light.

## Installation

```bash
cargo install sphene --version 0.1.0-alpha.1
```

HEIC support requires the system `libheif` library.

## Usage

Sphene accepts ImageMagick v6 and v7 conversion forms:

```bash
magick input.jpg output.webp
convert input.jpg output.webp
magick convert input.jpg -resize 800x600 -quality 80 output.webp
```

Native support covers JPEG, PNG, WebP, AVIF, HEIC, and HEIF file paths, with `-resize WxH` and `-quality N`. Dimensions must remain below 250 megapixels before allocation.

Unsupported flags, extensions, standard-input syntax, and ImageMagick resize modifiers are proxied to system ImageMagick. The proxy prefers `magick` and uses `convert` when `magick` is unavailable. It preserves ImageMagick's output streams and exit code.

## Limits

File-path input and output only. STDIN/STDOUT conversion, color-profile management, GUI, and non-Debian/glibc targets are outside v0.1.
