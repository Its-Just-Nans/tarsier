//! Tarsier App
use bladvak::{
    File,
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use bladvak::{
    eframe::{
        CreationContext,
        egui::{self, Color32, Image, ImageSource},
    },
    utils::grid::Grid,
    utils::is_native,
};
use bladvak::{egui_extras, log, utils::Documents};
use image::{ColorType, DynamicImage, ImageReader};
use std::{fmt::Debug, io::Cursor, path::PathBuf, sync::Arc};

use crate::{
    document::Document,
    edit_mode::{EditMode, Mode},
    panels::{CursorInfo, ImageInfo, ImageOperationsPanel},
    side_panel::ImageOperations,
};

/// New Image settings
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NewImage {
    /// is open
    pub(crate) is_open: bool,
    /// new image width
    pub(crate) width: u32,
    /// new image height
    pub(crate) height: u32,
    /// new image color type
    pub(crate) color_type: ColorType,
}

impl Default for NewImage {
    fn default() -> Self {
        Self {
            is_open: false,
            height: 1024,
            width: 1024,
            color_type: ColorType::Rgba16,
        }
    }
}

/// App settings
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct AppSettings {
    /// remove selection
    pub(crate) remove_selection_after_op: bool,
    /// Image infos as windows
    pub(crate) image_info_as_window: bool,
    /// New image settings
    #[serde(skip)]
    pub(crate) new_image: NewImage,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct TarsierApp {
    /// list of documents
    #[serde(skip)]
    pub(crate) documents: Documents<Document>,
    /// Editor mode
    pub(crate) mode: Mode,
    /// settings
    pub(crate) settings: AppSettings,
    /// Image operations panel
    pub(crate) image_operations: ImageOperations,
    /// Grid options
    pub(crate) grid: Grid,
}

impl Default for TarsierApp {
    fn default() -> Self {
        let (img, _) = TarsierApp::load_default_image();
        let document = Document {
            saved_img: img.clone(),
            img,
            filename: PathBuf::from("tarsier.png"),
            ..Default::default()
        };
        let mut documents = Documents::default();
        documents.push(document);
        Self {
            documents,
            mode: Mode::default(),
            image_operations: ImageOperations::default(),
            settings: AppSettings::default(),
            grid: Grid::default(),
        }
    }
}

/// default image
const ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl TarsierApp {
    /// Load the default image
    pub(crate) fn load_default_image() -> (DynamicImage, Cursor<&'static [u8]>) {
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
        let Some(document) = self.documents.get_current_doc_mut() else {
            ui.label("No document");
            return;
        };
        match self.mode.current {
            EditMode::Cursor => {
                ui.label("Doing nothing");
            }
            EditMode::Drawing => {
                let max_radius = document.img.width().max(document.img.height());
                self.mode.drawing.show(ui, max_radius);
            }
            EditMode::Selection => {
                let mut reset_rect = false;
                match &mut document.selection.rectangle {
                    Some(rect) => {
                        if ui
                            .label(format!(
                                "Selection: {:.0}x{:.0}",
                                (rect.max.x - rect.min.x).abs(),
                                (rect.max.y - rect.min.y).abs()
                            ))
                            .on_hover_text("Click to clear selection")
                            .clicked()
                        {
                            reset_rect = true;
                        }
                        let width = rect.width().abs();
                        let height = rect.height().abs();
                        ui.label(format!("Width: {width:.0}"));
                        ui.label(format!("Height: {height:.0}"));
                        let right = rect.right();
                        let bottom = rect.bottom();
                        let left = rect.left();
                        let top = rect.top();
                        #[allow(clippy::cast_precision_loss)]
                        ui.horizontal(|ui| {
                            ui.label("Min: ");
                            ui.add(egui::DragValue::new(rect.left_mut()).range(0.0..=right));
                            ui.add(egui::DragValue::new(rect.top_mut()).range(0.0..=bottom));
                        });
                        #[allow(clippy::cast_precision_loss)]
                        ui.horizontal(|ui| {
                            ui.label("Max: ");
                            ui.add(
                                egui::DragValue::new(rect.right_mut())
                                    .range(left..=(document.img.width() as f32)),
                            );
                            ui.add(
                                egui::DragValue::new(rect.bottom_mut())
                                    .range(top..=(document.img.height() as f32)),
                            );
                        });
                    }
                    None => {
                        ui.label("No selection");
                    }
                }
                if reset_rect {
                    document.selection.rectangle = None;
                }
                if let Some(selection) = document.selection.rectangle {
                    let icon_image = Image::new(Self::CROP_ICON);
                    let icon = if ui.ctx().global_style().visuals.dark_mode {
                        icon_image
                    } else {
                        icon_image.tint(Color32::BLACK)
                    };
                    #[allow(clippy::cast_sign_loss)]
                    #[allow(clippy::cast_possible_truncation)]
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
                        let cropped_img =
                            document
                                .img
                                .crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
                        self.update_image(cropped_img);
                        if let Some(document) = self.documents.get_current_doc_mut() {
                            document.selection.rectangle = None;
                        }
                    }
                }
            }
        }
    }

    /// create a new
    pub(crate) fn new_file(
        &mut self,
        filename: PathBuf,
        new_img: DynamicImage,
        opt_cursor: Option<Cursor<&[u8]>>,
    ) {
        let exif = if let Some(bytes) = opt_cursor {
            let mut bufreader = std::io::BufReader::new(bytes);
            exif::Reader::new().read_from_container(&mut bufreader).ok()
        } else {
            None
        };
        let new_document = Document {
            saved_img: new_img.clone(),
            img: new_img,
            exif,
            filename,
            ..Default::default()
        };
        self.documents.push(new_document);
    }

    /// Update image
    pub(crate) fn update_image(&mut self, new_img: DynamicImage) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        document.img = new_img;
        self.updated_image();
    }

    /// Post update image
    pub(crate) fn updated_image(&mut self) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        document.texture = None;
        if self.settings.remove_selection_after_op {
            document.selection.rectangle = None;
        }
    }
}

