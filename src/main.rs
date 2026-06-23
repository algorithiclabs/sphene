use clap::{Parser, Subcommand};
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

#[derive(Parser, Debug)]
#[command(name = "sphene")]
#[command(author, version, about = "Modern image processing CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Resize an input image specifying explicit dimensions
    Resize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long)]
        width: u32,
        #[arg(short, long)]
        height: u32,
    },
    /// Convert an image file format to another container layout
    Convert {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<(), SpheneError> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Resize { input, output, width, height }) => {
            println!("Sphene: Resizing '{}' to '{}' ({}x{}px)...", input, output, width, height);
            println!("Status: Core execution pipeline is initializing.");
        }
        Some(Commands::Convert { input, output }) => {
            println!("Sphene: Converting '{}' directly into '{}'...", input, output);
            println!("Status: Format conversion sub-pipeline is initializing.");
        }
        None => {
            println!("Sphene Image Processor Alpha. Run with '--help' to see usage instructions.");
        }
    }

    Ok(())
}
