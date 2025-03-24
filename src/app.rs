use std::{io::Cursor, path::PathBuf};

use crate::{
    errors::{ErrorManager, TarsierError},
    file::File,
    side_panel::ImageOperations,
    windows::WindowsData,
};
use egui::Pos2;
use image::ImageReader;
use poll_promise::Promise;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TarsierApp {
    #[serde(skip)]
    pub img: image::DynamicImage,

    #[serde(skip)]
    pub base_img: image::DynamicImage,

    #[serde(skip)]
    pub selection: Option<egui::Rect>,

    #[serde(skip)]
    pub start_selection: Pos2,

    #[serde(skip)]
    pub dropped_files: Vec<egui::DroppedFile>,

    #[serde(skip)]
    pub file_upload: Option<Promise<Result<File, TarsierError>>>,

    #[serde(skip)]
    pub is_selecting: bool,

    pub image_operations: ImageOperations,

    pub save_path: Option<PathBuf>,

    pub windows: WindowsData,

    #[serde(skip)]
    pub error_manager: ErrorManager,
}

const ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for TarsierApp {
    fn default() -> Self {
        let img = ImageReader::new(Cursor::new(ASSET))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        Self {
            base_img: img.clone(),
            img,
            selection: None,
            start_selection: Pos2::ZERO,
            is_selecting: false,
            image_operations: Default::default(),
            save_path: None,
            dropped_files: Default::default(),
            error_manager: Default::default(),
            windows: WindowsData::new(),
            file_upload: None,
        }
    }
}

impl TarsierApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl TarsierApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_save_path(&mut self) -> std::path::PathBuf {
        use rfd::FileDialog;
        let path = FileDialog::new()
            .set_directory(match &self.save_path {
                Some(path) => path.parent().unwrap(),
                None => std::path::Path::new("."),
            })
            .set_file_name(match &self.save_path {
                Some(path) => path.file_name().unwrap().to_string_lossy(),
                None => std::path::Path::new("image").to_string_lossy(),
            })
            .save_file();
        if let Some(path) = path {
            self.save_path = Some(path.clone());
            path
        } else {
            std::path::Path::new(".").to_path_buf()
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn get_save_path(&mut self) -> std::path::PathBuf {
        std::path::PathBuf::new()
    }
}

impl eframe::App for TarsierApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx);

        if self.windows.right_panel {
            self.side_panel(ctx);
        }
        self.central_panel(ctx);

        self.handle_files(ctx);

        self.windows(ctx);
    }
}
