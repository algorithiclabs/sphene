# Sphene Architecture & Rust Guidelines
- PRD Reference: ImageMagick drop-in replacement. Memory-safe core, proxy fallback.
- Target OS: Debian-based Linux (glibc).

## Build & Test Commands
- Check: `cargo check`
- Lint: `cargo clippy -- -D warnings` (Zero warnings allowed before committing).
- Test: `cargo test`
- Build: `cargo build --release`

## Rust / Code Constraints
- CLI Parsing: Use `clap`. Capture trailing args to forward to IM fallback.
- DOS Protection: Enforce `width * height < 250_000_000` (250 megapixels) BEFORE any buffer allocation. 
- Subprocess: Use `std::process::Command` for ImageMagick fallback. Pipe STDOUT/STDERR transparently.
- Color Space: Resizing must occur in Linear RGB space to prevent dark artifacts, convert back to sRGB after.
  - Pipeline is RGBA throughout (`resize_in_linear_space` in `src/lib.rs`): convert via `to_rgba32f`, apply
    the sRGB<->linear transfer function to the R, G, B channels only. The A channel is never gamma-converted
    (it's a coverage value, not a light intensity) — it's carried through untouched, but is still resampled
    by the Lanczos3 filter alongside the color channels.
  - JPEG has no alpha channel: `encode_output` flattens RGBA -> RGB only at the final encode step for that
    format. Every other supported format keeps alpha end-to-end.
- Quality-aware encoding (`encode_output` in `src/lib.rs`): branches on the output `ImageFormat`.
  - JPEG: `-quality` drives `image::codecs::jpeg::JpegEncoder::new_with_quality`; no flag falls back to
    `image`'s default encoder settings.
  - WEBP: real lossy encoding via the `webp` crate's libwebp binding (`webp::Encoder::from_image(&DynamicImage).encode(quality)`),
    not `image`'s own WebP encoder (which is lossless-only). `-quality` maps 0-100 -> the same range as
    libwebp's quality; no flag defaults to `DEFAULT_WEBP_QUALITY` (75.0).
  - All other formats (PNG, etc.): `image`'s default `save_with_format`, quality ignored.
- Dependencies: `image` (core), `clap` (cli), `libavif-sys` (AVIF), `libheif-rs` (HEIC), `webp` (lossy WEBP encode).
- Avoid STDIN pipes: File path to file path only for v0.1.

## Agent Execution (Claude Code)
- Run `cargo clippy -- -D warnings` after every logic change. Do not proceed if it fails.
- Trace first: Check `clap` struct alignment before modifying fallback proxy logic.
