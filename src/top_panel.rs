//! Top panel
use std::{io::Cursor, path::PathBuf};

use bladvak::errors::ErrorManager;
use egui::{text::LayoutJob, Color32, ColorImage, Image, ImageSource, TextFormat};
use image::ImageFormat;

use crate::{side_panel::EditMode, TarsierApp};

impl TarsierApp {
    /// Reset icon
    const RESET_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_x-circle.png");
    /// Rotate icon
    const ROTATE_CCW_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_rotate_ccw.png");
    /// Rotate icon inverse
    const ROTATE_CW_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_rotate_cw.png");
    /// Flip horizontal icon
    const FLIP_H_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_flip_h.png");
    /// Flip vertical icon
    const FLIP_V_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_flip_v.png");

    /// Show the file menu
    pub(crate) fn app_menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.menu_button("Save", |ui| {
            if ui.button("PNG").clicked() {
                ui.close();
                let save_path = self.get_save_path(ImageFormat::Png);
                if let Some(save_path) = error_manager.handle_error(save_path) {
                    let res = self.save_image(ImageFormat::Png, &save_path);
                    error_manager.handle_error(res);
                }
            }
            if ui.button("JPEG").clicked() {
                ui.close();
                let save_path = self.get_save_path(ImageFormat::Jpeg);
                if let Some(save_path) = error_manager.handle_error(save_path) {
                    let res = self.save_image(ImageFormat::Jpeg, &save_path);
                    error_manager.handle_error(res);
                }
            }
            if ui.button("BMP").clicked() {
                ui.close();
                let save_path = self.get_save_path(ImageFormat::Bmp);
                if let Some(save_path) = error_manager.handle_error(save_path) {
                    let res = self.save_image(ImageFormat::Bmp, &save_path);
                    error_manager.handle_error(res);
                }
            }
            if ui.button("GIF").clicked() {
                ui.close();
                let save_path = self.get_save_path(ImageFormat::Gif);
                if let Some(save_path) = error_manager.handle_error(save_path) {
                    let res = self.save_image(ImageFormat::Gif, &save_path);
                    error_manager.handle_error(res);
                }
            }
        });
    }

    /// Show the top panel
    pub(crate) fn app_top_panel(&mut self, ui: &mut egui::Ui) {
        let is_dark_theme = ui.ctx().style().visuals.dark_mode;
        ui.separator();
        let ico_image = Image::new(Self::RESET_ICON);
        if ui
            .add(egui::Button::image(if is_dark_theme {
                ico_image
            } else {
                ico_image.tint(Color32::BLACK)
            }))
            .on_hover_text("Reset the image")
            .clicked()
        {
            self.img = self.saved_img.clone();
            self.selection = None;
        }
        let ico_image = Image::new(Self::ROTATE_CW_ICON);
        ui.separator();
        if ui
            .add(egui::Button::image(if is_dark_theme {
                ico_image
            } else {
                ico_image.tint(Color32::BLACK)
            }))
            .on_hover_text("Rotate 90 degrees clockwise")
            .clicked()
        {
            self.img = self.img.rotate90();
        }

        let ico_image = Image::new(Self::ROTATE_CCW_ICON);
        if ui
            .add(egui::Button::image(if is_dark_theme {
                ico_image
            } else {
                ico_image.tint(Color32::BLACK)
            }))
            .on_hover_text("Rotate 90 degrees counter-clockwise")
            .clicked()
        {
            self.img = self.img.rotate270();
        }

        let ico_image = Image::new(Self::FLIP_H_ICON);
        if ui
            .add(egui::Button::image(if is_dark_theme {
                ico_image
            } else {
                ico_image.tint(Color32::BLACK)
            }))
            .on_hover_text("Flip horizontally")
            .clicked()
        {
            self.img = self.img.fliph();
        }
        let ico_image = Image::new(Self::FLIP_V_ICON);
        if ui
            .add(egui::Button::image(if is_dark_theme {
                ico_image
            } else {
                ico_image.tint(Color32::BLACK)
            }))
            .on_hover_text("Flip vertically")
            .clicked()
        {
            self.img = self.img.flipv();
        }
        ui.separator();
        let (default_color, background_color) = if ui.visuals().dark_mode {
            (Color32::LIGHT_GRAY, Color32::DARK_BLUE)
        } else {
            (Color32::DARK_GRAY, Color32::LIGHT_BLUE)
        };
        let mut job = LayoutJob::default();
        job.append(
            "Cursor ",
            0.0,
            TextFormat {
                color: default_color,
                ..Default::default()
            },
        );
        job.append(
            &format!("{}", self.image_operations.mode),
            0.0,
            TextFormat {
                color: default_color,
                background: background_color,
                ..Default::default()
            },
        );
        ui.menu_button(job, |ui| {
            let previous_state = self.image_operations.mode;
            ui.selectable_value(
                &mut self.image_operations.mode,
                EditMode::Selection,
                "Selection",
            );
            ui.selectable_value(
                &mut self.image_operations.mode,
                EditMode::Drawing,
                "Drawing",
            );
            if self.image_operations.mode != previous_state {
                ui.close();
                if self.image_operations.mode == EditMode::Drawing {
                    self.selection = None;
                }
            }
        });
        if let Some(selection) = self.selection {
            ui.separator();
            if ui
                .label(format!(
                    "Selection: {}x{}",
                    (selection.max.x - selection.min.x).abs() as u32,
                    (selection.max.y - selection.min.y).abs() as u32
                ))
                .on_hover_text("Click to clear selection")
                .clicked()
            {
                self.selection = None;
            }
        }
        ui.separator();
        if ui.button("Copy").clicked() {
            let size = [self.img.width() as _, self.img.height() as _];
            let rgb_img = self.img.to_rgba8();
            let pixels = rgb_img.as_flat_samples();
            let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            ui.ctx().copy_image(color_image);
        }
    }
}

