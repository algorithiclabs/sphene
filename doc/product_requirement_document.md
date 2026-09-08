### 1. The Linking Reality (AVIF/HEIC vs. 50MB Bloat)
[Thing] C-Bindings and OS Dependencies. 
[Action] Statically link AVIF. Dynamically link HEIC (or drop). 
[Reason] 
- **AVIF (Patent-Free):** We can statically link `libavif` into our Rust binary. The binary grows, but the user installs nothing. It works standalone.
- **HEIC (Patented):** Statically linking HEIC means *you* distribute the patented HEVC math in your binary (legal risk). Dynamically linking it means the binary stays small, but the user *must* run `apt install libheif-dev` on Debian. 
[Next Step] Statically link AVIF. Dynamically link HEIC. Restrict v0.1 OS target to Debian/glibc to simplify the C-toolchain build process.

### 2. Deep Scan: Final Blindspots
- **Blindspot 1: CLI Piped Streams (`STDIN` / `-`)**. ImageMagick often reads from STDIN (`cat img.jpg | magick - out.png`). Handling a byte stream, failing, and proxying that half-consumed stream to ImageMagick is a nightmare. [Fix] Explicitly ban STDIN (`-`) for v0.1. Require physical file paths (`sphene convert in.jpg out.jpg`).
- **Blindspot 2: Color Space Darkening**. Resizing in standard RGB causes dark artifacts. [Fix] Engine must convert to Linear RGB before resizing, then back to sRGB.
- **Blindspot 3: The "100% Safe" Lie**. [Fix] Update PRD. You cannot claim 100% memory safety if you link C libraries for AVIF/HEIC. It is "Memory-safe core; isolated C-codecs."

***

### SPHENE: Product Requirements Document (Final v0.1)

**Status:** MVP (v0.1) Scoping
**Target OS:** Debian-based Linux (glibc). macOS/Windows deferred.
**Core Constraint:** Memory-safe Rust core, explicit limits on C-bindings.

#### 1. The North Star (Vision)
Sphene is a high-speed, memory-safe cloud media processing engine. It serves as a drop-in replacement for the top 95% of ImageMagick web workloads. It halts OOM crashes via strict bounds checking and reduces compute time for modern formats.

#### 2. Architecture & Tech Stack
- **Core Logic & Routing:** Pure Rust (`image` crate, `clap` for CLI).
- **Decoders/Encoders:** Rust native for JPEG, PNG, WEBP.
- **C-Bindings:** `libavif` (statically linked for AVIF), `libheif` (dynamically linked, requires host OS installation).
- **Execution:** File-path to File-path only. No `STDIN/STDOUT` piping for v0.1.

#### 3. Product Mechanics & User Experience
1. **Silent Success:** UNIX standard. Output file, exit code `0`.
2. **DOS Protection:** Engine must calculate `width * height`. If `> MAX_PIXELS` (e.g., 250 megapixels), abort before allocation.
3. **The Proxy Fallback:** Any unsupported format, flag, or syntax triggers a sub-process handoff to system `magick`. 

#### 4. MVP (v0.1) Scope
**A. Supported Formats**
*   **Read:** `JPEG`, `PNG`, `WEBP`, `AVIF`, `HEIC` (if system lib present).
*   **Write:** `JPEG`, `PNG`, `WEBP`, `AVIF`.

**B. Supported CLI Syntax (Native Execution)**
*   **Format Conversion:** `sphene convert <input> <output>`
*   **Resizing (Standard):** `sphene convert <input> -resize <Width>x<Height> <output>` (Must preserve aspect ratio. Must resize in Linear RGB space).
*   **Quality:** `sphene convert <input> -quality <1-100> <output>`
*   **Chaining:** `sphene convert <input> -resize <WxH> -quality <Q> <output>`

**C. The Strangler Fig Proxy (Execution Rules)**
If arguments contain STDIN flags (`-`), advanced resize modifiers (`!`, `>`, `<`, `^`), complex filters (`-polaroid`), or unknown extensions:
1. Halt Rust parsing immediately (< 5ms).
2. Spawn system `magick` passing the exact raw `std::env::args()`.
3. Pipe STDOUT/STDERR back to user. Exit with `magick`'s exit code.

#### 5. Out of Scope (v0.1)
*   Static linking of HEIC.
*   Cross-compilation to Alpine (musl), Windows, or macOS.
*   STDIN/STDOUT data piping.
*   Custom GUI, web interfaces, or direct S3 streaming.

***

No outstanding questions. Architecture is verified, constrained, and executable. 
To start: add `image`, `clap`, and `libavif-sys` to `Cargo.toml`.
