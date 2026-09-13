# Sphene launch-readiness review

Decision: do not present Sphene as an ImageMagick replacement yet. Present it as a guarded native fast path and safety boundary for image workers, with an ImageMagick compatibility escape hatch.

The product has a real wedge: native Sphene reads dimensions before decoding and rejects images at or above 250 million pixels. The product does not yet have evidence for broad ImageMagick compatibility, production throughput, concurrency behavior, or protection when work is delegated to ImageMagick.

## What is proven

- A 107 KiB PNG declaring 30,000 × 30,000 pixels caused the local ImageMagick run to use about 15.1 GB resident memory and complete in 16.36 seconds.
- Sphene rejected the same input before decode in 0.00 seconds with about 2.7 MB resident memory.
- On one controlled 6000 × 4000 gradient-to-WebP fixture, Sphene averaged 669.2 ms versus ImageMagick's 835.3 ms across five warm runs. Both outputs were 4,984 bytes and RMSE was 0.
- Fallback reason logging works through `RUST_LOG`.
- Fallback discovery checks OS-known paths and `PATH`; the previous macOS Homebrew failure is fixed.
- The Debian Bookworm container builds with a 5.78 MB build context, runs as the non-root `sphene` user, rejects the bomb, and completes native plus fallback smoke tests.

Evidence and commands: [benchmark report](benchmark-report.md).

These are fixture-specific observations. They are not universal speed, memory, or outage-prevention claims.

## Product positioning

### Use this pitch

> Sphene gives image workers a bounded native fast path for common conversions. It rejects oversized inputs before pixel allocation, keeps the existing ImageMagick-shaped CLI, and delegates unsupported syntax when compatibility matters more than native execution.

### Do not use these claims

- “Drop-in replacement for ImageMagick.” Native coverage is narrow and file-to-file only.
- “Stops all decompression bombs.” Delegated ImageMagick work has its own resource and security behavior.
- “Prevents production outages.” The local stress test showed a dangerous allocation, not an actual cloud-worker OOM kill.
- “1.25× faster.” One synthetic fixture supports “1.25× faster in this recorded run,” not a general benchmark claim.
- “Top 95% of workloads.” No workload corpus supports that number.

The buyer is not replacing ten years of ImageMagick immediately. The credible adoption path is: put Sphene in front of existing workers, get safe rejection and native execution for the common subset, retain ImageMagick for the long tail, then expand coverage from measured demand.

## Must-fix before public launch

### 1. Keep testing the target, not only a Mac

Run the release binary in the actual Debian/glibc container with ImageMagick 6 and 7. Record:

- native conversion success;
- `magick` fallback;
- `convert` fallback;
- missing executable behavior;
- exit-code forwarding;
- resource use under the container memory limit.

The Debian Bookworm smoke test now passes. Keep it in CI; macOS path support is useful for development, not proof of production support.

### 2. Make fallback safety explicit

Sphene's native guard does not protect commands delegated to ImageMagick. ImageMagick documents that its open default policy should be tightened for untrusted web input, including resource limits, allowed coders, path restrictions, delegate restrictions, and sandboxing. See [ImageMagick's security policy](https://imagemagick.org/security-policy/) and [resource limits](https://imagemagick.org/command-line-options/).

The deployment recipe must set a memory limit, disk limit, time limit, thread limit, private temporary directory, allowed formats, and disabled delegates. Treat fallback as a compatibility boundary, not a security boundary.

### 3. Improve error UX

The CLI now prints the stable human message and exits nonzero instead of exposing Rust's debug enum. Keep this covered by an integration test.

### 4. Build a representative corpus

The current benchmark uses a synthetic gradient. Add fixtures for:

- camera JPEGs;
- transparent PNGs;
- animated or multi-frame inputs;
- high-bit-depth images;
- AVIF and WebP;
- malformed/truncated files;
- EXIF orientation and color profiles;
- oversized dimensions with tiny compressed payloads.

Report median, p95, peak RSS, output dimensions, output size, and quality/error metrics. Pin CPU, OS, ImageMagick version, Sphene commit, command, and fixture hashes.

### 5. Add security and supply-chain gates

- Enable GitHub private vulnerability reporting. The repository currently reports security analysis features disabled.
- Run `cargo audit` and a dependency/license policy check in CI; neither tool is installed in this workspace.
- Review the C codec boundaries (`libavif-sys`, system ImageMagick/libheif) and document supported versions.
- Fuzz dimension parsing, format sniffing, native decoders, and fallback argument forwarding.
- Run package inspection and `cargo publish --dry-run` from a clean tree. Cargo recommends dry-run verification before publishing and treats published versions as permanent archives ([Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)).

### 6. Close legal and claim hygiene

- Keep the MIT OR Apache-2.0 licensing and dependency notices accurate.
- Inventory transitive dependency licenses before distributing binaries or containers.
- Avoid implying ImageMagick endorsement or ownership; use the name only to describe compatibility.
- Keep benchmark claims tied to dated, reproducible evidence.
- If a hosted service is added later, publish retention, deletion, regional processing, abuse handling, and subprocess-isolation commitments. The current CLI does not receive or retain user uploads.

This is product and engineering guidance, not legal advice.

## Community launch rules

### Hacker News

Use Show HN only when the repository and demo are ready for hostile technical scrutiny. Lead with the measured engineering problem, not a slogan:

> Show HN: Sphene — a bounded native image conversion path for ImageMagick workers

Show the commands, environment, raw measurements, known limitations, and why the fallback exists. Do not claim a universal replacement or hide that ImageMagick remains in the path. Hacker News asks users not to use the site primarily for promotion, discourages marketing/PR language, and expects original sources ([HN guidelines](https://news.ycombinator.com/newsguidelines.html), [Show HN guidance](https://news.ycombinator.com/yli.html)).

### Reddit

Do not cross-post identical promotional copy into unrelated subreddits. Participate in the communities first, follow each moderator's rules, disclose affiliation, answer technical criticism, and publish the benchmark methodology. Reddit's platform rules require authentic participation and prohibit spam/content manipulation ([Reddit Rules](https://redditinc.com/policies/reddit-rules)).

## Go / no-go

### No-go today

- No representative image corpus.
- No dependency audit result.
- Fallback security policy is deployment-dependent and undocumented as an operational artifact.
- Public README, benchmark report, and launch report changes are not yet committed or released.

### Go after these gates

- Debian 6/7 fallback matrix passes. Bookworm/ImageMagick 6 is currently verified; ImageMagick 7 remains open.
- Native and fallback security limits are documented and tested.
- Error output is human-readable and covered by an integration test.
- Corpus benchmark is reproducible and honest about variance.
- Private vulnerability reporting and CI security checks are enabled.
- README, crates.io package, GitHub release, and benchmark report agree.