impl TarsierApp {
    /// Save the current image
    fn save_image(&mut self, format: ImageFormat, path_file: &PathBuf) -> Result<(), String> {
        let mut bytes: Vec<u8> = Vec::new();
        self.img
            .write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|e| format!("Cannot write image: {e}"))?;
        self.save_file(&bytes, path_file)
    }

    /// Save the data to a file
    #[cfg(not(target_arch = "wasm32"))]
    fn save_file(&mut self, data: &[u8], path_file: &PathBuf) -> Result<(), String> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(path_file).map_err(|e| format!("Cannot create file: {e}"))?;
        file.write_all(data)
            .map_err(|e| format!("Cannot write file: {e}"))
    }

    #[cfg(target_arch = "wasm32")]
    fn save_file(&mut self, data: &[u8], path_file: &PathBuf) -> Result<(), String> {
        // create blob
        use eframe::wasm_bindgen::JsCast;
        use js_sys::Array;

        log::info!("Saving file to {:?}", path_file);
        let filename = match path_file.file_name() {
            Some(name) => name.to_str().ok_or("Cannot get filename")?,
            None => "image.png",
        };

        let array_data = Array::new();
        array_data.push(&js_sys::Uint8Array::from(data));
        let blob = web_sys::Blob::new_with_u8_array_sequence(&array_data)
            .map_err(|_| "Cannot create image data")?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Cannot create image url data")?;
        // create link
        let document = web_sys::window()
            .ok_or("Cannot get the website window")?
            .document()
            .ok_or("Cannot get the website document")?;
        let a = document
            .create_element("a")
            .map_err(|_| "Cannot create <a> element")?;
        a.set_attribute("href", &url)
            .map_err(|_| "Cannot create add href attribute")?;
        a.set_attribute("download", filename)
            .map_err(|_| "Cannot create add download attribute")?;

        // click link
        a.dyn_ref::<web_sys::HtmlElement>()
            .ok_or("Cannot simulate click")?
            .click();
        // revoke url
        web_sys::Url::revoke_object_url(&url)
            .map_err(|_| "Cannot remove object url with revoke_object_url".into())
    }
}
