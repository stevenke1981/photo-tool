# Photo Tool Plan

## 1. Product Goal

Build a high-performance Windows desktop photo utility with Rust + egui.

The app should support:

- Image viewing and inspection
- Image format conversion
- Batch conversion
- Resize, crop, rotate, and flip
- 360-compatible image export
- Future CLI automation
- Future executable delivery as a standalone Windows app

The first release focuses on reliable local image workflows, not cloud services.

## 2. Technology Decision

Use Rust + egui as the primary stack.

Recommended core stack:

- Rust workspace for clear module boundaries
- `eframe` / `egui` for native desktop UI
- `image` for common image decoding and encoding
- `rayon` for parallel batch processing
- `rfd` for native file/folder dialogs
- `kamadak-exif` or similar crate for EXIF reading
- Custom XMP injection logic for GPano metadata
- `anyhow` / `thiserror` for error handling
- `serde` / `toml` for saved settings

Why Rust + egui:

- Strong performance for large images and batch jobs
- Good fit for a portable Windows executable
- Responsive UI can be kept separate from worker threads
- Easier long-term control over memory, concurrency, and file handling
- egui is fast to build with and suitable for utility-style desktop tools

Tradeoffs:

- More initial engineering than Python
- Some formats and metadata workflows may need extra crates or custom logic
- Advanced 360 stitching is not trivial and should be a later phase

## 3. MVP Scope

### 3.1 Image Viewer

Features:

- Open one image
- Open a folder and browse supported images
- Preview image in the main viewport
- Zoom in, zoom out, fit to window, and show 100 percent
- Pan large images
- Show image information:
  - File name
  - Format
  - Dimensions
  - File size
  - Color type when available

Supported input formats for MVP:

- JPEG
- PNG
- WEBP
- BMP
- TIFF

### 3.2 Format Conversion

Single-image conversion:

- Select output format
- Choose output location
- Set quality for JPEG and WEBP
- Convert transparency safely when output format does not support alpha
- Show success or error state in UI

Supported output formats for MVP:

- JPEG
- PNG
- WEBP
- BMP
- TIFF

### 3.3 Batch Conversion

Batch features:

- Select multiple files
- Select a folder
- Filter by supported extensions
- Convert all selected files to one output format
- Optional resize during export
- Parallel processing with worker threads
- Progress bar
- Per-file result list
- Error report for failed files

The UI must stay responsive during batch work.

### 3.4 Basic Edits

MVP edit operations:

- Resize by width and height
- Preserve aspect ratio toggle
- Rotate 90 degrees left/right
- Flip horizontal/vertical
- Crop to a selected rectangle
- Save the current edited/composited result as a new file
- Brightness adjustment
- Contrast adjustment
- Saturation adjustment
- Exposure adjustment
- Blur
- Sharpen
- Grayscale
- Invert colors

Non-destructive editing is preferred in the UI. The source file should not be overwritten unless the user explicitly chooses overwrite.

### 3.6 Composition and Projects

Implemented next-slice editing features:

- Text layers with font selection, size, color, opacity, stroke, and shadow.
- Font dropdown previews and measured text-layer selection bounds.
- Image/icon layers from files and clipboard paste.
- Layer selection, dragging, alignment, reordering, visibility, duplication, deletion, undo/redo, and keyboard nudging.
- Flattened composition export.
- Editable `.photo-project` save/load so the user can reopen a layered work file later.

Recommended next priorities after this slice:

- Font search/favorites for long Windows font lists.
- Layer locking and alignment guides.
- Export presets for social media sizes.
- Recent files/projects list.

### 3.5 360-Compatible Export

MVP 360 export means creating an image that 360-aware platforms can recognize as an equirectangular panorama.

Important limitation:

- A normal single photo cannot become a true captured 360 panorama.
- MVP creates a 2:1 equirectangular-compatible output and injects GPano/XMP metadata.
- True multi-photo panorama stitching is a later phase.

360 export modes:

- Pad to 2:1:
  - Place the source image on a 2:1 canvas.
  - Fill unused space with a color or blurred background.

- Stretch to 2:1:
  - Resize the full image directly to a 2:1 ratio.
  - Useful for images already close to panorama layout.

- Center crop to 2:1:
  - Crop the center area to 2:1.
  - Useful for wide images.

360 output rules:

- Output as JPEG.
- Width must be twice the height.
- Inject GPano XMP metadata.
- Let user set heading, pitch, and roll later if needed.

## 4. Out of Scope for MVP

These should not block the first release:

- Real panorama stitching from multiple photos
- AI image enhancement
- RAW camera file support
- Full EXIF editor
- Advanced layer effects beyond the current text/image layer editor
- GPU image pipeline
- Plugin system
- Timeline or album management

## 5. Proposed Project Structure

