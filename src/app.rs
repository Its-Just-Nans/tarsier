//! Tarsier App
use std::{fmt::Debug, io::Cursor, path::PathBuf, sync::Arc};

use crate::side_panel::{EditMode, ImageOperations};
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use eframe::CreationContext;
use egui::{Color32, Image, ImageSource, Pos2};
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
    pub saved_img: image::DynamicImage,

    /// Selection rectangle
    #[serde(skip)]
    pub selection: Option<egui::Rect>,

    /// Start selection position
    #[serde(skip)]
    pub start_selection: Pos2,

    /// Is currently selecting
    #[serde(skip)]
    pub is_selecting: bool,

    /// Selection as windows
    pub cursor_op_as_window: bool,

    /// Operations as windows
    pub operations_as_window: bool,

    /// Image infos as windows
    pub image_info_as_window: bool,

    /// Image operations panel
    pub image_operations: ImageOperations,

    /// Path to save the image
    pub save_path: Option<PathBuf>,
}

impl Debug for TarsierApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_fmt = f.debug_struct("TarsierApp");
        debug_fmt.finish()
    }
}

/// default image
const ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for TarsierApp {
    fn default() -> Self {
        let img = Self::load_default_image();
        Self {
            saved_img: img.clone(),
            img,
            selection: None,
            cursor_op_as_window: false,
            start_selection: Pos2::ZERO,
            is_selecting: false,
            operations_as_window: false,
            image_info_as_window: false,
            image_operations: Default::default(),
            save_path: None,
        }
    }
}

impl TarsierApp {
    /// Called once before the first frame.
    fn new_app(cc: &eframe::CreationContext<'_>) -> Self {
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
    /// Create a new Tarsier App with an image
    /// # Errors
    /// Return error if fail to load image
    pub fn new_app_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError> {
        if args.len() > 1 {
            use image::ImageReader;
            let path = &args[1];
            match ImageReader::open(path) {
                Ok(reader) => match reader.decode() {
                    Ok(img) => {
                        let mut app = Self::new_app(cc);
                        app.saved_img = img.clone();
                        app.img = img;
                        Ok(app)
                    }
                    Err(e) => {
                        eprintln!("Failed to load image '{path}': {e}");
                        Err(AppError::new(format!("Failed to load image '{path}': {e}")))
                    }
                },
                Err(e) => {
                    eprintln!("Failed to load image '{path}': {e}");
                    Err(AppError::new(format!("Failed to load image '{path}': {e}")))
                }
            }
        } else {
            Ok(TarsierApp::new_app(cc))
        }
    }

    /// Load the default image
    fn load_default_image() -> DynamicImage {
        ImageReader::new(Cursor::new(ASSET))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
    }

    /// Crop icon
    const CROP_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_crop.png");

    /// Cursor ui
    pub(crate) fn cursor_ui(&mut self, ui: &mut egui::Ui) {
        if self.image_operations.mode == EditMode::Drawing {
            self.button_drawing(ui);
        } else {
            match self.selection {
                Some(rect) => {
                    let width = rect.width().abs() as u32;
                    let height = rect.height().abs() as u32;
                    ui.label(format!("Width: {width}"));
                    ui.label(format!("Height: {height}"));
                    ui.label(format!("Min: {:?}", rect.left_top()));
                    ui.label(format!("Max: {:?}", rect.right_bottom()));
                }
                None => {
                    ui.label("No selection");
                }
            }
            if let Some(selection) = self.selection {
                let icon_image = Image::new(Self::CROP_ICON);
                let icon = if ui.ctx().style().visuals.dark_mode {
                    icon_image
                } else {
                    icon_image.tint(Color32::BLACK)
                };
                if ui
                    .add(egui::Button::image_and_text(icon, "Crop"))
                    .on_hover_text("Crop the image")
                    .clicked()
                {
                    let min_pos = selection.min;
                    let max_pos = selection.max;
                    let min_x = min_pos.x as u32;
                    let min_y = min_pos.y as u32;
                    let max_x = max_pos.x as u32;
                    let max_y = max_pos.y as u32;
                    let cropped_img = self
                        .img
                        .crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
                    self.img = cropped_img;
                    self.selection = None;
                }
            }
        }
    }
}

impl BladvakApp for TarsierApp {
    fn settings(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.separator();
        ui.checkbox(&mut self.cursor_op_as_window, "Cursor windows");
        ui.checkbox(&mut self.image_info_as_window, "Image info windows");
        ui.checkbox(&mut self.operations_as_window, "Operations windows");
        ui.separator();
        if ui.button("Default image").clicked() {
            self.saved_img = Self::load_default_image();
            self.img = Self::load_default_image();
            self.selection = None;
        }
    }

    fn is_side_panel(&self) -> bool {
        !self.cursor_op_as_window || !self.image_info_as_window || !self.operations_as_window
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, bytes: &[u8]) -> Result<(), AppError> {
        let img_reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
        let img = match img_reader.decode() {
            Ok(img) => img,
            Err(e) => return Err(AppError::new_with_source(Arc::new(e))),
        };
        self.saved_img = img.clone();
        self.img = img;
        self.selection = None;
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        self.app_top_panel(ui)
    }

    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_menu_file(ui, error_manager)
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager)
    }

    fn name(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn side_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        egui::Frame::central_panel(&ui.ctx().style()).show(ui, |parent_ui| {
            self.app_side_panel(parent_ui, error_manager)
        });
    }

    fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, AppError> {
        Ok(TarsierApp::new_app(cc))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn new_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError> {
        TarsierApp::new_app_with_args(cc, args)
    }
}
