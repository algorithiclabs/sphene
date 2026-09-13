# Sphene benchmark record

This document records measured results. It is evidence, not a universal performance or safety guarantee.

## What is actually differentiated

Sphene's strongest defensible claim is bounded native processing: it reads image dimensions before decoding and rejects images at or above 250,000,000 pixels. Common conversions then use the native Rust path. Unsupported syntax is delegated to ImageMagick.

Sphene is not currently a full ImageMagick replacement. Native support is narrower, input/output is file-only, native input is limited to 50 MiB, and fallback safety inherits ImageMagick behavior.

## Test environment

- Host: Apple Silicon arm64, macOS 26.6.2
- ImageMagick: 7.1.2-31 Q16-HDRI aarch64
- Sphene: v0.1.1 release binary
- Date recorded: 2026-09-13

The ImageMagick comparison ran locally. The production target is Debian-based Linux with glibc; repeat the same commands on the deployment image before making capacity claims.

## Test 1: oversized-image guard

Fixture generation:

```bash
mkdir stress-test
cd stress-test
magick -size 30000x30000 canvas:red bomb.png
identify bomb.png
```

Observed fixture: 107 KiB PNG declaring 30,000 × 30,000 pixels, or 900,000,000 pixels.

ImageMagick command:

```bash
/usr/bin/time -l magick convert bomb.png -resize 800x800 out_im.webp
```

Observed result:

- Completed in 16.36 seconds.
- Peak resident memory: 15,096,135,680 bytes, approximately 15.1 GB.
- Produced an 800 × 800 WebP.

Sphene command:

```bash
/usr/bin/time -l ../target/release/sphene convert bomb.png -resize 800x800 out_sphene.webp
```

Observed result:

- Returned immediately in 0.00 seconds with exit status 1.
- Peak resident memory: 2,785,280 bytes, approximately 2.7 MB.
- Returned `DimensionsTooLarge(30000, 30000)`.
- Did not decode the image or create an output.

Interpretation: a memory-constrained worker could be OOM-terminated by the ImageMagick allocation. Sphene converts that input into a bounded rejection. The local ImageMagick process itself completed; this test did not produce an actual OOM kill.

## Test 2: common resize and WebP conversion

Fixture:

```bash
magick -size 6000x4000 gradient:'#153b6d-#f2b84b' input.png
```

Command under test:

```bash
tool input.png -resize 1200x1200 -quality 75 output.webp
```

Replace `tool` with either `magick` or the Sphene release binary. Five warm runs were measured with:

```bash
hyperfine --warmup 2 --runs 5 \
  'magick input.png -resize 1200x1200 -quality 75 im-bench.webp' \
  '../target/release/sphene input.png -resize 1200x1200 -quality 75 sphene-bench.webp'
```

Observed results:

| Tool | Mean | Output | Comparison |
| --- | ---: | ---: | ---: |
| ImageMagick 7.1.2 | 835.3 ms | 4,984 bytes | baseline |
| Sphene v0.1.1 | 669.2 ms | 4,984 bytes | 1.25× faster |

Both outputs were 1,200 × 800 WebP files. ImageMagick `compare -metric RMSE` reported `0 (0)` for this fixture.

Interpretation: Sphene was faster on this controlled gradient. This does not establish a general speed advantage. A representative corpus of photographic, transparent, animated, and high-bit-depth images is required for that claim.

## Test 3: Debian container smoke test

The release image was built from `Dockerfile` using Debian Bookworm and ImageMagick 6. The build context was 5.78 MB after adding `.dockerignore`; the final process runs as the non-root `sphene` user.

```bash
docker build --tag sphene-launch-check .
docker run --rm sphene-launch-check --version
```

Observed version: `sphene 0.1.1`.

The same container:

- rejected `bomb.png` before decode with exit status 1 and the human-readable safety error;
- completed a native resize conversion;
- completed a resize-modifier fallback through Debian's ImageMagick 6 installation.

This validates the target container path. It does not validate ImageMagick 7 or production memory limits.

## Test 4: logging and fallback probe

```bash
RUST_LOG=warn sphene input.png -resize '1200x1200^' fallback.webp
```

Observed native decision log:

```text
WARN sphene: Strangler Fig proxy triggered: resize modifier is unsupported
```

On the macOS test host, fallback execution then failed because the binary searches only `/usr/bin/magick` and `/usr/bin/convert`, while Homebrew installed them under `/opt/homebrew/bin`. This is a portability defect in the current implementation, not evidence that Debian fallback works. Repeat this test on Debian before release claims.

## Claims allowed by this record

- Sphene can reject a 900 MP input before decoding with very low observed memory use.
- Native common-path conversion completed faster than ImageMagick on the recorded fixture.
- Native and ImageMagick outputs matched on the recorded gradient comparison.
- Fallback decisions are observable through `RUST_LOG`.

## Claims not supported yet

- Universal speed or memory superiority.
- “Drop-in replacement” for ImageMagick's broad feature set.
- Protection when unsupported operations are delegated to ImageMagick.
- Production outage prevention without a Debian/container test and worker memory limit.
- Throughput, concurrency, or tail-latency improvements.