```text
D:\photo-tool
  plan.md
  README.md
  Cargo.toml
  crates/
    photo-tool/
      Cargo.toml
      src/
        main.rs
        app.rs
        state.rs
        ui/
          mod.rs
          viewer.rs
          toolbar.rs
          convert_panel.rs
          batch_panel.rs
          panorama_panel.rs
    photo-core/
      Cargo.toml
      src/
        lib.rs
        error.rs
        formats.rs
        image_io.rs
        convert.rs
        resize.rs
        transform.rs
        batch.rs
        panorama360.rs
        gpano_xmp.rs
    photo-cli/
      Cargo.toml
      src/
        main.rs
  tests/
    fixtures/
```

Workspace crates:

- `photo-core`: image processing, format conversion, 360 export, metadata logic
- `photo-tool`: egui desktop application
- `photo-cli`: optional CLI wrapper for automation and smoke tests

This keeps UI code separate from image-processing logic.

## 6. Architecture

### 6.1 UI Layer

Responsibilities:

- Render main egui interface
- Manage user interactions
- Display image preview
- Send work requests to background workers
- Show progress and errors

The UI should not directly perform heavy image conversion.

### 6.2 Core Layer

Responsibilities:

- Decode and encode images
- Convert formats
- Resize and transform images
- Generate 360-compatible output
- Inject GPano metadata
- Provide stable APIs that can be reused by GUI and CLI

### 6.3 Worker Layer

Responsibilities:

- Run long operations outside the UI frame loop
- Process batch jobs in parallel
- Send progress messages back to UI
- Allow cancellation in a later milestone

### 6.4 CLI Layer

Responsibilities:

- Provide repeatable smoke tests
- Allow future scripting
- Make conversion features usable without GUI

Initial CLI commands can be minimal:

```text
photo-cli convert input.jpg output.webp --quality 90
photo-cli batch input-folder output-folder --format jpg
photo-cli pano360 input.jpg output.jpg --mode pad
```

## 7. UI Plan

Main window layout:

- Top toolbar:
  - Open file
  - Open folder
  - Save as
  - Batch
  - 360 export

- Left panel:
  - Folder image list
  - File metadata

- Center viewport:
  - Image preview
  - Zoom and pan

- Right panel:
  - Conversion settings
  - Resize settings
  - 360 export settings

- Bottom status bar:
  - Current operation
  - Progress
  - Errors

The UI should feel like a practical utility, not a marketing-style app.

## 8. Milestones

### Milestone 1: Rust Workspace and Viewer

Deliverables:

- Rust workspace
- egui window
- Open image dialog
- Decode and preview image
- Basic metadata panel
- `cargo fmt`, `cargo clippy`, and `cargo test` passing

### Milestone 2: Single Image Conversion

Deliverables:

- Format detection
- Convert one image to JPEG, PNG, WEBP, BMP, and TIFF
- Quality option for JPEG and WEBP
- Save-as flow in GUI
- Core conversion tests

### Milestone 3: Batch Conversion

Deliverables:

- Multi-file and folder selection
- Batch queue
- Background worker
- Progress reporting
- Parallel conversion
- Error report

### Milestone 4: Basic Editing

Deliverables:

- Resize
- Aspect-ratio lock
- Rotate
- Flip
- Crop
- Preview updated result before export

### Milestone 5: 360-Compatible Export

Deliverables:

- Pad to 2:1
- Stretch to 2:1
- Center crop to 2:1
- JPEG output
- GPano/XMP metadata injection
- CLI smoke test to verify 2:1 dimensions and GPano metadata marker

### Milestone 6: Packaging

Deliverables:

- Release build
- `dist\photo-tool.exe`
- Optional `dist\photo-cli.exe`
- Basic README
- Windows smoke test

## 9. Validation Gates

Run during development:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

MVP manual checks:

- Open a JPEG and PNG in the GUI.
- Convert JPEG to WEBP.
- Convert PNG with transparency to JPEG and verify background handling.
- Batch convert a folder.
- Export one image as 360-compatible JPEG.
- Confirm 360 output is exactly 2:1.
- Confirm 360 output contains GPano/XMP metadata.
- Confirm the GUI remains responsive during batch conversion.

## 10. Open Product Questions

- Should the first UI language be Traditional Chinese, English, or both?
- Should 360 export default to pad, stretch, or crop?
- Should batch conversion overwrite existing files, auto-rename, or ask each time?
- Should the app prioritize tiny output size or visual quality by default?
- Should CLI be included in MVP or start as an internal smoke-test tool?

## 11. Recommended First Implementation Path

Start with:

1. Create the Rust workspace.
2. Build `photo-core` image loading and metadata structs.
3. Build the egui viewer.
4. Add single-file conversion.
5. Add 360-compatible export.
6. Add batch conversion.
7. Package the Windows executable.

This order gives a visible app early while keeping the core image pipeline reusable.
