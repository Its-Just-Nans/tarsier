//! Tarsier App
use bladvak::eframe::{
    self,
    egui::{self, Color32, Image, ImageSource, Pos2},
    CreationContext,
};
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use bladvak::{egui_extras, log};
use image::{ColorType, DynamicImage, ImageReader};
use std::{fmt::Debug, io::Cursor, path::PathBuf, sync::Arc};

use crate::side_panel::{EditMode, ImageOperations};

/// New Image settings
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewImage {
    /// is open
    pub(crate) is_open: bool,
    /// new image width
    pub(crate) width: u32,
    /// new image heigth
    pub(crate) heigth: u32,
    /// new image color type
    pub(crate) color_type: ColorType,
}

impl Default for NewImage {
    fn default() -> Self {
        Self {
            is_open: false,
            heigth: 400,
            width: 400,
            color_type: ColorType::Rgba16,
        }
    }
}

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

    /// Image texture
    #[serde(skip)]
    pub texture: Option<egui::TextureHandle>,

    /// Exif of the image
    #[serde(skip)]
    pub exif: Option<exif::Exif>,

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

    /// Image infos as windows
    pub image_info_as_window: bool,

    /// Image operations panel
    pub image_operations: ImageOperations,

    /// Path to save the image
    pub save_path: Option<PathBuf>,

    /// New image settings
    #[serde(skip)]
    pub new_image: NewImage,
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
        let (img, _) = Self::load_default_image();
        Self {
            saved_img: img.clone(),
            img,
            texture: None,
            exif: None,
            selection: None,
            cursor_op_as_window: false,
            start_selection: Pos2::ZERO,
            is_selecting: false,
            image_info_as_window: false,
            image_operations: Default::default(),
            save_path: None,
            new_image: Default::default(),
        }
    }
}

impl TarsierApp {
    /// Called once before the first frame.
    fn new_app(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        bladvak::utils::get_saved_app_state::<Self>(cc)
    }
    /// Create a new Tarsier App with an image
    /// # Errors
    /// Return error if fail to load image
    pub fn new_app_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError> {
        if args.len() > 1 {
            use image::ImageReader;
            let path = &args[1];
            let bytes = std::fs::read(path)?;
            let cursor: Cursor<&[u8]> = Cursor::new(bytes.as_ref());
            let img_reader = ImageReader::new(cursor);
            match img_reader.with_guessed_format()?.decode() {
                Ok(img) => {
                    let mut app = Self::new_app(cc);
                    let cursor_data = Cursor::new(bytes.as_ref());
                    app.update_file(img, Some(cursor_data));
                    Ok(app)
                }
                Err(e) => {
                    eprintln!("Failed to load image '{path}': {e}");
                    Err(AppError::new_with_source(
                        format!("Failed to load image '{path}'"),
                        Arc::new(e),
                    ))
                }
            }
        } else {
            Ok(TarsierApp::new_app(cc))
        }
    }

    /// Load the default image
    fn load_default_image() -> (DynamicImage, Cursor<&'static [u8]>) {
        let cursor = Cursor::new(ASSET);
        // allow unwrap_used since asset is static
        #[allow(clippy::unwrap_used)]
        let img = ImageReader::new(cursor.clone())
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        (img, cursor)
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
                    self.update_image(cropped_img);
                }
            }
        }
    }

    /// Update the image file
    pub(crate) fn update_file(&mut self, new_img: DynamicImage, opt_cursor: Option<Cursor<&[u8]>>) {
        self.saved_img = new_img.clone();
        self.update_image(new_img);
        let exifreader = exif::Reader::new();
        if let Some(bytes) = opt_cursor {
            let mut bufreader = std::io::BufReader::new(bytes);
            match exifreader.read_from_container(&mut bufreader) {
                Ok(exif) => self.exif = Some(exif),
                Err(e) => {
                    self.exif = None;
                    log::info!("Cannot get exif of image: {e}");
                }
            };
        } else {
            self.exif = None;
        }
    }

    /// Update image
    pub(crate) fn update_image(&mut self, new_img: DynamicImage) {
        self.img = new_img;
        self.updated_image();
    }

    /// Post update image
    pub(crate) fn updated_image(&mut self) {
        self.texture = None;
        self.selection = None;
    }
}

impl BladvakApp for TarsierApp {
    fn settings(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.separator();
        ui.checkbox(&mut self.cursor_op_as_window, "Cursor windows");
        ui.checkbox(&mut self.image_info_as_window, "Image info windows");
        ui.checkbox(&mut self.image_operations.is_window, "Operations windows");
        ui.separator();
        if ui.button("Default image").clicked() {
            let (img, cursor) = Self::load_default_image();
            self.update_file(img, Some(cursor));
        }
    }

    fn is_side_panel(&self) -> bool {
        !self.cursor_op_as_window || !self.image_info_as_window || !self.image_operations.is_window
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, bytes: &[u8]) -> Result<(), AppError> {
        let img_reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
        let img = match img_reader.decode() {
            Ok(img) => img,
            Err(e) => {
                return Err(AppError::new_with_source(
                    "Cannot decode image",
                    Arc::new(e),
                ))
            }
        };
        let cursor = Cursor::new(bytes);
        self.update_file(img, Some(cursor));
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_top_panel(ui, error_manager);
    }

    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_menu_file(ui, error_manager)
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager)
    }

    fn name() -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn repo_url() -> String {
        "https://github.com/Its-Just-Nans/tarsier".to_string()
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
