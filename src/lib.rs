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
