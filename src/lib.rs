use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpheneError {
    #[error("Failed to read image file from path: {0}")]
    IoError(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("Unknown error occurred during image processing")]
    Unknown,
}

/// Flags the native engine parses itself. Anything else routes to the ImageMagick fallback.
const NATIVE_FLAGS: [&str; 2] = ["-resize", "-quality"];
/// ImageMagick resize modifiers (`!`, `>`, `<`, `^`) that the native resize path doesn't implement.
const RESIZE_MODIFIERS: [char; 4] = ['!', '>', '<', '^'];

/// Strangler Fig gate: scans raw CLI args for syntax the native engine can't handle
/// (STDIN `-`, resize modifiers, or any flag outside `NATIVE_FLAGS`) so parsing can
/// halt before clap ever touches them, per the PRD's <5ms handoff requirement.
pub fn needs_fallback(args: &[String]) -> bool {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-" {
            return true;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            if !NATIVE_FLAGS.contains(&arg.as_str()) {
                return true;
            }
            if args.get(i + 1).is_some_and(|v| v.contains(RESIZE_MODIFIERS)) {
                return true;
            }
            i += 1;
        }
        i += 1;
    }
    false
}

/// Spawns `program` with the exact raw args, inheriting STDOUT/STDERR, and returns its exit code.
pub fn spawn_fallback(program: &str, args: &[String]) -> Result<i32, SpheneError> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .map_err(|e| SpheneError::IoError(e.to_string()))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn native_convert_syntax_does_not_fall_back() {
        assert!(!needs_fallback(&args(&["convert", "in.jpg", "out.png"])));
        assert!(!needs_fallback(&args(&[
            "convert", "in.jpg", "-resize", "800x600", "-quality", "80", "out.png"
        ])));
    }

    #[test]
    fn stdin_flag_falls_back() {
        assert!(needs_fallback(&args(&["convert", "-", "out.png"])));
    }

    #[test]
    fn resize_modifier_falls_back() {
        assert!(needs_fallback(&args(&[
            "convert", "in.jpg", "-resize", "800x600^", "out.png"
        ])));
    }

    #[test]
    fn unknown_flag_falls_back() {
        assert!(needs_fallback(&args(&["convert", "in.jpg", "-polaroid", "out.png"])));
    }

    #[test]
    fn fallback_proxy_pipes_exit_code() {
        // Stands in for `magick`: any program is valid here, only the exit-code
        // passthrough plumbing is under test.
        let code = spawn_fallback("sh", &args(&["-c", "exit 42"])).unwrap();
        assert_eq!(code, 42);
    }
}

/// Core function to handle image resizing logic.
pub fn resize_image(_input: &str, _output: &str, width: u32, height: u32) -> Result<(), SpheneError> {
    println!("Sphene Lib: Initializing resize routine for {}x{}px", width, height);
    // Real pixel processing code will go here in the future
    Ok(())
}

/// Core function to handle format conversion logic.
pub fn convert_image(input: &str, output: &str) -> Result<(), SpheneError> {
    println!("Sphene Lib: Initializing conversion routine from {} to {}", input, output);
    Ok(())
}
