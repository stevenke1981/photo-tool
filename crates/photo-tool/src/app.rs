use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use base64::{Engine as _, engine::general_purpose};
use eframe::egui;
use egui::{
    Color32, ColorImage, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2,
};
use image::DynamicImage;
use photo_core::{
    BatchOptions, BatchSummary, ComposeDocument, ComposeLayer, ConvertOptions, ImageInfo,
    ImageLayer, PanoramaMode, PanoramaOptions, SupportedFormat, TextLayer, collect_supported_files,
    inspect_image, load_dynamic_image, process_batch_with_progress, render_composition,
    save_dynamic_image, text_layer_bounds, write_panorama_dynamic_jpeg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    ZhTw,
    En,
}

impl Language {
    fn name(self) -> &'static str {
        match self {
            Self::ZhTw => "繁體中文",
            Self::En => "English",
        }
    }

    fn t(self, key: TextKey) -> &'static str {
        match self {
            Self::ZhTw => key.zh_tw(),
            Self::En => key.en(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TextKey {
    Actions,
    ApplyCrop,
    ApplyResize,
    ApplyBlur,
    ApplyBrightness,
    ApplyContrast,
    ApplyExposure,
    ApplySaturation,
    ApplySharpen,
    Adjustments,
    Batch,
    BatchRunning,
    BrowseHint,
    Convert,
    Compose,
    CropRectangle,
    Current,
    Edit,
    Export360,
    ExportComposite,
    File,
    Fit,
    FlipH,
    FlipV,
    Folder,
    Format,
    Image,
    AddImage,
    AddText,
    Font,
    Project,
    SaveProject,
    OpenProject,
    Images,
    InputFolder,
    LastBatch,
    Language,
    Layers,
    LayerName,
    MaxH,
    MaxW,
    NoImage,
    NoLayer,
    OpenFolder,
    OpenImage,
    OutputFolder,
    PasteImage,
    PreserveAspect,
    Previous,
    Quality,
    Reload,
    ResetEdits,
    ResizeToFit,
    RotateLeft,
    RotateRight,
    RunBatch,
    SaveAsNewFile,
    SaveConverted,
    Grayscale,
    Invert,
    Brightness,
    Contrast,
    Saturation,
    Exposure,
    Blur,
    Sharpen,
    Threshold,
    SelectedLayer,
    Source,
    Status,
    TextColor,
    TextContent,
    FontSize,
    Opacity,
    Position,
    Size,
    Stroke,
    Shadow,
    Align,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignTop,
    AlignMiddle,
    AlignBottom,
    Visible,
    DeleteLayer,
    DuplicateLayer,
    Undo,
    Redo,
    KeyboardHint,
    MoveUp,
    MoveDown,
    ZoomIn,
    ZoomOut,
}

impl TextKey {
    fn zh_tw(self) -> &'static str {
        match self {
            Self::Actions => "操作",
            Self::ApplyCrop => "套用裁切",
            Self::ApplyResize => "套用尺寸",
            Self::ApplyBlur => "套用模糊",
            Self::ApplyBrightness => "套用亮度",
            Self::ApplyContrast => "套用對比",
            Self::ApplyExposure => "套用曝光",
            Self::ApplySaturation => "套用飽和度",
            Self::ApplySharpen => "套用銳利化",
            Self::Adjustments => "色彩 / 濾鏡",
            Self::Batch => "批次轉檔",
            Self::BatchRunning => "批次轉檔執行中。",
            Self::BrowseHint => "開啟圖片、資料夾，或直接把圖檔拖放到視窗。",
            Self::Convert => "格式轉換",
            Self::Compose => "合成 / 圖層",
            Self::CropRectangle => "裁切範圍",
            Self::Current => "目前",
            Self::Edit => "編輯",
            Self::Export360 => "匯出 360 JPEG",
            Self::ExportComposite => "匯出合成圖",
            Self::File => "檔案",
            Self::Fit => "符合視窗",
            Self::FlipH => "水平翻轉",
            Self::FlipV => "垂直翻轉",
            Self::Folder => "資料夾",
            Self::Format => "格式",
            Self::Image => "圖片",
            Self::AddImage => "加入圖片 / 圖示",
            Self::AddText => "新增文字",
            Self::Font => "字型",
            Self::Project => "專案",
            Self::SaveProject => "儲存專案",
            Self::OpenProject => "開啟專案",
            Self::Images => "圖片數",
            Self::InputFolder => "輸入資料夾",
            Self::LastBatch => "上次批次",
            Self::Language => "語言",
            Self::Layers => "圖層",
            Self::LayerName => "圖層名稱",
            Self::MaxH => "高上限 ",
            Self::MaxW => "寬上限 ",
            Self::NoImage => "尚未載入圖片。",
            Self::NoLayer => "尚未選取圖層。",
            Self::OpenFolder => "開啟資料夾",
            Self::OpenImage => "開啟圖片",
            Self::OutputFolder => "輸出資料夾",
            Self::PasteImage => "貼上圖片",
            Self::PreserveAspect => "維持比例",
            Self::Previous => "上一張",
            Self::Quality => "品質",
            Self::Reload => "重新載入",
            Self::ResetEdits => "重設編輯",
            Self::ResizeToFit => "縮放到範圍內",
            Self::RotateLeft => "向左旋轉",
            Self::RotateRight => "向右旋轉",
            Self::RunBatch => "開始批次轉檔",
            Self::SaveAsNewFile => "另存新檔",
            Self::SaveConverted => "另存轉檔",
            Self::Grayscale => "灰階",
            Self::Invert => "反相",
            Self::Brightness => "亮度",
            Self::Contrast => "對比",
            Self::Saturation => "飽和度",
            Self::Exposure => "曝光",
            Self::Blur => "模糊",
            Self::Sharpen => "銳利化",
            Self::Threshold => "門檻",
            Self::SelectedLayer => "選取圖層",
            Self::Source => "來源",
            Self::Status => "狀態",
            Self::TextColor => "文字顏色",
            Self::TextContent => "文字內容",
            Self::FontSize => "字級",
            Self::Opacity => "透明度",
            Self::Position => "位置",
            Self::Size => "尺寸",
            Self::Stroke => "描邊",
            Self::Shadow => "陰影",
            Self::Align => "對齊",
            Self::AlignLeft => "靠左",
            Self::AlignCenter => "水平置中",
            Self::AlignRight => "靠右",
            Self::AlignTop => "靠上",
            Self::AlignMiddle => "垂直置中",
            Self::AlignBottom => "靠下",
            Self::Visible => "顯示",
            Self::DeleteLayer => "刪除圖層",
            Self::DuplicateLayer => "複製圖層",
            Self::Undo => "復原",
            Self::Redo => "重做",
            Self::KeyboardHint => "提示：可在畫布拖曳圖層；Delete 刪除；方向鍵微調。",
            Self::MoveUp => "上移",
            Self::MoveDown => "下移",
            Self::ZoomIn => "放大",
            Self::ZoomOut => "縮小",
        }
    }

    fn en(self) -> &'static str {
        match self {
            Self::Actions => "Actions",
            Self::ApplyCrop => "Apply Crop",
            Self::ApplyResize => "Apply Resize",
            Self::ApplyBlur => "Apply Blur",
            Self::ApplyBrightness => "Apply Brightness",
            Self::ApplyContrast => "Apply Contrast",
            Self::ApplyExposure => "Apply Exposure",
            Self::ApplySaturation => "Apply Saturation",
            Self::ApplySharpen => "Apply Sharpen",
            Self::Adjustments => "Color / Filters",
            Self::Batch => "Batch",
            Self::BatchRunning => "Batch conversion is running.",
            Self::BrowseHint => "Open an image, open a folder, or drag files into this window.",
            Self::Convert => "Convert",
            Self::Compose => "Compose / Layers",
            Self::CropRectangle => "Crop rectangle",
            Self::Current => "Current",
            Self::Edit => "Edit",
            Self::Export360 => "Export 360 JPEG",
            Self::ExportComposite => "Export Composite",
            Self::File => "File",
            Self::Fit => "Fit",
            Self::FlipH => "Flip H",
            Self::FlipV => "Flip V",
            Self::Folder => "Folder",
            Self::Format => "Format",
            Self::Image => "Image",
            Self::AddImage => "Add Image / Icon",
            Self::AddText => "Add Text",
            Self::Font => "Font",
            Self::Project => "Project",
            Self::SaveProject => "Save Project",
            Self::OpenProject => "Open Project",
            Self::Images => "Images",
            Self::InputFolder => "Input Folder",
            Self::LastBatch => "Last batch",
            Self::Language => "Language",
            Self::Layers => "Layers",
            Self::LayerName => "Layer name",
            Self::MaxH => "Max H ",
            Self::MaxW => "Max W ",
            Self::NoImage => "No image loaded.",
            Self::NoLayer => "No layer selected.",
            Self::OpenFolder => "Open Folder",
            Self::OpenImage => "Open Image",
            Self::OutputFolder => "Output Folder",
            Self::PasteImage => "Paste Image",
            Self::PreserveAspect => "Preserve aspect",
            Self::Previous => "Previous",
            Self::Quality => "Quality",
            Self::Reload => "Reload",
            Self::ResetEdits => "Reset Edits",
            Self::ResizeToFit => "Resize to fit",
            Self::RotateLeft => "Rotate Left",
            Self::RotateRight => "Rotate Right",
            Self::RunBatch => "Run Batch Convert",
            Self::SaveAsNewFile => "Save As",
            Self::SaveConverted => "Save Converted",
            Self::Grayscale => "Grayscale",
            Self::Invert => "Invert",
            Self::Brightness => "Brightness",
            Self::Contrast => "Contrast",
            Self::Saturation => "Saturation",
            Self::Exposure => "Exposure",
            Self::Blur => "Blur",
            Self::Sharpen => "Sharpen",
            Self::Threshold => "Threshold",
            Self::SelectedLayer => "Selected layer",
            Self::Source => "Source",
            Self::Status => "Status",
            Self::TextColor => "Text color",
            Self::TextContent => "Text",
            Self::FontSize => "Font size",
            Self::Opacity => "Opacity",
            Self::Position => "Position",
            Self::Size => "Size",
            Self::Stroke => "Stroke",
            Self::Shadow => "Shadow",
            Self::Align => "Align",
            Self::AlignLeft => "Left",
            Self::AlignCenter => "Center",
            Self::AlignRight => "Right",
            Self::AlignTop => "Top",
            Self::AlignMiddle => "Middle",
            Self::AlignBottom => "Bottom",
            Self::Visible => "Visible",
            Self::DeleteLayer => "Delete Layer",
            Self::DuplicateLayer => "Duplicate Layer",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::KeyboardHint => "Tip: drag layers on canvas; Delete removes; arrow keys nudge.",
            Self::MoveUp => "Move Up",
            Self::MoveDown => "Move Down",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
        }
    }
}

