# Contributing to Sphene

## Development

Sphene targets Debian-based Linux with glibc. HEIC support requires the system `libheif` development package; AVIF builds require the native build dependencies used by `libavif-sys`.

Run the required checks before opening a pull request:

```bash
cargo fmt -- --check
cargo check
cargo test
cargo clippy -- -D warnings
```

Keep changes focused. Add a regression test for non-trivial behavior. Do not commit generated files, local image fixtures, or credentials.

## Pull requests

Open changes against `main`. Explain user-visible behavior, compatibility impact, and test coverage. Release changes require an entry in `CHANGELOG.md`.

## License

Contributions are accepted under the repository's MIT OR Apache-2.0 license.
