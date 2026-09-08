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
            convert_image(&input, &output)?;
        }
    }

    Ok(())
}
