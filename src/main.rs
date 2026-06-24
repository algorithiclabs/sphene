use clap::{Parser, Subcommand};
use sphene::{convert_image, resize_image, SpheneError};

#[derive(Parser, Debug)]
#[command(name = "sphene", author, version, about = "Modern image processing CLI tool", disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Resize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long)]
        width: u32,
        #[arg(short, long)]
        height: u32,
        #[clap(long, action = clap::ArgAction::HelpLong)]
        help: Option<bool>,
    },
    /// Convert an image file format to another container layout
    Convert {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[clap(long, action = clap::ArgAction::HelpLong)]
        help: Option<bool>,
    },
}

fn main() -> Result<(), SpheneError> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Resize { input, output, width, height, help: _ }) => {
            resize_image(input, output, *width, *height)?;
        }
        Some(Commands::Convert { input, output, help: _ }) => {
            convert_image(input, output)?;
        }
        None => {
            println!("Sphene Image Processor. Use --help for usage instructions.");
        }
    }

    Ok(())
}