impl BladvakApp<'_> for TarsierApp {
    fn side_panel(
        &mut self,
        ui: &mut egui::Ui,
        func_ui: impl FnOnce(&mut egui::Ui, &mut TarsierApp),
    ) {
        egui::Frame::central_panel(&ui.ctx().global_style())
            .show(ui, |panel_ui| func_ui(panel_ui, self));
    }

    fn panel_list(&self) -> Vec<Box<dyn bladvak::app::BladvakPanel<App = Self>>> {
        vec![
            Box::new(ImageInfo),
            Box::new(ImageOperationsPanel),
            Box::new(CursorInfo),
        ]
    }

    fn is_side_panel(&self) -> bool {
        self.documents.is_some()
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        let img_reader = ImageReader::new(Cursor::new(&file.data)).with_guessed_format()?;
        let img = match img_reader.decode() {
            Ok(img) => img,
            Err(e) => {
                return Err(AppError::new_with_source(
                    "Cannot decode image",
                    Arc::new(e),
                ));
            }
        };
        let cursor = Cursor::new(file.data.as_ref());
        self.new_file(PathBuf::from("dropped.png"), img, Some(cursor));
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_top_panel(ui, error_manager);
    }

    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_menu_file(ui, error_manager);
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
        self.show_new_image_modal(ui);
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

    fn icon() -> &'static [u8] {
        &include_bytes!("../assets/icon-256.png")[..]
    }

    fn try_new_with_args(
        saved_state: Self,
        cc: &CreationContext<'_>,
        args: &[String],
        _error_manager: &mut ErrorManager,
    ) -> Result<Self, AppError> {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if is_native() && args.len() > 1 {
            use std::fs;
            let mut app = saved_state;
            app.documents.clear();
            for one_path in &args[1..] {
                let absolute_path = fs::canonicalize(one_path)
                    .map_err(|e| format!("Unable to canonicalize path '{one_path}': {e}"))?;
                let bytes = fs::read(&absolute_path)?;
                let cursor: Cursor<&[u8]> = Cursor::new(bytes.as_ref());
                let img_reader = ImageReader::new(cursor);
                match img_reader.with_guessed_format()?.decode() {
                    Ok(img) => {
                        let cursor_data = Cursor::new(bytes.as_ref());
                        app.new_file(absolute_path, img, Some(cursor_data));
                    }
                    Err(e) => {
                        log::error!("Failed to load image '{}': {e}", absolute_path.display());
                        return Err(AppError::new_with_source(
                            format!("Failed to load image '{}'", absolute_path.display()),
                            Arc::new(e),
                        ));
                    }
                }
            }
            Ok(app)
        } else {
            Ok(saved_state)
        }
    }
}