enum BatchMessage {
    Progress { completed: usize, total: usize },
    Done(BatchSummary),
}

#[derive(Clone)]
struct FontChoice {
    name: String,
    path: PathBuf,
    preview_family: String,
}

#[derive(Clone, Copy)]
enum AlignTarget {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectFile {
    version: u32,
    background_png: String,
    layers: Vec<ProjectLayer>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
enum ProjectLayer {
    Text {
        name: String,
        text: String,
        font_path: Option<String>,
        x: i32,
        y: i32,
        font_size: f32,
        color: [u8; 4],
        opacity: f32,
        stroke: bool,
        shadow: bool,
        visible: bool,
    },
    Image {
        name: String,
        image_png: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        opacity: f32,
        visible: bool,
    },
}

pub struct PhotoToolApp {
    language: Language,
    pending_open: Option<PathBuf>,
    image_path: Option<PathBuf>,
    image_info: Option<ImageInfo>,
    original_image: Option<DynamicImage>,
    working_image: Option<DynamicImage>,
    texture: Option<TextureHandle>,
    texture_size: Vec2,
    compose_layers: Vec<ComposeLayer>,
    selected_layer: Option<usize>,
    undo_stack: Vec<Vec<ComposeLayer>>,
    redo_stack: Vec<Vec<ComposeLayer>>,
    drag_snapshot: Option<Vec<ComposeLayer>>,
    font_bytes: Option<Vec<u8>>,
    font_choices: Vec<FontChoice>,
    folder_files: Vec<PathBuf>,
    selected_index: Option<usize>,
    zoom: f32,
    fit_to_window: bool,
    output_format: SupportedFormat,
    quality: u8,
    resize_width: u32,
    resize_height: u32,
    preserve_aspect: bool,
    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
    brightness_value: i32,
    contrast_value: f32,
    saturation_value: f32,
    exposure_value: f32,
    blur_sigma: f32,
    sharpen_sigma: f32,
    sharpen_threshold: i32,
    panorama_mode: PanoramaMode,
    panorama_width: u32,
    batch_input_dir: Option<PathBuf>,
    batch_output_dir: Option<PathBuf>,
    batch_resize_enabled: bool,
    batch_max_width: u32,
    batch_max_height: u32,
    batch_receiver: Option<Receiver<BatchMessage>>,
    batch_summary: Option<BatchSummary>,
    batch_completed: usize,
    batch_total: usize,
    status: String,
}

impl PhotoToolApp {
    pub fn new(initial_path: Option<PathBuf>) -> Self {
        Self {
            language: Language::ZhTw,
            pending_open: initial_path,
            image_path: None,
            image_info: None,
            original_image: None,
            working_image: None,
            texture: None,
            texture_size: Vec2::ZERO,
            compose_layers: Vec::new(),
            selected_layer: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            drag_snapshot: None,
            font_bytes: crate::fonts::load_system_font_bytes(),
            font_choices: load_font_choices(),
            folder_files: Vec::new(),
            selected_index: None,
            zoom: 1.0,
            fit_to_window: true,
            output_format: SupportedFormat::Jpeg,
            quality: 92,
            resize_width: 1,
            resize_height: 1,
            preserve_aspect: true,
            crop_x: 0,
            crop_y: 0,
            crop_width: 1,
            crop_height: 1,
            brightness_value: 0,
            contrast_value: 0.0,
            saturation_value: 0.0,
            exposure_value: 0.0,
            blur_sigma: 2.0,
            sharpen_sigma: 1.0,
            sharpen_threshold: 8,
            panorama_mode: PanoramaMode::Pad,
            panorama_width: 2048,
            batch_input_dir: None,
            batch_output_dir: None,
            batch_resize_enabled: false,
            batch_max_width: 2048,
            batch_max_height: 2048,
            batch_receiver: None,
            batch_summary: None,
            batch_completed: 0,
            batch_total: 0,
            status: "請開啟圖片或資料夾。".to_owned(),
        }
    }
}

impl Default for PhotoToolApp {
    fn default() -> Self {
        Self::new(None)
    }
}

impl eframe::App for PhotoToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_batch(ui.ctx());
        self.open_pending_path(ui.ctx());
        self.open_dropped_files(ui.ctx());
        self.handle_keyboard_shortcuts(ui.ctx());

        let ctx = ui.ctx().clone();
        egui::Panel::top("toolbar").show_inside(ui, |ui| self.toolbar(ui, &ctx));
        egui::Panel::left("metadata")
            .resizable(true)
            .default_size(360.0)
            .min_size(280.0)
            .show_inside(ui, |ui| self.metadata_panel(ui, &ctx));
        egui::Panel::right("actions")
            .resizable(true)
            .default_size(380.0)
            .min_size(320.0)
            .show_inside(ui, |ui| self.actions_panel(ui));
        egui::Panel::bottom("status").show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{}:", self.tr(TextKey::Status)));
                ui.label(&self.status);
            });
        });
        egui::CentralPanel::default_margins().show_inside(ui, |ui| self.preview_panel(ui));
    }
}

