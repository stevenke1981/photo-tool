# Photo Tool

Rust + egui image viewer and converter with 360-compatible JPEG export.

## Current Features

- Traditional Chinese UI by default, with an English language switch.
- Larger default desktop window for image work.
- Open and preview images in the desktop app.
- Open folders and browse supported images.
- Drag image files or folders into the window to open them.
- Launch the GUI with an image or folder path argument.
- Inspect basic image information.
- Zoom to fit, view at 100%, zoom in, and zoom out.
- Rotate, flip, resize, and crop the current image without overwriting the source.
- Adjust brightness, contrast, saturation, exposure, blur, sharpen, grayscale, and invert.
- Save the current edited/composited result as a new image file.
- Add editable text layers over an image, including per-layer font selection with live font previews.
- Add image/icon layers from files.
- Paste image layers from the clipboard.
- Move layers up/down, hide/show them, delete them, align them, and edit position, size, opacity, text color, stroke, and shadow.
- Show selected text bounds based on the actual rendered text size.
- Drag selected layers directly on the canvas.
- Use Undo / Redo for layer edits.
- Duplicate layers.
- Use Delete to remove a selected layer and arrow keys to nudge it.
- Save and reopen editable `.photo-project` files with the background image and layers intact.
- Export the flattened composition as a new image.
- Convert images between JPEG, PNG, WEBP, BMP, and TIFF.
- Batch convert folders with optional resize-to-fit.
- Export one image as a 2:1 360-compatible JPEG with GPano/XMP metadata.
- Use CLI commands for repeatable smoke tests.

## Run the App

```powershell
cargo run -p photo-tool
```

For the packaged app, run:

```powershell
.\dist\photo-tool.exe
```

## Development

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p photo-tool
```

## CLI

```powershell
cargo run -p photo-cli -- info input.jpg
cargo run -p photo-cli -- convert input.png output.jpg --format jpg --quality 90
cargo run -p photo-cli -- batch input-folder output-folder --format webp --quality 90 --max-width 2048 --max-height 2048
cargo run -p photo-cli -- pano360 input.jpg output_360.jpg --mode pad --width 2048 --quality 92
```

## Release Build

```powershell
.\scripts\build-release.ps1
```

Optional full check-and-build:

```powershell
.\scripts\build-release.ps1 -RunTests
```
