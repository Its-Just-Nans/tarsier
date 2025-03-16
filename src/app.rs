use std::{io::Cursor, sync::Arc};

use egui::{load::SizedTexture, ColorImage, Image, ImageData, Pos2, Sense, TextureOptions};
use image::ImageReader;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TarsierApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    scene_rect: egui::Rect,

    #[serde(skip)]
    img: image::DynamicImage,

    #[serde(skip)]
    base_img: image::DynamicImage,

    #[serde(skip)]
    selection: Option<[Pos2; 2]>,

    #[serde(skip)]
    is_selecting: bool,
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
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            scene_rect: egui::Rect::NAN,
            base_img: img.clone(),
            img,
            selection: None,
            is_selecting: false,
        }
    }
}

impl TarsierApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn save_image(img: &image::DynamicImage, format: image::ImageFormat, filename: &str) {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format).unwrap();
    save_file(&bytes, filename);
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file(data: &[u8], filename: &str) {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(filename).unwrap();
    file.write_all(data).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn save_file(data: &[u8], filename: &str) {
    // create blob
    use eframe::wasm_bindgen::JsCast;
    use js_sys::Array;

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

impl eframe::App for TarsierApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }
                ui.menu_button("Options", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                ui.separator();
                if ui
                    .add_enabled(self.selection.is_some(), egui::Button::new("Crop"))
                    .on_hover_text("Crop the image")
                    .clicked()
                {
                    if let Some(selection) = self.selection {
                        let min_pos = selection[0];
                        let max_pos = selection[1];
                        let min_x = min_pos.x as u32;
                        let min_y = min_pos.y as u32;
                        let max_x = max_pos.x as u32;
                        let max_y = max_pos.y as u32;
                        let cropped_img = self.img.crop(min_x, min_y, max_x - min_x, max_y - min_y);
                        self.img = cropped_img;
                        self.selection = None;
                    }
                }
                if ui.button("reset").clicked() {
                    self.img = self.base_img.clone();
                    self.selection = None;
                }
                if ui.button("rotate").clicked() {
                    self.img = self.img.rotate90();
                }
                if ui.button("rotate inv").clicked() {
                    self.img = self.img.rotate270();
                }
                if ui.button("flip h").clicked() {
                    self.img = self.img.fliph();
                }
                if ui.button("flip v").clicked() {
                    self.img = self.img.flipv();
                }
                ui.menu_button("Save", |ui| {
                    if ui.button("PNG").clicked() {
                        save_image(&self.img, image::ImageFormat::Png, "image.png");
                    }
                    if ui.button("JPEG").clicked() {
                        save_image(&self.img, image::ImageFormat::Jpeg, "image.jpg");
                    }
                    if ui.button("BMP").clicked() {
                        save_image(&self.img, image::ImageFormat::Bmp, "image.bmp");
                    }
                    if ui.button("GIF").clicked() {
                        save_image(&self.img, image::ImageFormat::Gif, "image.gif");
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            let size = [self.img.width() as _, self.img.height() as _];
            let image_buffer = self.img.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            let svg_texture = ctx.load_texture(
                "svg",
                ImageData::Color(Arc::new(image)),
                TextureOptions::default(),
            );

            let sized_texture = SizedTexture::from_handle(&svg_texture);

            let response = ui.add(Image::new(sized_texture).sense(Sense::click_and_drag()));
            let img_pos_rect = response.rect;

            let painter = ui.painter();
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if !self.is_selecting {
                        self.selection = None;
                    }
                    self.selection = match self.selection {
                        Some(rect) => Some([rect[0], pos]),
                        None => Some([pos, pos]),
                    };
                    self.is_selecting = true;
                }
            } else {
                self.is_selecting = false;
            }
            if response.clicked() {
                self.selection = None;
            }
            if let Some(selection) = self.selection {
                let min_pos = img_pos_rect.min;
                let rect_selection = egui::Rect::from_two_pos(selection[0], selection[1]);
                painter.rect_stroke(
                    rect_selection,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::BLACK),
                    egui::StrokeKind::Outside,
                );
                // rect above selection
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        Pos2::new(min_pos.x, min_pos.y),
                        Pos2::new(size[0] as f32 + min_pos.x, rect_selection.min.y),
                    ),
                    0.0,
                    egui::Color32::from_black_alpha(50),
                );
                // rect below selection
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        Pos2::new(min_pos.x, rect_selection.max.y),
                        Pos2::new(size[0] as f32 + min_pos.x, size[1] as f32 + min_pos.y),
                    ),
                    0.0,
                    egui::Color32::from_black_alpha(50),
                );
                // rect left of selection
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        Pos2::new(min_pos.x, rect_selection.min.y),
                        Pos2::new(rect_selection.min.x, rect_selection.max.y),
                    ),
                    0.0,
                    egui::Color32::from_black_alpha(50),
                );
                // rect right of selection
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        Pos2::new(rect_selection.max.x, rect_selection.min.y),
                        Pos2::new(size[0] as f32 + min_pos.x, rect_selection.max.y),
                    ),
                    0.0,
                    egui::Color32::from_black_alpha(50),
                );
            }
            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     egui::warn_if_debug_build(ui);
            // });
        });
    }
}
