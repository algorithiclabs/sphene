# Changelog

All notable changes to Sphene are documented here.

## [0.1.2] - Unreleased

- Improved ImageMagick fallback discovery across Debian, macOS, and `PATH`.
- Added human-readable CLI errors.
- Added Docker build-context hygiene.

## [0.1.1] - 2026-09-13

- Added library-safe logging for Strangler Fig proxy decisions.
- Initialized CLI logging from `RUST_LOG`.
- Wired `--version` to the crate version.

## [0.1.0] - 2026-09-10

First functional release.

- Native file-to-file conversion for JPEG, PNG, WebP, AVIF, and HEIC/HEIF input where the system codec is available.
- Linear-light RGBA resizing with Lanczos3 filtering.
- JPEG, lossy WebP, and AVIF quality controls.
- 250-megapixel pre-allocation safety limit.
- ImageMagick 6/7 fallback for unsupported syntax and formats.
- File paths only; standard input and output streams remain out of scope.

## [0.1.0-alpha.1] - 2026-06-10

- Initial published placeholder release.
