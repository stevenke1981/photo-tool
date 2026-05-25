use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use photo_core::{
    BatchOptions, ConvertOptions, PanoramaMode, PanoramaOptions, SupportedFormat,
    collect_supported_files, convert_image, inspect_image, process_batch, write_panorama_jpeg,
};

#[derive(Debug, Parser)]
#[command(name = "photo-cli")]
#[command(about = "Photo Tool command-line utilities")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Info {
        input: PathBuf,
    },
    Convert {
        input: PathBuf,
        output: PathBuf,
        #[arg(long)]
        format: SupportedFormat,
        #[arg(long, default_value_t = 92)]
        quality: u8,
    },
    Pano360 {
        input: PathBuf,
        output: PathBuf,
        #[arg(long, default_value = "pad")]
        mode: PanoramaMode,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long, default_value_t = 92)]
        quality: u8,
    },
    Batch {
        input_dir: PathBuf,
        output_dir: PathBuf,
        #[arg(long)]
        format: SupportedFormat,
        #[arg(long, default_value_t = 92)]
        quality: u8,
        #[arg(long)]
        max_width: Option<u32>,
        #[arg(long)]
        max_height: Option<u32>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Info { input } => {
            let info = inspect_image(input)?;
            println!(
                "{}: {}x{} {} {} bytes",
                info.path.display(),
                info.width,
                info.height,
                info.format,
                info.file_size
            );
        }
        Command::Convert {
            input,
            output,
            format,
            quality,
        } => {
            let result = convert_image(
                input,
                output,
                ConvertOptions {
                    format,
                    quality,
                    background: [255, 255, 255, 255],
                },
            )?;
            println!(
                "converted {} -> {} ({}x{}, {})",
                result.input.display(),
                result.output.display(),
                result.width,
                result.height,
                result.format
            );
        }
        Command::Pano360 {
            input,
            output,
            mode,
            width,
            quality,
        } => {
            let result = write_panorama_jpeg(
                input,
                output,
                PanoramaOptions {
                    mode,
                    target_width: width,
                    quality,
                    background: [0, 0, 0, 255],
                },
            )?;
            println!(
                "wrote 360-compatible JPEG {} ({}x{}, mode={})",
                result.output.display(),
                result.width,
                result.height,
                result.mode
            );
        }
        Command::Batch {
            input_dir,
            output_dir,
            format,
            quality,
            max_width,
            max_height,
        } => {
            std::fs::create_dir_all(&output_dir)?;
            let files = collect_supported_files(&input_dir);
            let summary = process_batch(
                &files,
                &BatchOptions {
                    output_dir,
                    convert: ConvertOptions {
                        format,
                        quality,
                        background: [255, 255, 255, 255],
                    },
                    max_width,
                    max_height,
                },
            );
            println!(
                "batch complete: {} total, {} succeeded, {} failed",
                summary.total, summary.succeeded, summary.failed
            );
            for result in summary
                .results
                .iter()
                .filter(|result| result.error.is_some())
            {
                println!(
                    "failed {}: {}",
                    result.input.display(),
                    result.error.as_deref().unwrap_or("unknown error")
                );
            }
        }
    }

    Ok(())
}
