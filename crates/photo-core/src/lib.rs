pub mod batch;
pub mod compose;
pub mod convert;
pub mod error;
pub mod formats;
pub mod gpano_xmp;
pub mod image_io;
pub mod panorama360;
pub mod resize;
pub mod transform;

pub use batch::{
    BatchOptions, BatchResult, BatchSummary, collect_supported_files, process_batch,
    process_batch_with_progress,
};
pub use compose::{
    ComposeDocument, ComposeLayer, ImageLayer, LayerBounds, TextLayer, render_composition,
    text_layer_bounds,
};
pub use convert::{
    ConvertOptions, ConvertResult, convert_image, flatten_alpha, save_dynamic_image,
};
pub use error::{PhotoError, Result};
pub use formats::SupportedFormat;
pub use image_io::{ImageInfo, inspect_image, load_dynamic_image};
pub use panorama360::{
    PanoramaMode, PanoramaOptions, PanoramaResult, make_equirectangular,
    write_panorama_dynamic_jpeg, write_panorama_jpeg,
};
