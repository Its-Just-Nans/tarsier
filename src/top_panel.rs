use std::{io::Cursor, path::PathBuf};

use egui::{text::LayoutJob, Color32, ImageSource, TextFormat, ThemePreference};
use poll_promise::Promise;

use crate::{errors::TarsierError, file::File, side_panel::EditMode, TarsierApp};

const RESET_ICON: ImageSource<'_> = egui::include_image!("../assets/x-circle.png");
const ROTATE_CCW_ICON: ImageSource<'_> = egui::include_image!("../assets/rotate_ccw.png");
const ROTATE_CW_ICON: ImageSource<'_> = egui::include_image!("../assets/rotate_cw.png");
const FLIP_H_ICON: ImageSource<'_> = egui::include_image!("../assets/flip_h.png");
const FLIP_V_ICON: ImageSource<'_> = egui::include_image!("../assets/flip_v.png");

impl TarsierApp {
    #[cfg(target_arch = "wasm32")]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_local(async {
            let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
            if let Some(curr_file) = file_selected {
                let buf = curr_file.read().await;
                return Ok(File {
                    path: curr_file.file_name(),
                    data: buf,
                });
            }
            // no file selected
            Err(TarsierError::new_fake(
                "Upload: no file Selected".to_string(),
            ))
        }));
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_thread("slow", move || {
            if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                // read file as string
                if let Some(path) = path_buf.to_str() {
                    let path = path.to_string();
                    let buf = std::fs::read(path.clone());
                    let buf = match buf {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("{:?}", e);
                            return Err(TarsierError::new(e.to_string()));
                        }
                    };
                    return Ok(File { path, data: buf });
                }
            }
            // no file selected
            Err(TarsierError::new_fake(
                "Upload: no file Selected".to_string(),
            ))
        }))
    }

    pub fn menu_file(&mut self, ui: &mut egui::Ui) {
        // NOTE: no File->Quit on web pages!
        ui.menu_button("File", |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            if ui.button("Open").clicked() {
                ui.close_menu();
                self.handle_file_open();
            }
            ui.menu_button("Save", |ui| {
                if ui.button("PNG").clicked() {
                    ui.close_menu();
                    let save_path = self.get_save_path();
                    self.save_image(image::ImageFormat::Png, &save_path);
                }
                if ui.button("JPEG").clicked() {
                    ui.close_menu();
                    let save_path = self.get_save_path();
                    self.save_image(image::ImageFormat::Jpeg, &save_path);
                }
                if ui.button("BMP").clicked() {
                    ui.close_menu();
                    let save_path = self.get_save_path();
                    self.save_image(image::ImageFormat::Bmp, &save_path);
                }
                if ui.button("GIF").clicked() {
                    ui.close_menu();
                    let save_path = self.get_save_path();
                    self.save_image(image::ImageFormat::Gif, &save_path);
                }
            });
            ui.menu_button("Theme", |ui| {
                let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
                ui.selectable_value(&mut theme_preference, ThemePreference::Light, "☀ Light");
                ui.selectable_value(&mut theme_preference, ThemePreference::Dark, "🌙 Dark");
                ui.selectable_value(&mut theme_preference, ThemePreference::System, "💻 System");
                ui.ctx().set_theme(theme_preference);
            });
            ui.add(
                egui::Hyperlink::from_label_and_url(
                    "Github repo",
                    "https://github.com/Its-Just-Nans/tarsier",
                )
                .open_in_new_tab(true),
            );
            egui::warn_if_debug_build(ui);
        });
    }

    pub fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                self.menu_file(ui);
                ui.menu_button("Windows", |ui| {
                    ui.checkbox(&mut self.windows.selection_window, "Selection");
                    ui.checkbox(&mut self.windows.right_panel, "Right Panel");
                    ui.checkbox(&mut self.windows.error_window, "Error Panel");
                });
                ui.separator();
                if ui
                    .add(egui::Button::image(RESET_ICON))
                    .on_hover_text("Reset the image")
                    .clicked()
                {
                    self.img = self.base_img.clone();
                    self.selection = None;
                }
                ui.separator();

                if ui
                    .add(egui::Button::image(ROTATE_CW_ICON))
                    .on_hover_text("Rotate 90 degrees clockwise")
                    .clicked()
                {
                    self.img = self.img.rotate90();
                }
                if ui
                    .add(egui::Button::image(ROTATE_CCW_ICON))
                    .on_hover_text("Rotate 90 degrees counter-clockwise")
                    .clicked()
                {
                    self.img = self.img.rotate270();
                }
                if ui
                    .add(egui::Button::image(FLIP_H_ICON))
                    .on_hover_text("Flip horizontally")
                    .clicked()
                {
                    self.img = self.img.fliph();
                }
                if ui
                    .add(egui::Button::image(FLIP_V_ICON))
                    .on_hover_text("Flip vertically")
                    .clicked()
                {
                    self.img = self.img.flipv();
                }
                ui.separator();
                let (default_color, _strong_color) = if ui.visuals().dark_mode {
                    (Color32::LIGHT_GRAY, Color32::WHITE)
                } else {
                    (Color32::DARK_GRAY, Color32::BLACK)
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
                        background: Color32::from_rgb(144, 209, 255),
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
                        ui.close_menu();
                        if self.image_operations.mode == EditMode::Drawing {
                            self.selection = None;
                        }
                    }
                });
                ui.separator();
                if let Some(selection) = self.selection {
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
            });
        });
    }
}

impl TarsierApp {
    fn save_image(&mut self, format: image::ImageFormat, path_file: &PathBuf) {
        let mut bytes: Vec<u8> = Vec::new();
        let write_res = self.img.write_to(&mut Cursor::new(&mut bytes), format);
        if self.error_manager.handle_error(write_res).is_some() {
            self.save_file(&bytes, path_file);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_file(&mut self, data: &[u8], path_file: &PathBuf) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(path_file).unwrap();
        file.write_all(data).unwrap();
    }

    // TODO handle unwraps
    #[cfg(target_arch = "wasm32")]
    fn save_file(&mut self, data: &[u8], path_file: &PathBuf) {
        // create blob
        use eframe::wasm_bindgen::JsCast;
        use js_sys::Array;

        log::info!("Saving file to {:?}", path_file);
        let filename = match path_file.file_name() {
            Some(name) => name.to_str().unwrap(),
            None => "image.png",
        };

        let array_data = Array::new();
        array_data.push(&js_sys::Uint8Array::from(data));
        let blob = web_sys::Blob::new_with_u8_array_sequence(&array_data).unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        // create link
        let document = web_sys::window().unwrap().document().unwrap();
        let a = document.create_element("a").unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", filename).unwrap();
        // click link
        a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        // revoke url
        web_sys::Url::revoke_object_url(&url).unwrap();
    }
}
