//! Top panel
use bladvak::eframe::egui::{self, Color32, TextFormat, text::LayoutJob};
use bladvak::errors::ErrorManager;
use image::ImageFormat;
use std::io::Cursor;
use std::path::Path;

use crate::document::Document;
use crate::{TarsierApp, edit_mode::EditMode};

impl TarsierApp {
    /// Show the file menu
    pub(crate) fn app_menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if ui.button("New").clicked() {
            self.settings.new_image.is_open = true;
            ui.close();
        }
        self.menu_clipboard(ui, error_manager);
        if self.documents.is_some() {
            ui.menu_button("Save", |ui| {
                if ui.button("PNG").clicked() {
                    ui.close();
                    self.save_image_with_format(
                        ImageFormat::Png.extensions_str()[0],
                        ImageFormat::Png,
                        error_manager,
                    );
                }
                if ui.button("JPEG").clicked() {
                    ui.close();
                    self.save_image_with_format(
                        ImageFormat::Jpeg.extensions_str()[0],
                        ImageFormat::Jpeg,
                        error_manager,
                    );
                }
                if ui.button("BMP").clicked() {
                    ui.close();
                    self.save_image_with_format(
                        ImageFormat::Bmp.extensions_str()[0],
                        ImageFormat::Bmp,
                        error_manager,
                    );
                }
                if ui.button("GIF").clicked() {
                    ui.close();
                    self.save_image_with_format(
                        ImageFormat::Gif.extensions_str()[0],
                        ImageFormat::Gif,
                        error_manager,
                    );
                }
            });
        }
    }

    /// Show the clipboard menu
    pub fn menu_clipboard(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        let is_document = self.documents.is_some();
        ui.menu_button("Clipboard", |ui| {
            if is_document
                && ui.button("Copy").clicked()
                && let Some(document) = self.documents.get_current_doc_mut()
                && let Err(e) = bladvak::utils::set_image_in_clipboard(
                    ui.ctx(),
                    document.img.width() as usize,
                    document.img.height() as usize,
                    document.img.to_rgba8().as_flat_samples().as_slice(),
                )
            {
                error_manager.add_error(e);
            }

            if ui.button("Paste").clicked()
                && let Err(err) = self.clipboard.launch_get_image()
            {
                error_manager.add_error(err);
            }
        });
    }

    /// Save the current image with the specified format
    pub(crate) fn save_image_with_format(
        &mut self,
        extension: &str,
        image_format: ImageFormat,
        error_manager: &mut ErrorManager,
    ) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            error_manager.add_error("No document to save");
            return;
        };
        let current_save_path =
            if document.filename.extension().and_then(|e| e.to_str()) == Some(extension) {
                document.filename.clone()
            } else {
                document.filename.with_extension(extension)
            };
        let save_path = bladvak::utils::get_save_path(Some(&current_save_path));
        match save_path {
            Ok(save_p) => {
                if let Some(path_to_save) = save_p {
                    document.filename.clone_from(&path_to_save);
                    if let Err(err) = document.save_image(image_format, &path_to_save) {
                        error_manager.add_error(err);
                    }
                }
            }
            Err(e) => {
                error_manager.add_error(e);
            }
        }
    }

    /// Show the top panel
    pub(crate) fn app_top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        match self.clipboard.image(ui.ctx()) {
            Some(Ok((rgba_data, width, height))) => {
                #[allow(clippy::cast_possible_truncation)]
                if let Some(buffer) =
                    image::ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
                {
                    use std::path::PathBuf;

                    let img = image::DynamicImage::ImageRgba8(buffer);
                    self.new_file(PathBuf::from("pasted.png"), img, None);
                } else {
                    error_manager.add_error("Invalid image data from clipboard".to_string());
                }
            }
            Some(Err(err)) => {
                error_manager.add_error(err);
            }
            None => {}
        }
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        ui.separator();
        let (default_color, background_color) = if ui.visuals().dark_mode {
            (Color32::LIGHT_GRAY, Color32::DARK_BLUE)
        } else {
            (Color32::DARK_GRAY, Color32::LIGHT_BLUE)
        };
        let mut job = LayoutJob::default();
        job.append(
            "Mode ",
            0.0,
            TextFormat {
                color: default_color,
                ..Default::default()
            },
        );
        job.append(
            &format!("{}", self.mode.current),
            0.0,
            TextFormat {
                color: default_color,
                background: background_color,
                ..Default::default()
            },
        );
        ui.menu_button(job, |ui| {
            let previous_state = self.mode.current;
            ui.selectable_value(
                &mut self.mode.current,
                EditMode::Cursor,
                EditMode::Cursor.to_string(),
            );
            ui.selectable_value(
                &mut self.mode.current,
                EditMode::Selection,
                EditMode::Selection.to_string(),
            );
            ui.selectable_value(
                &mut self.mode.current,
                EditMode::Drawing,
                EditMode::Drawing.to_string(),
            );
            ui.selectable_value(
                &mut self.mode.current,
                EditMode::ColorSelection,
                EditMode::ColorSelection.to_string(),
            );
            if self.mode.current != previous_state {
                ui.close();
                if self.mode.current == EditMode::Cursor {
                    document.selection.rectangle = None;
                }
            }
        });
        ui.separator();
        self.documents.show_file_list(ui);
    }
}

impl Document {
    /// Save the current image
    fn save_image(&mut self, format: ImageFormat, path_file: &Path) -> Result<(), String> {
        let mut bytes: Vec<u8> = Vec::new();
        self.img
            .write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|e| format!("Cannot write image: {e}"))?;
        bladvak::utils::save_file(&bytes, path_file)
    }
}
