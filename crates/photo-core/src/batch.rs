use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{ConvertOptions, ConvertResult, SupportedFormat, convert_image};

#[derive(Debug, Clone)]
pub struct BatchOptions {
    pub output_dir: PathBuf,
    pub convert: ConvertOptions,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<BatchResult>,
}

pub fn collect_supported_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| SupportedFormat::from_path(path).is_some())
        .collect()
}

pub fn process_batch(inputs: &[PathBuf], options: &BatchOptions) -> BatchSummary {
    process_batch_with_progress(inputs, options, |_completed, _total| {})
}

pub fn process_batch_with_progress(
    inputs: &[PathBuf],
    options: &BatchOptions,
    progress: impl Fn(usize, usize) + Sync,
) -> BatchSummary {
    let total = inputs.len();
    let completed = Arc::new(AtomicUsize::new(0));
    let results: Vec<BatchResult> = inputs
        .par_iter()
        .map(|input| {
            let result = process_one(input, options);
            let completed = completed.fetch_add(1, Ordering::Relaxed) + 1;
            progress(completed, total);
            result
        })
        .collect();

    let succeeded = results
        .iter()
        .filter(|result| result.error.is_none())
        .count();
    let failed = results.len().saturating_sub(succeeded);

    BatchSummary {
        total: results.len(),
        succeeded,
        failed,
        results,
    }
}

fn process_one(input: &Path, options: &BatchOptions) -> BatchResult {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("image");
    let output = options
        .output_dir
        .join(format!("{stem}.{}", options.convert.format.extension()));

    let result =
        if let (Some(max_width), Some(max_height)) = (options.max_width, options.max_height) {
            convert_resized_image(input, &output, options.convert, max_width, max_height)
        } else {
            convert_image(input, &output, options.convert)
        };

    match result {
        Ok(ConvertResult { output, .. }) => BatchResult {
            input: input.to_path_buf(),
            output: Some(output),
            error: None,
        },
        Err(error) => BatchResult {
            input: input.to_path_buf(),
            output: None,
            error: Some(error.to_string()),
        },
    }
}

fn convert_resized_image(
    input: &Path,
    output: &Path,
    options: ConvertOptions,
    max_width: u32,
    max_height: u32,
) -> crate::Result<ConvertResult> {
    let image = crate::load_dynamic_image(input)?;
    let resized = crate::resize::resize_to_fit(&image, max_width, max_height);
    let mut result = crate::save_dynamic_image(&resized, output, options)?;
    result.input = input.to_path_buf();
    Ok(result)
}
