//! Tarsier App
use std::{io::Cursor, path::PathBuf};

use crate::{
    errors::ErrorManager, file_handler::FileHandler, settings::Settings,
    side_panel::ImageOperations, windows::WindowsManager,
};
use egui::Pos2;
use image::{DynamicImage, ImageReader};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TarsierApp {
    /// Current image
    #[serde(skip)]
    pub img: image::DynamicImage,

    /// Save of the image
    #[serde(skip)]
    pub base_img: image::DynamicImage,

    /// Selection rectangle
    #[serde(skip)]
    pub selection: Option<egui::Rect>,

    /// Start selection position
    #[serde(skip)]
    pub start_selection: Pos2,

    /// Is currently selecting
    #[serde(skip)]
    pub is_selecting: bool,

    /// Image operations panel
    pub image_operations: ImageOperations,

    /// Path to save the image
    pub save_path: Option<PathBuf>,

    /// Windows manager
    pub windows: WindowsManager,

    /// Error_manager
    #[serde(skip)]
    pub error_manager: ErrorManager,

    /// Settings Ui
    pub settings: Settings,

    /// File Handler
    pub file_handler: FileHandler,
}

const ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for TarsierApp {
    fn default() -> Self {
        let img = Self::load_default_image();
        Self {
            base_img: img.clone(),
            img,
            selection: None,
            start_selection: Pos2::ZERO,
            is_selecting: false,
            image_operations: Default::default(),
            save_path: None,
            error_manager: Default::default(),
            windows: Default::default(),
            settings: Default::default(),
            file_handler: Default::default(),
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

    /// Load the default image
    fn load_default_image() -> DynamicImage {
        ImageReader::new(Cursor::new(ASSET))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
    }

    /// Get the save path
    /// # Errors
    /// Failed if the input is wrong
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_save_path(&mut self) -> Result<std::path::PathBuf, String> {
        use rfd::FileDialog;
        let path = FileDialog::new()
            .set_directory(match &self.save_path {
                Some(path) => path.parent().ok_or("Cannot get parent in the path")?,
                None => std::path::Path::new("."),
            })
            .set_file_name(match &self.save_path {
                Some(path) => path
                    .file_name()
                    .ok_or("Cannot get file name")?
                    .to_string_lossy(),
                None => std::path::Path::new("image").to_string_lossy(),
            })
            .save_file();
        let res = if let Some(path) = path {
            self.save_path = Some(path.clone());
            path
        } else {
            std::path::Path::new(".").to_path_buf()
        };
        Ok(res)
    }
    /// Get a new path
    /// # Errors
    /// No error in wasm
    #[cfg(target_arch = "wasm32")]
    pub fn get_save_path(&mut self) -> Result<std::path::PathBuf, String> {
        Ok(std::path::PathBuf::new())
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

        match self.file_handler.handle_files(ctx) {
            Ok(Some(img)) => {
                self.base_img = img.clone();
                self.img = img;
                self.selection = None;
            }
            Ok(None) => {}
            Err(err) => {
                self.error_manager.add_error(err);
            }
        }

        self.windows(ctx);
        self.error_manager.show(ctx);
        self.settings.show(ctx, |ui| {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("{} settings", "Windows"));
            });
            ui.checkbox(&mut self.windows.selection_window, "Selection");
            ui.checkbox(&mut self.windows.right_panel, "Right Panel");

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("{} settings", self.error_manager.title()));
                ui.button("⟳").clicked().then(|| {
                    self.error_manager = Default::default();
                });
            });
            self.error_manager.show_settings(ui);

            ui.separator();
            if ui.button("Default image").clicked() {
                self.base_img = Self::load_default_image();
                self.img = Self::load_default_image();
            }
        });
    }
}