impl PhotoToolApp {
    fn tr(&self, key: TextKey) -> &'static str {
        self.language.t(key)
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.tr(TextKey::OpenImage)).clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.open_path(ctx, path);
            }

            if ui.button(self.tr(TextKey::OpenFolder)).clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                self.open_folder(ctx, path);
            }

            if ui.button(self.tr(TextKey::OpenProject)).clicked() {
                self.open_project(ctx);
            }

            if ui
                .add_enabled(
                    self.working_image.is_some(),
                    egui::Button::new(self.tr(TextKey::SaveAsNewFile)),
                )
                .clicked()
            {
                self.save_as_new_file();
            }

            ui.label(self.tr(TextKey::Project));
            if ui
                .add_enabled(
                    self.working_image.is_some(),
                    egui::Button::new(self.tr(TextKey::SaveProject)),
                )
                .clicked()
            {
                self.save_project();
            }

            if ui
                .add_enabled(
                    self.previous_index().is_some(),
                    egui::Button::new(self.tr(TextKey::Previous)),
                )
                .clicked()
            {
                self.open_folder_index(ctx, self.previous_index().unwrap_or(0));
            }

            if ui
                .add_enabled(
                    self.next_index().is_some(),
                    egui::Button::new("下一張 / Next"),
                )
                .clicked()
            {
                self.open_folder_index(ctx, self.next_index().unwrap_or(0));
            }

            if ui.button(self.tr(TextKey::Reload)).clicked()
                && let Some(path) = self.image_path.clone()
            {
                self.open_image(ctx, path);
            }

            ui.separator();

            egui::ComboBox::from_label(self.tr(TextKey::Language))
                .selected_text(self.language.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.language, Language::ZhTw, Language::ZhTw.name());
                    ui.selectable_value(&mut self.language, Language::En, Language::En.name());
                });

            ui.separator();

            if ui.button(self.tr(TextKey::Fit)).clicked() {
                self.fit_to_window = true;
            }
            if ui.button("100%").clicked() {
                self.fit_to_window = false;
                self.zoom = 1.0;
            }
            if ui.button(self.tr(TextKey::ZoomOut)).clicked() {
                self.fit_to_window = false;
                self.zoom = (self.zoom / 1.25).clamp(0.05, 8.0);
            }
            if ui.button(self.tr(TextKey::ZoomIn)).clicked() {
                self.fit_to_window = false;
                self.zoom = (self.zoom * 1.25).clamp(0.05, 8.0);
            }
        });
    }

    fn metadata_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(self.tr(TextKey::Image));
        ui.separator();

        match &self.image_info {
            Some(info) => {
                ui.label(format!(
                    "{}: {}",
                    self.tr(TextKey::File),
                    info.path.display()
                ));
                ui.label(format!("{}: {}", self.tr(TextKey::Format), info.format));
                ui.label(format!(
                    "{}: {} x {}",
                    self.tr(TextKey::Source),
                    info.width,
                    info.height
                ));
                if let Some(image) = &self.working_image {
                    ui.label(format!(
                        "{}: {} x {}",
                        self.tr(TextKey::Current),
                        image.width(),
                        image.height()
                    ));
                }
                ui.label(format!("Color: {}", info.color_type));
                ui.label(format!("Bytes: {}", info.file_size));
            }
            None => {
                ui.label(self.tr(TextKey::NoImage));
            }
        }

        ui.add_space(16.0);
        ui.heading(self.tr(TextKey::Folder));
        ui.separator();
        ui.label(format!(
            "{}: {}",
            self.tr(TextKey::Images),
            self.folder_files.len()
        ));

        egui::ScrollArea::vertical()
            .max_height(520.0)
            .show(ui, |ui| {
                for index in 0..self.folder_files.len() {
                    let file_name = self.folder_files[index]
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("(image)")
                        .to_owned();
                    let selected = self.selected_index == Some(index);
                    if ui.selectable_label(selected, file_name).clicked() {
                        self.open_folder_index(ctx, index);
                    }
                }
            });
    }

    fn actions_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr(TextKey::Actions));
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let ctx = ui.ctx().clone();
            self.convert_panel(ui);
            ui.add_space(20.0);
            self.edit_panel(ui, &ctx);
            ui.add_space(20.0);
            self.compose_panel(ui, &ctx);
            ui.add_space(20.0);
            self.panorama_panel(ui);
            ui.add_space(20.0);
            self.batch_panel(ui);
        });
    }

    fn convert_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr(TextKey::Convert));
        ui.separator();

        egui::ComboBox::from_label(self.tr(TextKey::Format))
            .selected_text(self.output_format.label())
            .show_ui(ui, |ui| {
                for format in SupportedFormat::ALL {
                    ui.selectable_value(&mut self.output_format, format, format.label());
                }
            });

        let quality_label = self.tr(TextKey::Quality);
        ui.add(egui::Slider::new(&mut self.quality, 1..=100).text(quality_label));

        let has_image = self.working_image.is_some();
        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::SaveConverted)),
            )
            .clicked()
        {
            self.save_converted();
        }
    }

    fn edit_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(self.tr(TextKey::Edit));
        ui.separator();

        let has_image = self.working_image.is_some();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::RotateLeft)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::rotate_left);
            }
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::RotateRight)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::rotate_right);
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::FlipH)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::flip_horizontal);
            }
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::FlipV)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::flip_vertical);
            }
        });

        ui.checkbox(
            &mut self.preserve_aspect,
            self.language.t(TextKey::PreserveAspect),
        );
        ui.horizontal(|ui| {
            let width_changed = ui
                .add(
                    egui::DragValue::new(&mut self.resize_width)
                        .range(1..=65535)
                        .prefix("W "),
                )
                .changed();
            let height_changed = ui
                .add(
                    egui::DragValue::new(&mut self.resize_height)
                        .range(1..=65535)
                        .prefix("H "),
                )
                .changed();
            if self.preserve_aspect {
                self.sync_resize_aspect(width_changed, height_changed);
            }
        });
        if ui
            .add_enabled(has_image, egui::Button::new(self.tr(TextKey::ApplyResize)))
            .clicked()
        {
            self.apply_resize(ctx);
        }

        ui.label(self.tr(TextKey::CropRectangle));
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.crop_x)
                    .range(0..=65535)
                    .prefix("X "),
            );
            ui.add(
                egui::DragValue::new(&mut self.crop_y)
                    .range(0..=65535)
                    .prefix("Y "),
            );
            ui.add(
                egui::DragValue::new(&mut self.crop_width)
                    .range(1..=65535)
                    .prefix("W "),
            );
            ui.add(
                egui::DragValue::new(&mut self.crop_height)
                    .range(1..=65535)
                    .prefix("H "),
            );
        });
        if ui
            .add_enabled(has_image, egui::Button::new(self.tr(TextKey::ApplyCrop)))
            .clicked()
        {
            self.apply_crop(ctx);
        }

        ui.separator();
        ui.label(self.tr(TextKey::Adjustments));
        let brightness_label = self.tr(TextKey::Brightness);
        ui.add(egui::Slider::new(&mut self.brightness_value, -255..=255).text(brightness_label));
        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::ApplyBrightness)),
            )
            .clicked()
        {
            let value = self.brightness_value;
            self.map_working_image(ctx, |image| {
                photo_core::transform::adjust_brightness(image, value)
            });
        }

        let contrast_label = self.tr(TextKey::Contrast);
        ui.add(egui::Slider::new(&mut self.contrast_value, -100.0..=100.0).text(contrast_label));
        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::ApplyContrast)),
            )
            .clicked()
        {
            let value = self.contrast_value;
            self.map_working_image(ctx, |image| {
                photo_core::transform::adjust_contrast(image, value)
            });
        }

        let saturation_label = self.tr(TextKey::Saturation);
        ui.add(
            egui::Slider::new(&mut self.saturation_value, -100.0..=100.0).text(saturation_label),
        );
        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::ApplySaturation)),
            )
            .clicked()
        {
            let value = self.saturation_value;
            self.map_working_image(ctx, |image| {
                photo_core::transform::adjust_saturation(image, value)
            });
        }

        let exposure_label = self.tr(TextKey::Exposure);
        ui.add(egui::Slider::new(&mut self.exposure_value, -2.0..=2.0).text(exposure_label));
        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::ApplyExposure)),
            )
            .clicked()
        {
            let value = self.exposure_value;
            self.map_working_image(ctx, |image| {
                photo_core::transform::adjust_exposure(image, value)
            });
        }

        let blur_label = self.tr(TextKey::Blur);
        ui.add(egui::Slider::new(&mut self.blur_sigma, 0.1..=20.0).text(blur_label));
        if ui
            .add_enabled(has_image, egui::Button::new(self.tr(TextKey::ApplyBlur)))
            .clicked()
        {
            let sigma = self.blur_sigma;
            self.map_working_image(ctx, |image| photo_core::transform::blur(image, sigma));
        }

        let sharpen_label = self.tr(TextKey::Sharpen);
        let threshold_label = self.tr(TextKey::Threshold);
        ui.add(egui::Slider::new(&mut self.sharpen_sigma, 0.1..=10.0).text(sharpen_label));
        ui.add(egui::Slider::new(&mut self.sharpen_threshold, 0..=255).text(threshold_label));
        if ui
            .add_enabled(has_image, egui::Button::new(self.tr(TextKey::ApplySharpen)))
            .clicked()
        {
            let sigma = self.sharpen_sigma;
            let threshold = self.sharpen_threshold;
            self.map_working_image(ctx, |image| {
                photo_core::transform::sharpen(image, sigma, threshold)
            });
        }

        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::Grayscale)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::grayscale);
            }
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::Invert)))
                .clicked()
            {
                self.map_working_image(ctx, photo_core::transform::invert);
            }
        });

        if ui
            .add_enabled(
                self.original_image.is_some(),
                egui::Button::new(self.tr(TextKey::ResetEdits)),
            )
            .clicked()
        {
            self.reset_edits(ctx);
        }
    }

    fn panorama_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("360");
        ui.separator();

        egui::ComboBox::from_label("Mode")
            .selected_text(self.panorama_mode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.panorama_mode, PanoramaMode::Pad, "pad");
                ui.selectable_value(&mut self.panorama_mode, PanoramaMode::Stretch, "stretch");
                ui.selectable_value(&mut self.panorama_mode, PanoramaMode::Crop, "crop");
            });

        ui.add(
            egui::DragValue::new(&mut self.panorama_width)
                .range(2..=32768)
                .prefix("Width "),
        );

        if ui
            .add_enabled(
                self.working_image.is_some(),
                egui::Button::new(self.tr(TextKey::Export360)),
            )
            .clicked()
        {
            self.export_panorama();
        }
    }

    fn compose_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(self.tr(TextKey::Compose));
        ui.separator();

        let has_image = self.working_image.is_some();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::AddText)))
                .clicked()
            {
                self.add_text_layer(ctx);
            }
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::AddImage)))
                .clicked()
            {
                self.add_image_layer_from_file(ctx);
            }
            if ui
                .add_enabled(has_image, egui::Button::new(self.tr(TextKey::PasteImage)))
                .clicked()
            {
                self.paste_image_layer(ctx);
            }
        });

        if ui
            .add_enabled(
                has_image,
                egui::Button::new(self.tr(TextKey::ExportComposite)),
            )
            .clicked()
        {
            self.export_composite();
        }

        ui.add_space(8.0);
        ui.label(self.tr(TextKey::Layers));
        let mut changed = false;
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for index in 0..self.compose_layers.len() {
                    let label = format!(
                        "{} {}",
                        if self.compose_layers[index].is_visible() {
                            "●"
                        } else {
                            "○"
                        },
                        self.compose_layers[index].name()
                    );
                    if ui
                        .selectable_label(self.selected_layer == Some(index), label)
                        .clicked()
                    {
                        self.selected_layer = Some(index);
                    }
                }
            });

        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(self.can_undo(), egui::Button::new(self.tr(TextKey::Undo)))
                .clicked()
            {
                self.undo(ctx);
            }
            if ui
                .add_enabled(self.can_redo(), egui::Button::new(self.tr(TextKey::Redo)))
                .clicked()
            {
                self.redo(ctx);
            }
            if ui
                .add_enabled(
                    self.can_move_layer_up(),
                    egui::Button::new(self.tr(TextKey::MoveUp)),
                )
                .clicked()
            {
                self.push_history();
                self.move_selected_layer(-1);
                changed = true;
            }
            if ui
                .add_enabled(
                    self.can_move_layer_down(),
                    egui::Button::new(self.tr(TextKey::MoveDown)),
                )
                .clicked()
            {
                self.push_history();
                self.move_selected_layer(1);
                changed = true;
            }
            if ui
                .add_enabled(
                    self.selected_layer.is_some(),
                    egui::Button::new(self.tr(TextKey::DuplicateLayer)),
                )
                .clicked()
            {
                self.push_history();
                self.duplicate_selected_layer();
                changed = true;
            }
            if ui
                .add_enabled(
                    self.selected_layer.is_some(),
                    egui::Button::new(self.tr(TextKey::DeleteLayer)),
                )
                .clicked()
            {
                self.push_history();
                self.delete_selected_layer();
                changed = true;
            }
        });

        ui.label(self.tr(TextKey::Align));
        ui.horizontal_wrapped(|ui| {
            for (target, label) in [
                (AlignTarget::Left, self.tr(TextKey::AlignLeft)),
                (AlignTarget::Center, self.tr(TextKey::AlignCenter)),
                (AlignTarget::Right, self.tr(TextKey::AlignRight)),
                (AlignTarget::Top, self.tr(TextKey::AlignTop)),
                (AlignTarget::Middle, self.tr(TextKey::AlignMiddle)),
                (AlignTarget::Bottom, self.tr(TextKey::AlignBottom)),
            ] {
                if ui
                    .add_enabled(self.can_align_selected_layer(), egui::Button::new(label))
                    .clicked()
                {
                    self.push_history();
                    self.align_selected_layer(target);
                    changed = true;
                }
            }
        });

        ui.separator();
        let before_edit = self.compose_layers.clone();
        let edited = self.selected_layer_editor(ui);
        if edited {
            self.push_history_snapshot(before_edit);
            changed = true;
        }
        ui.label(self.tr(TextKey::KeyboardHint));

        if changed {
            self.update_texture(ctx);
        }
    }

    fn selected_layer_editor(&mut self, ui: &mut egui::Ui) -> bool {
        let name_label = self.tr(TextKey::LayerName);
        let visible_label = self.tr(TextKey::Visible);
        let position_label = self.tr(TextKey::Position);
        let size_label = self.tr(TextKey::Size);
        let opacity_label = self.tr(TextKey::Opacity);
        let text_label = self.tr(TextKey::TextContent);
        let font_label = self.tr(TextKey::Font);
        let font_size_label = self.tr(TextKey::FontSize);
        let color_label = self.tr(TextKey::TextColor);
        let stroke_label = self.tr(TextKey::Stroke);
        let shadow_label = self.tr(TextKey::Shadow);
        let selected_label = self.tr(TextKey::SelectedLayer);
        let no_layer_label = self.tr(TextKey::NoLayer);
        let font_choices = self.font_choices.clone();

        let Some(index) = self.selected_layer else {
            ui.label(no_layer_label);
            return false;
        };

        let Some(layer) = self.compose_layers.get_mut(index) else {
            ui.label(no_layer_label);
            return false;
        };

        let mut changed = false;
        ui.label(selected_label);
        ui.horizontal(|ui| {
            ui.label(name_label);
            changed |= ui.text_edit_singleline(layer.name_mut()).changed();
        });

        let mut visible = layer.is_visible();
        if ui.checkbox(&mut visible, visible_label).changed() {
            layer.set_visible(visible);
            changed = true;
        }

        match layer {
            ComposeLayer::Text(layer) => {
                ui.label(text_label);
                changed |= ui.text_edit_multiline(&mut layer.text).changed();
                ui.label(position_label);
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(egui::DragValue::new(&mut layer.x).prefix("X "))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut layer.y).prefix("Y "))
                        .changed();
                });
                changed |= ui
                    .add(egui::Slider::new(&mut layer.font_size, 8.0..=256.0).text(font_size_label))
                    .changed();
                let selected_font = layer
                    .font_path
                    .as_ref()
                    .and_then(|path| {
                        font_choices
                            .iter()
                            .find(|choice| &choice.path == path)
                            .map(|choice| choice.name.as_str())
                    })
                    .unwrap_or("系統預設 / Default");
                egui::ComboBox::from_label(font_label)
                    .selected_text(selected_font)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                layer.font_path.is_none(),
                                egui::RichText::new("Aa 字型 - 系統預設 / Default").size(18.0),
                            )
                            .clicked()
                        {
                            layer.font_path = None;
                            changed = true;
                        }

                        for choice in &font_choices {
                            let selected = layer.font_path.as_ref() == Some(&choice.path);
                            let preview = egui::RichText::new(format!("Aa 字型 - {}", choice.name))
                                .font(egui::FontId::new(
                                    18.0,
                                    egui::FontFamily::Name(choice.preview_family.clone().into()),
                                ));
                            if ui.selectable_label(selected, preview).clicked() {
                                layer.font_path = Some(choice.path.clone());
                                changed = true;
                            }
                        }
                    });
                changed |= ui
                    .add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0).text(opacity_label))
                    .changed();
                ui.label(color_label);
                changed |= ui
                    .color_edit_button_srgba_unmultiplied(&mut layer.color)
                    .changed();
                changed |= ui.checkbox(&mut layer.stroke, stroke_label).changed();
                changed |= ui.checkbox(&mut layer.shadow, shadow_label).changed();
            }
            ComposeLayer::Image(layer) => {
                ui.label(position_label);
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(egui::DragValue::new(&mut layer.x).prefix("X "))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut layer.y).prefix("Y "))
                        .changed();
                });
                ui.label(size_label);
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut layer.width)
                                .range(1..=65535)
                                .prefix("W "),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut layer.height)
                                .range(1..=65535)
                                .prefix("H "),
                        )
                        .changed();
                });
                changed |= ui
                    .add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0).text(opacity_label))
                    .changed();
            }
        }

        changed
    }

    fn batch_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr(TextKey::Batch));
        ui.separator();

        if ui.button(self.tr(TextKey::InputFolder)).clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            self.batch_input_dir = Some(path);
        }
        if let Some(path) = &self.batch_input_dir {
            ui.label(format!(
                "{}: {}",
                self.tr(TextKey::InputFolder),
                path.display()
            ));
        }

        if ui.button(self.tr(TextKey::OutputFolder)).clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            self.batch_output_dir = Some(path);
        }
        if let Some(path) = &self.batch_output_dir {
            ui.label(format!(
                "{}: {}",
                self.tr(TextKey::OutputFolder),
                path.display()
            ));
        }

        ui.checkbox(
            &mut self.batch_resize_enabled,
            self.language.t(TextKey::ResizeToFit),
        );
        let max_w_label = self.tr(TextKey::MaxW);
        let max_h_label = self.tr(TextKey::MaxH);
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.batch_max_width)
                    .range(1..=65535)
                    .prefix(max_w_label),
            );
            ui.add(
                egui::DragValue::new(&mut self.batch_max_height)
                    .range(1..=65535)
                    .prefix(max_h_label),
            );
        });

        let can_run = self.batch_input_dir.is_some()
            && self.batch_output_dir.is_some()
            && self.batch_receiver.is_none();
        if ui
            .add_enabled(can_run, egui::Button::new(self.tr(TextKey::RunBatch)))
            .clicked()
        {
            self.start_batch();
        }

        if self.batch_receiver.is_some() {
            let progress = if self.batch_total == 0 {
                0.0
            } else {
                self.batch_completed as f32 / self.batch_total as f32
            };
            ui.add(
                egui::ProgressBar::new(progress)
                    .animate(true)
                    .text(format!("{} / {}", self.batch_completed, self.batch_total)),
            );
            ui.label(self.tr(TextKey::BatchRunning));
        }

        if let Some(summary) = &self.batch_summary {
            ui.label(format!(
                "{}: {} total, {} succeeded, {} failed",
                self.tr(TextKey::LastBatch),
                summary.total,
                summary.succeeded,
                summary.failed
            ));
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .show(ui, |ui| {
                    for result in summary.results.iter().take(12) {
                        let label = match (&result.output, &result.error) {
                            (Some(output), None) => format!("OK {}", output.display()),
                            (_, Some(error)) => {
                                format!("Failed {}: {error}", result.input.display())
                            }
                            _ => format!("Skipped {}", result.input.display()),
                        };
                        ui.label(label);
                    }
                });
        }
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(texture) = &self.texture {
            let available = ui.available_size().max(Vec2::splat(1.0));
            let scale = if self.fit_to_window {
                (available.x / self.texture_size.x)
                    .min(available.y / self.texture_size.y)
                    .clamp(0.01, 1.0)
            } else {
                self.zoom
            };
            let display_size = self.texture_size * scale;
            let texture_id = texture.id();
            let (viewport_rect, _) = ui.allocate_exact_size(available, Sense::hover());
            let image_rect = Rect::from_center_size(viewport_rect.center(), display_size);
            let response = ui.interact(
                image_rect,
                ui.id().with("canvas_image"),
                Sense::click_and_drag(),
            );
            ui.painter().image(
                texture_id,
                image_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            self.handle_canvas_interaction(ui.ctx(), &response, image_rect, scale);
            self.paint_selected_layer(ui, image_rect, scale);
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading("Photo Tool");
                ui.label(self.tr(TextKey::BrowseHint));
            });
        }
    }

    fn handle_canvas_interaction(
        &mut self,
        ctx: &egui::Context,
        response: &egui::Response,
        image_rect: Rect,
        scale: f32,
    ) {
        if response.drag_started()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let image_pos = screen_to_image(pointer, image_rect, scale);
            self.selected_layer = self.hit_test_layer(image_pos);
            self.drag_snapshot = self
                .selected_layer
                .is_some()
                .then(|| self.compose_layers.clone());
        }

        if response.dragged()
            && let Some(index) = self.selected_layer
        {
            let frame_delta = response.drag_delta() / scale.max(0.01);
            if frame_delta.length_sq() > 0.0 {
                self.move_layer_by(index, frame_delta);
                self.update_texture(ctx);
            }
        }

        if response.drag_stopped()
            && let Some(snapshot) = self.drag_snapshot.take()
        {
            self.push_history_snapshot(snapshot);
        }

        if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let image_pos = screen_to_image(pointer, image_rect, scale);
            self.selected_layer = self.hit_test_layer(image_pos);
        }
    }

    fn paint_selected_layer(&self, ui: &mut egui::Ui, image_rect: Rect, scale: f32) {
        let Some(index) = self.selected_layer else {
            return;
        };
        let Some(layer) = self.compose_layers.get(index) else {
            return;
        };
        let bounds = self.layer_bounds(layer);
        let min = image_rect.min + bounds.min.to_vec2() * scale;
        let max = image_rect.min + bounds.max.to_vec2() * scale;
        ui.painter().rect_stroke(
            Rect::from_min_max(min, max),
            0.0,
            Stroke::new(2.0, Color32::from_rgb(80, 180, 255)),
            StrokeKind::Outside,
        );
    }

    fn open_pending_path(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.pending_open.take() {
            self.open_path(ctx, path);
        }
    }

    fn open_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path {
                self.open_path(ctx, path);
                break;
            }
        }
    }

    fn open_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        if path.is_dir() {
            self.open_folder(ctx, path);
        } else {
            self.open_image(ctx, path);
        }
    }

    fn open_folder(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.folder_files = collect_supported_files(&path);
        self.folder_files.sort();
        self.selected_index = None;

        if self.folder_files.is_empty() {
            self.status = match self.language {
                Language::ZhTw => format!("資料夾沒有可支援的圖片：{}", path.display()),
                Language::En => format!("No supported images found in {}", path.display()),
            };
            return;
        }

        self.open_folder_index(ctx, 0);
        self.status = match self.language {
            Language::ZhTw => format!(
                "已載入資料夾：{}，共 {} 張圖片",
                path.display(),
                self.folder_files.len()
            ),
            Language::En => format!(
                "Loaded folder {} with {} images",
                path.display(),
                self.folder_files.len()
            ),
        };
    }

    fn open_folder_index(&mut self, ctx: &egui::Context, index: usize) {
        if let Some(path) = self.folder_files.get(index).cloned() {
            self.selected_index = Some(index);
            self.open_image(ctx, path);
        }
    }

    fn open_image(&mut self, ctx: &egui::Context, path: PathBuf) {
        match load_dynamic_image(&path) {
            Ok(image) => {
                self.original_image = Some(image.clone());
                self.working_image = Some(image);
                self.compose_layers.clear();
                self.selected_layer = None;
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.drag_snapshot = None;
                self.image_info = inspect_image(&path).ok();
                self.image_path = Some(path.clone());
                self.sync_edit_dimensions();
                self.update_texture(ctx);
                self.status = match self.language {
                    Language::ZhTw => format!("已載入：{}", path.display()),
                    Language::En => format!("Loaded {}", path.display()),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => {
                        format!("無法開啟圖片：{}。錯誤：{error}", path.display())
                    }
                    Language::En => format!("Failed to open image {}: {error}", path.display()),
                };
            }
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(image) = self.render_current_image() {
            let rgba = image.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.texture = Some(ctx.load_texture("preview", color_image, TextureOptions::LINEAR));
            self.texture_size = Vec2::new(image.width() as f32, image.height() as f32);
        }
    }

    fn save_converted(&mut self) {
        let Some(image) = self.render_current_image() else {
            return;
        };

        let Some(output) = rfd::FileDialog::new()
            .set_file_name(format!("converted.{}", self.output_format.extension()))
            .save_file()
        else {
            return;
        };

        match save_dynamic_image(
            &image,
            &output,
            ConvertOptions {
                format: self.output_format,
                quality: self.quality,
                background: [255, 255, 255, 255],
            },
        ) {
            Ok(result) => {
                self.status = match self.language {
                    Language::ZhTw => format!(
                        "已轉檔到 {}（{}x{}）",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                    Language::En => format!(
                        "Converted to {} ({}x{})",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("轉檔失敗：{error}"),
                    Language::En => format!("Conversion failed: {error}"),
                };
            }
        }
    }

    fn save_as_new_file(&mut self) {
        let Some(image) = self.render_current_image() else {
            return;
        };

        let default_name = self.default_save_as_name();
        let Some(mut output) = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff"],
            )
            .set_file_name(default_name)
            .save_file()
        else {
            return;
        };

        if output.extension().is_none() {
            output.set_extension(self.output_format.extension());
        }
        let format = SupportedFormat::from_path(&output).unwrap_or(self.output_format);

        match save_dynamic_image(
            &image,
            &output,
            ConvertOptions {
                format,
                quality: self.quality,
                background: [255, 255, 255, 255],
            },
        ) {
            Ok(result) => {
                self.status = match self.language {
                    Language::ZhTw => format!(
                        "已另存新檔：{}（{}x{}）",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                    Language::En => format!(
                        "Saved as {} ({}x{})",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("另存新檔失敗：{error}"),
                    Language::En => format!("Save as failed: {error}"),
                };
            }
        }
    }

    fn default_save_as_name(&self) -> String {
        let stem = self
            .image_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|value| value.to_str())
            .unwrap_or("photo");
        format!("{stem}_edited.{}", self.output_format.extension())
    }

    fn export_panorama(&mut self) {
        let Some(image) = self.render_current_image() else {
            return;
        };

        let Some(output) = rfd::FileDialog::new()
            .set_file_name("photo_360.jpg")
            .save_file()
        else {
            return;
        };

        let input = self.image_path.clone().unwrap_or_default();
        match write_panorama_dynamic_jpeg(
            &image,
            &output,
            PanoramaOptions {
                mode: self.panorama_mode,
                target_width: Some(self.panorama_width.max(2)),
                quality: self.quality,
                background: [0, 0, 0, 255],
            },
            input,
        ) {
            Ok(result) => {
                self.status = match self.language {
                    Language::ZhTw => format!(
                        "已匯出 360 JPEG 到 {}（{}x{}，模式={}）",
                        result.output.display(),
                        result.width,
                        result.height,
                        result.mode
                    ),
                    Language::En => format!(
                        "Exported 360 JPEG to {} ({}x{}, mode={})",
                        result.output.display(),
                        result.width,
                        result.height,
                        result.mode
                    ),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("360 匯出失敗：{error}"),
                    Language::En => format!("360 export failed: {error}"),
                };
            }
        }
    }

    fn map_working_image<F>(&mut self, ctx: &egui::Context, mapper: F)
    where
        F: FnOnce(&DynamicImage) -> DynamicImage,
    {
        if let Some(image) = &self.working_image {
            self.working_image = Some(mapper(image));
            self.sync_edit_dimensions();
            self.update_texture(ctx);
            self.status = match self.language {
                Language::ZhTw => "已套用編輯。".to_owned(),
                Language::En => "Edit applied.".to_owned(),
            };
        }
    }

    fn apply_resize(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.working_image {
            self.working_image = Some(photo_core::resize::resize_exact(
                image,
                self.resize_width,
                self.resize_height,
            ));
            self.sync_edit_dimensions();
            self.update_texture(ctx);
            self.status = match self.language {
                Language::ZhTw => "已套用尺寸調整。".to_owned(),
                Language::En => "Resize applied.".to_owned(),
            };
        }
    }

    fn apply_crop(&mut self, ctx: &egui::Context) {
        let Some(image) = &self.working_image else {
            return;
        };

        let x = self.crop_x.min(image.width().saturating_sub(1));
        let y = self.crop_y.min(image.height().saturating_sub(1));
        let width = self.crop_width.min(image.width().saturating_sub(x)).max(1);
        let height = self
            .crop_height
            .min(image.height().saturating_sub(y))
            .max(1);
        self.working_image = Some(image.crop_imm(x, y, width, height));
        self.sync_edit_dimensions();
        self.update_texture(ctx);
        self.status = match self.language {
            Language::ZhTw => "已套用裁切。".to_owned(),
            Language::En => "Crop applied.".to_owned(),
        };
    }

    fn reset_edits(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.original_image {
            self.working_image = Some(image.clone());
            self.sync_edit_dimensions();
            self.update_texture(ctx);
            self.status = match self.language {
                Language::ZhTw => "已重設編輯。".to_owned(),
                Language::En => "Edits reset.".to_owned(),
            };
        }
    }

    fn sync_resize_aspect(&mut self, width_changed: bool, height_changed: bool) {
        let Some(image) = &self.working_image else {
            return;
        };
        let ratio = image.width() as f32 / image.height().max(1) as f32;

        if width_changed {
            self.resize_height = ((self.resize_width as f32 / ratio).round() as u32).max(1);
        } else if height_changed {
            self.resize_width = ((self.resize_height as f32 * ratio).round() as u32).max(1);
        }
    }

    fn sync_edit_dimensions(&mut self) {
        if let Some(image) = &self.working_image {
            self.resize_width = image.width().max(1);
            self.resize_height = image.height().max(1);
            self.crop_x = 0;
            self.crop_y = 0;
            self.crop_width = image.width().max(1);
            self.crop_height = image.height().max(1);
        }
    }

    fn start_batch(&mut self) {
        let Some(input_dir) = self.batch_input_dir.clone() else {
            return;
        };
        let Some(output_dir) = self.batch_output_dir.clone() else {
            return;
        };

        let options = BatchOptions {
            output_dir: output_dir.clone(),
            convert: ConvertOptions {
                format: self.output_format,
                quality: self.quality,
                background: [255, 255, 255, 255],
            },
            max_width: self.batch_resize_enabled.then_some(self.batch_max_width),
            max_height: self.batch_resize_enabled.then_some(self.batch_max_height),
        };

        let (sender, receiver) = mpsc::channel();
        self.batch_receiver = Some(receiver);
        self.batch_summary = None;
        self.batch_completed = 0;
        self.batch_total = 0;
        self.status = match self.language {
            Language::ZhTw => "批次轉檔已開始。".to_owned(),
            Language::En => "Batch conversion started.".to_owned(),
        };

        thread::spawn(move || {
            let _ = std::fs::create_dir_all(&output_dir);
            let files = collect_supported_files(&input_dir);
            let total = files.len();
            let _ = sender.send(BatchMessage::Progress {
                completed: 0,
                total,
            });
            let progress_sender = sender.clone();
            let summary = process_batch_with_progress(&files, &options, move |completed, total| {
                let _ = progress_sender.send(BatchMessage::Progress { completed, total });
            });
            let _ = sender.send(BatchMessage::Done(summary));
        });
    }

    fn poll_batch(&mut self, ctx: &egui::Context) {
        let mut clear_receiver = false;

        if let Some(receiver) = &self.batch_receiver {
            loop {
                match receiver.try_recv() {
                    Ok(BatchMessage::Progress { completed, total }) => {
                        self.batch_completed = completed;
                        self.batch_total = total;
                    }
                    Ok(BatchMessage::Done(summary)) => {
                        self.batch_completed = summary.total;
                        self.batch_total = summary.total;
                        self.status = match self.language {
                            Language::ZhTw => format!(
                                "批次完成：共 {}，成功 {}，失敗 {}",
                                summary.total, summary.succeeded, summary.failed
                            ),
                            Language::En => format!(
                                "Batch complete: {} total, {} succeeded, {} failed",
                                summary.total, summary.succeeded, summary.failed
                            ),
                        };
                        self.batch_summary = Some(summary);
                        clear_receiver = true;
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        ctx.request_repaint();
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.status = match self.language {
                            Language::ZhTw => "批次背景工作非預期停止。".to_owned(),
                            Language::En => "Batch worker stopped unexpectedly.".to_owned(),
                        };
                        clear_receiver = true;
                        break;
                    }
                }
            }
        }

        if clear_receiver {
            self.batch_receiver = None;
        }
    }

    fn render_current_image(&self) -> Option<DynamicImage> {
        let background = self.working_image.as_ref()?.clone();
        if self.compose_layers.is_empty() {
            return Some(background);
        }

        Some(render_composition(
            &ComposeDocument {
                background,
                layers: self.compose_layers.clone(),
            },
            self.font_bytes.as_deref(),
        ))
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }

        let (undo, redo, delete, duplicate, left, right, up, down) = ctx.input(|input| {
            (
                input.modifiers.ctrl && input.key_pressed(egui::Key::Z),
                input.modifiers.ctrl && input.key_pressed(egui::Key::Y),
                input.key_pressed(egui::Key::Delete),
                input.modifiers.ctrl && input.key_pressed(egui::Key::D),
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
            )
        });

        if undo {
            self.undo(ctx);
            return;
        }
        if redo {
            self.redo(ctx);
            return;
        }
        if delete && self.selected_layer.is_some() {
            self.push_history();
            self.delete_selected_layer();
            self.update_texture(ctx);
            return;
        }
        if duplicate && self.selected_layer.is_some() {
            self.push_history();
            self.duplicate_selected_layer();
            self.update_texture(ctx);
            return;
        }

        let mut delta = Vec2::ZERO;
        if left {
            delta.x -= 1.0;
        }
        if right {
            delta.x += 1.0;
        }
        if up {
            delta.y -= 1.0;
        }
        if down {
            delta.y += 1.0;
        }

        if delta != Vec2::ZERO
            && let Some(index) = self.selected_layer
        {
            self.push_history();
            self.move_layer_by(index, delta);
            self.update_texture(ctx);
        }
    }

    fn push_history(&mut self) {
        self.push_history_snapshot(self.compose_layers.clone());
    }

    fn push_history_snapshot(&mut self, snapshot: Vec<ComposeLayer>) {
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn undo(&mut self, ctx: &egui::Context) {
        let Some(previous) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push(self.compose_layers.clone());
        self.compose_layers = previous;
        self.selected_layer =
            normalize_selected_layer(self.selected_layer, self.compose_layers.len());
        self.update_texture(ctx);
    }

    fn redo(&mut self, ctx: &egui::Context) {
        let Some(next) = self.redo_stack.pop() else {
            return;
        };
        self.undo_stack.push(self.compose_layers.clone());
        self.compose_layers = next;
        self.selected_layer =
            normalize_selected_layer(self.selected_layer, self.compose_layers.len());
        self.update_texture(ctx);
    }

    fn add_text_layer(&mut self, ctx: &egui::Context) {
        self.push_history();
        let mut layer = TextLayer {
            name: match self.language {
                Language::ZhTw => format!("文字 {}", self.compose_layers.len() + 1),
                Language::En => format!("Text {}", self.compose_layers.len() + 1),
            },
            ..Default::default()
        };
        if let Some(image) = &self.working_image {
            layer.x = (image.width() / 10) as i32;
            layer.y = (image.height() / 10) as i32;
            layer.font_size = (image.height() as f32 / 12.0).clamp(24.0, 96.0);
        }
        self.compose_layers.push(ComposeLayer::Text(layer));
        self.selected_layer = Some(self.compose_layers.len() - 1);
        self.update_texture(ctx);
        self.status = match self.language {
            Language::ZhTw => "已新增文字圖層。".to_owned(),
            Language::En => "Text layer added.".to_owned(),
        };
    }

    fn add_image_layer_from_file(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };

        match load_dynamic_image(&path) {
            Ok(image) => {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("image")
                    .to_owned();
                self.push_image_layer(ctx, name, image);
                self.status = match self.language {
                    Language::ZhTw => format!("已加入圖片圖層：{}", path.display()),
                    Language::En => format!("Image layer added: {}", path.display()),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("無法加入圖片：{error}"),
                    Language::En => format!("Failed to add image: {error}"),
                };
            }
        }
    }

    fn paste_image_layer(&mut self, ctx: &egui::Context) {
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_image()) {
            Ok(image) => {
                let width = image.width as u32;
                let height = image.height as u32;
                let bytes = image.bytes.into_owned();
                if let Some(rgba) = image::RgbaImage::from_raw(width, height, bytes) {
                    let name = match self.language {
                        Language::ZhTw => format!("貼上圖片 {}", self.compose_layers.len() + 1),
                        Language::En => format!("Pasted Image {}", self.compose_layers.len() + 1),
                    };
                    self.push_image_layer(ctx, name, DynamicImage::ImageRgba8(rgba));
                    self.status = match self.language {
                        Language::ZhTw => "已從剪貼簿貼上圖片。".to_owned(),
                        Language::En => "Image pasted from clipboard.".to_owned(),
                    };
                } else {
                    self.status = match self.language {
                        Language::ZhTw => "剪貼簿圖片格式無法讀取。".to_owned(),
                        Language::En => "Clipboard image format could not be read.".to_owned(),
                    };
                }
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("剪貼簿沒有可貼上的圖片：{error}"),
                    Language::En => format!("No image could be pasted from clipboard: {error}"),
                };
            }
        }
    }

    fn push_image_layer(&mut self, ctx: &egui::Context, name: String, image: DynamicImage) {
        self.push_history();
        let mut layer = ImageLayer::new(name, image);
        if let Some(background) = &self.working_image {
            let max_width = (background.width() / 2).max(1);
            let max_height = (background.height() / 2).max(1);
            let scale = (max_width as f32 / layer.width as f32)
                .min(max_height as f32 / layer.height as f32)
                .min(1.0);
            layer.width = ((layer.width as f32 * scale).round() as u32).max(1);
            layer.height = ((layer.height as f32 * scale).round() as u32).max(1);
            layer.x = ((background.width().saturating_sub(layer.width)) / 2) as i32;
            layer.y = ((background.height().saturating_sub(layer.height)) / 2) as i32;
        }
        self.compose_layers.push(ComposeLayer::Image(layer));
        self.selected_layer = Some(self.compose_layers.len() - 1);
        self.update_texture(ctx);
    }

    fn export_composite(&mut self) {
        let Some(image) = self.render_current_image() else {
            return;
        };

        let Some(output) = rfd::FileDialog::new()
            .set_file_name(format!("composite.{}", self.output_format.extension()))
            .save_file()
        else {
            return;
        };

        match save_dynamic_image(
            &image,
            &output,
            ConvertOptions {
                format: self.output_format,
                quality: self.quality,
                background: [255, 255, 255, 255],
            },
        ) {
            Ok(result) => {
                self.status = match self.language {
                    Language::ZhTw => format!(
                        "已匯出合成圖：{}（{}x{}）",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                    Language::En => format!(
                        "Composite exported to {} ({}x{})",
                        result.output.display(),
                        result.width,
                        result.height
                    ),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("合成圖匯出失敗：{error}"),
                    Language::En => format!("Composite export failed: {error}"),
                };
            }
        }
    }

    fn save_project(&mut self) {
        let Some(output) = rfd::FileDialog::new()
            .add_filter("Photo Project", &["photo-project"])
            .set_file_name("photo.photo-project")
            .save_file()
        else {
            return;
        };

        let result = self
            .project_from_state()
            .and_then(|project| {
                serde_json::to_string_pretty(&project).map_err(|error| error.to_string())
            })
            .and_then(|json| std::fs::write(&output, json).map_err(|error| error.to_string()));

        match result {
            Ok(()) => {
                self.status = match self.language {
                    Language::ZhTw => format!("已儲存專案：{}", output.display()),
                    Language::En => format!("Project saved: {}", output.display()),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("專案儲存失敗：{error}"),
                    Language::En => format!("Project save failed: {error}"),
                };
            }
        }
    }

    fn open_project(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Photo Project", &["photo-project"])
            .pick_file()
        else {
            return;
        };

        let result = std::fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|json| {
                serde_json::from_str::<ProjectFile>(&json).map_err(|error| error.to_string())
            })
            .and_then(|project| self.apply_project(ctx, project));

        match result {
            Ok(()) => {
                self.status = match self.language {
                    Language::ZhTw => format!("已開啟專案：{}", path.display()),
                    Language::En => format!("Project opened: {}", path.display()),
                };
            }
            Err(error) => {
                self.status = match self.language {
                    Language::ZhTw => format!("專案開啟失敗：{error}"),
                    Language::En => format!("Project open failed: {error}"),
                };
            }
        }
    }

    fn project_from_state(&self) -> Result<ProjectFile, String> {
        let background = self
            .working_image
            .as_ref()
            .ok_or_else(|| "No image loaded".to_owned())?;
        let mut layers = Vec::with_capacity(self.compose_layers.len());

        for layer in &self.compose_layers {
            match layer {
                ComposeLayer::Text(layer) => layers.push(ProjectLayer::Text {
                    name: layer.name.clone(),
                    text: layer.text.clone(),
                    font_path: layer
                        .font_path
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string()),
                    x: layer.x,
                    y: layer.y,
                    font_size: layer.font_size,
                    color: layer.color,
                    opacity: layer.opacity,
                    stroke: layer.stroke,
                    shadow: layer.shadow,
                    visible: layer.visible,
                }),
                ComposeLayer::Image(layer) => layers.push(ProjectLayer::Image {
                    name: layer.name.clone(),
                    image_png: encode_png_base64(&layer.image)?,
                    x: layer.x,
                    y: layer.y,
                    width: layer.width,
                    height: layer.height,
                    opacity: layer.opacity,
                    visible: layer.visible,
                }),
            }
        }

        Ok(ProjectFile {
            version: 1,
            background_png: encode_png_base64(background)?,
            layers,
        })
    }

    fn apply_project(&mut self, ctx: &egui::Context, project: ProjectFile) -> Result<(), String> {
        if project.version != 1 {
            return Err(format!("Unsupported project version {}", project.version));
        }

        let background = decode_png_base64(&project.background_png)?;
        let mut layers = Vec::with_capacity(project.layers.len());

        for layer in project.layers {
            match layer {
                ProjectLayer::Text {
                    name,
                    text,
                    font_path,
                    x,
                    y,
                    font_size,
                    color,
                    opacity,
                    stroke,
                    shadow,
                    visible,
                } => layers.push(ComposeLayer::Text(TextLayer {
                    name,
                    text,
                    font_path: font_path.map(PathBuf::from),
                    x,
                    y,
                    font_size,
                    color,
                    opacity,
                    stroke,
                    shadow,
                    visible,
                })),
                ProjectLayer::Image {
                    name,
                    image_png,
                    x,
                    y,
                    width,
                    height,
                    opacity,
                    visible,
                } => layers.push(ComposeLayer::Image(ImageLayer {
                    name,
                    image: decode_png_base64(&image_png)?,
                    x,
                    y,
                    width,
                    height,
                    opacity,
                    visible,
                })),
            }
        }

        self.original_image = Some(background.clone());
        self.working_image = Some(background);
        self.image_path = None;
        self.image_info = None;
        self.compose_layers = layers;
        self.selected_layer = normalize_selected_layer(None, self.compose_layers.len());
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.drag_snapshot = None;
        self.sync_edit_dimensions();
        self.update_texture(ctx);
        Ok(())
    }

    fn can_move_layer_up(&self) -> bool {
        self.selected_layer
            .is_some_and(|index| index + 1 < self.compose_layers.len())
    }

    fn can_move_layer_down(&self) -> bool {
        self.selected_layer.is_some_and(|index| index > 0)
    }

    fn move_selected_layer(&mut self, direction: i32) {
        let Some(index) = self.selected_layer else {
            return;
        };
        let new_index = if direction < 0 {
            index + 1
        } else {
            index.saturating_sub(1)
        };
        if new_index < self.compose_layers.len() {
            self.compose_layers.swap(index, new_index);
            self.selected_layer = Some(new_index);
        }
    }

    fn duplicate_selected_layer(&mut self) {
        let Some(index) = self.selected_layer else {
            return;
        };
        let Some(layer) = self.compose_layers.get(index).cloned() else {
            return;
        };
        let mut layer = layer;
        match &mut layer {
            ComposeLayer::Text(layer) => {
                layer.name = format!("{} copy", layer.name);
                layer.x += 16;
                layer.y += 16;
            }
            ComposeLayer::Image(layer) => {
                layer.name = format!("{} copy", layer.name);
                layer.x += 16;
                layer.y += 16;
            }
        }
        let insert_at = index + 1;
        self.compose_layers.insert(insert_at, layer);
        self.selected_layer = Some(insert_at);
    }

    fn move_layer_by(&mut self, index: usize, delta: Vec2) {
        let Some(layer) = self.compose_layers.get_mut(index) else {
            return;
        };
        let dx = delta.x.round() as i32;
        let dy = delta.y.round() as i32;
        match layer {
            ComposeLayer::Text(layer) => {
                layer.x += dx;
                layer.y += dy;
            }
            ComposeLayer::Image(layer) => {
                layer.x += dx;
                layer.y += dy;
            }
        }
    }

    fn hit_test_layer(&self, image_pos: Pos2) -> Option<usize> {
        self.compose_layers
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, layer)| {
                (layer.is_visible() && self.layer_bounds(layer).contains(image_pos))
                    .then_some(index)
            })
    }

    fn can_align_selected_layer(&self) -> bool {
        self.selected_layer.is_some() && self.working_image.is_some()
    }

    fn align_selected_layer(&mut self, target: AlignTarget) {
        let Some(index) = self.selected_layer else {
            return;
        };
        let Some(image) = &self.working_image else {
            return;
        };
        let Some(layer) = self.compose_layers.get(index) else {
            return;
        };

        let bounds = self.layer_bounds(layer);
        let image_width = image.width() as f32;
        let image_height = image.height() as f32;
        let dx = match target {
            AlignTarget::Left => -bounds.min.x,
            AlignTarget::Center => (image_width - bounds.width()) / 2.0 - bounds.min.x,
            AlignTarget::Right => image_width - bounds.width() - bounds.min.x,
            _ => 0.0,
        };
        let dy = match target {
            AlignTarget::Top => -bounds.min.y,
            AlignTarget::Middle => (image_height - bounds.height()) / 2.0 - bounds.min.y,
            AlignTarget::Bottom => image_height - bounds.height() - bounds.min.y,
            _ => 0.0,
        };

        self.move_layer_by(index, Vec2::new(dx, dy));
    }

    fn layer_bounds(&self, layer: &ComposeLayer) -> Rect {
        match layer {
            ComposeLayer::Text(layer) => {
                let bounds = text_layer_bounds(layer, self.font_bytes.as_deref());
                Rect::from_min_size(
                    Pos2::new(bounds.x as f32, bounds.y as f32),
                    Vec2::new(bounds.width as f32, bounds.height as f32),
                )
            }
            ComposeLayer::Image(layer) => Rect::from_min_size(
                Pos2::new(layer.x as f32, layer.y as f32),
                Vec2::new(layer.width as f32, layer.height as f32),
            ),
        }
    }

    fn delete_selected_layer(&mut self) {
        let Some(index) = self.selected_layer else {
            return;
        };
        if index < self.compose_layers.len() {
            self.compose_layers.remove(index);
        }
        self.selected_layer = if self.compose_layers.is_empty() {
            None
        } else {
            Some(index.min(self.compose_layers.len() - 1))
        };
    }

    fn previous_index(&self) -> Option<usize> {
        self.selected_index
            .filter(|index| *index > 0)
            .map(|index| index - 1)
    }

    fn next_index(&self) -> Option<usize> {
        self.selected_index
            .and_then(|index| (index + 1 < self.folder_files.len()).then_some(index + 1))
    }
}

fn screen_to_image(pointer: Pos2, image_rect: Rect, scale: f32) -> Pos2 {
    let offset = (pointer - image_rect.min) / scale.max(0.01);
    Pos2::new(offset.x, offset.y)
}

fn normalize_selected_layer(selected: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selected.unwrap_or(0).min(len - 1))
    }
}

fn encode_png_base64(image: &DynamicImage) -> Result<String, String> {
    let mut cursor = Cursor::new(Vec::new());
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(general_purpose::STANDARD.encode(cursor.into_inner()))
}

fn decode_png_base64(encoded: &str) -> Result<DynamicImage, String> {
    let bytes = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| error.to_string())?;
    image::load_from_memory(&bytes).map_err(|error| error.to_string())
}

fn load_font_choices() -> Vec<FontChoice> {
    crate::fonts::list_system_font_paths()
        .into_iter()
        .map(|path| {
            let name = crate::fonts::display_font_name(&path);
            let preview_family = crate::fonts::preview_family_name(&path);
            FontChoice {
                name,
                path,
                preview_family,
            }
        })
        .collect()
}
