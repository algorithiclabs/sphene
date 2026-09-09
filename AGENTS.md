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
- Dependencies: `image` (core), `clap` (cli), `libavif-sys` (AVIF), `libheif-rs` (HEIC). 
- Avoid STDIN pipes: File path to file path only for v0.1.

## Agent Execution (Claude Code)
- Run `cargo clippy -- -D warnings` after every logic change. Do not proceed if it fails.
- Trace first: Check `clap` struct alignment before modifying fallback proxy logic.
