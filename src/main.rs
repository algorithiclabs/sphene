use clap::{Parser, Subcommand};
use sphene::{convert_image, needs_fallback, spawn_fallback, SpheneError};

#[derive(Parser, Debug)]
#[command(name = "sphene", author, version, about = "Memory-safe image processing CLI with ImageMagick fallback")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert an image, optionally resizing and/or adjusting quality: `convert <input> [-resize WxH] [-quality N] <output>`
    Convert {
        input: String,
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        rest: Vec<String>,
    },
}

/// Parses a `WxH` resize argument, e.g. `"800x600"`.
fn parse_resize(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn main() -> Result<(), SpheneError> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    if needs_fallback(&raw_args) {
        let code = spawn_fallback("magick", &raw_args)?;
        std::process::exit(code);
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::Convert { input, mut rest } => {
            let output = rest.pop().ok_or_else(|| {
                SpheneError::UnsupportedFormat("missing output path".to_string())
            })?;

            let mut resize = None;
            let mut quality = None;
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "-resize" => {
                        let val = rest.get(i + 1).ok_or_else(|| {
                            SpheneError::UnsupportedFormat("-resize requires a value".to_string())
                        })?;
                        resize = Some(parse_resize(val).ok_or_else(|| {
                            SpheneError::UnsupportedFormat(format!("invalid -resize value: {val}"))
                        })?);
                        i += 2;
                    }
                    "-quality" => {
                        let val = rest.get(i + 1).ok_or_else(|| {
                            SpheneError::UnsupportedFormat("-quality requires a value".to_string())
                        })?;
                        quality = Some(val.parse::<u8>().map_err(|_| {
                            SpheneError::UnsupportedFormat(format!("invalid -quality value: {val}"))
                        })?);
                        i += 2;
                    }
                    other => {
                        return Err(SpheneError::UnsupportedFormat(format!(
                            "unexpected argument: {other}"
                        )))
                    }
                }
            }

            convert_image(&input, &output, resize, quality)?;
        }
    }

    Ok(())
}
