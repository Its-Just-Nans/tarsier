use std::{io::Cursor, path::PathBuf, sync::Arc};

use egui::{
    load::SizedTexture, ColorImage, Image, ImageData, ImageSource, Pos2, Sense, TextureOptions,
};
use image::{GenericImage, ImageReader};

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct ImageOperations {
    blur: f32,
    hue_rotation: i32,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TarsierApp {
    #[serde(skip)]
    img: image::DynamicImage,

    #[serde(skip)]
    img_position: egui::Rect,

    #[serde(skip)]
    base_img: image::DynamicImage,

    #[serde(skip)]
    selection: Option<egui::Rect>,

    #[serde(skip)]
    start_selection: Pos2,

    #[serde(skip)]
    is_selecting: bool,

    image_operations: ImageOperations,

    save_path: Option<PathBuf>,
}

const ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

const CROP_ICON: ImageSource<'_> = egui::include_image!("../assets/crop.png");
const RESET_ICON: ImageSource<'_> = egui::include_image!("../assets/x-circle.png");
const ROTATE_CCW_ICON: ImageSource<'_> = egui::include_image!("../assets/rotate_ccw.png");
const ROTATE_CW_ICON: ImageSource<'_> = egui::include_image!("../assets/rotate_cw.png");
const FLIP_H_ICON: ImageSource<'_> = egui::include_image!("../assets/flip_h.png");
const FLIP_V_ICON: ImageSource<'_> = egui::include_image!("../assets/flip_v.png");

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
            img_position: egui::Rect::ZERO,
            selection: None,
            start_selection: Pos2::ZERO,
            is_selecting: false,
            image_operations: Default::default(),
            save_path: None,
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

fn save_image(img: &image::DynamicImage, format: image::ImageFormat, path_file: &PathBuf) {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format).unwrap();
    save_file(&bytes, path_file);
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file(data: &[u8], path_file: &PathBuf) {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(path_file).unwrap();
    file.write_all(data).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn save_file(data: &[u8], path_file: &PathBuf) {
    // create blob
    use eframe::wasm_bindgen::JsCast;
    use js_sys::Array;

    let filename = path_file.file_name().unwrap().to_str().unwrap();

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

impl TarsierApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn get_save_path(&mut self) -> std::path::PathBuf {
        use rfd::FileDialog;
        let path = FileDialog::new()
            .set_directory(match &self.save_path {
                Some(path) => path,
                None => std::path::Path::new("."),
            })
            .pick_folder();
        if let Some(path) = path {
            self.save_path = Some(path.clone());
            path
        } else {
            std::path::PathBuf::new()
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn get_save_path(&mut self) -> std::path::PathBuf {
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
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    ui.add(egui::Hyperlink::from_label_and_url(
                        "Github repo",
                        "https://github.com/Its-Just-Nans/tarsier",
                    ));
                });
                ui.menu_button("Options", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                ui.separator();
                if ui
                    .add_enabled(self.selection.is_some(), egui::Button::image(CROP_ICON))
                    .on_disabled_hover_text("You first need to select an area to crop")
                    .on_hover_text("Crop the image")
                    .clicked()
                {
                    if let Some(selection) = self.selection {
                        let min_pos = selection.min;
                        let max_pos = selection.max;
                        let min_x = min_pos.x as u32;
                        let min_y = min_pos.y as u32;
                        let max_x = max_pos.x as u32;
                        let max_y = max_pos.y as u32;
                        let cropped_img =
                            self.img
                                .crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
                        self.img = cropped_img;
                        self.selection = None;
                    }
                }
                if ui
                    .add(egui::Button::image(RESET_ICON))
                    .on_hover_text("Reset the image")
                    .clicked()
                {
                    self.img = self.base_img.clone();
                    self.selection = None;
                }
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
                ui.menu_button("Save", |ui| {
                    if ui.button("PNG").clicked() {
                        let save_path = self.get_save_path();
                        save_image(
                            &self.img,
                            image::ImageFormat::Png,
                            &save_path.join("image.png"),
                        );
                    }
                    if ui.button("JPEG").clicked() {
                        let save_path = self.get_save_path();
                        save_image(
                            &self.img,
                            image::ImageFormat::Jpeg,
                            &save_path.join("image.jpg"),
                        );
                    }
                    if ui.button("BMP").clicked() {
                        let save_path = self.get_save_path();
                        save_image(
                            &self.img,
                            image::ImageFormat::Bmp,
                            &save_path.join("image.bmp"),
                        );
                    }
                    if ui.button("GIF").clicked() {
                        let save_path = self.get_save_path();
                        save_image(
                            &self.img,
                            image::ImageFormat::Gif,
                            &save_path.join("image.gif"),
                        );
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

        egui::CentralPanel::default().show(ctx, |ui| {
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
            self.img_position = response.rect;

            let painter = ui.painter();
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if !self.is_selecting {
                        self.selection = None;
                    }
                    let correct_pos = Pos2::new(
                        pos.x.clamp(
                            0.0 + self.img_position.min.x,
                            size[0] as f32 + self.img_position.min.x,
                        ),
                        pos.y.clamp(
                            0.0 + self.img_position.min.y,
                            size[1] as f32 + self.img_position.min.y,
                        ),
                    );
                    self.selection = match self.selection {
                        Some(_rect) => {
                            Some(egui::Rect::from_two_pos(self.start_selection, correct_pos))
                        }
                        None => {
                            self.start_selection = correct_pos;
                            Some(egui::Rect::from_two_pos(correct_pos, correct_pos))
                        }
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
                let min_pos = self.img_position.min;
                let rect_selection = selection;
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

        egui::SidePanel::right("my_panel").show(ctx, |ui| {
            ui.heading("Image Info");
            ui.label(format!("Size: {}x{}", self.img.width(), self.img.height()));
            ui.label(format!("Format: {:?}", self.img.color()));

            ui.separator();
            ui.add(
                egui::DragValue::new(&mut self.image_operations.blur)
                    .speed(1)
                    .range(0.0..=100.0),
            );

            if ui.button("Blur").clicked() {
                match self.selection {
                    Some(selection) => {
                        let min_x = selection.min.x as u32 - self.img_position.min.x as u32;
                        let min_y = selection.min.y as u32 - self.img_position.min.y as u32;
                        let max_x = selection.max.x as u32 - self.img_position.min.x as u32;
                        let max_y = selection.max.y as u32 - self.img_position.min.y as u32;
                        let cropped_img = self.img.crop(min_x, min_y, max_x - min_x, max_y - min_y);
                        let inner_mut = cropped_img.blur(self.image_operations.blur);
                        self.img.copy_from(&inner_mut, min_x, min_y).unwrap();
                    }
                    None => {
                        self.img = self.img.blur(self.image_operations.blur);
                    }
                }
            }

            if ui.button("Grayscale").clicked() {
                match self.selection {
                    Some(selection) => {
                        let min_x = selection.min.x as u32 - self.img_position.min.x as u32;
                        let min_y = selection.min.y as u32 - self.img_position.min.y as u32;
                        let max_x = selection.max.x as u32 - self.img_position.min.x as u32;
                        let max_y = selection.max.y as u32 - self.img_position.min.y as u32;
                        let cropped_img = self.img.crop(min_x, min_y, max_x - min_x, max_y - min_y);
                        let inner_mut = cropped_img.grayscale();
                        self.img.copy_from(&inner_mut, min_x, min_y).unwrap();
                    }
                    None => {
                        self.img = self.img.grayscale();
                    }
                }
            }

            if ui.button("invert").clicked() {
                match self.selection {
                    Some(selection) => {
                        let min_x = selection.min.x as u32 - self.img_position.min.x as u32;
                        let min_y = selection.min.y as u32 - self.img_position.min.y as u32;
                        let max_x = selection.max.x as u32 - self.img_position.min.x as u32;
                        let max_y = selection.max.y as u32 - self.img_position.min.y as u32;
                        let mut cropped_img =
                            self.img.crop(min_x, min_y, max_x - min_x, max_y - min_y);
                        cropped_img.invert();
                        self.img.copy_from(&cropped_img, min_x, min_y).unwrap();
                    }
                    None => {
                        self.img.invert();
                    }
                }
            }
            ui.add(
                egui::DragValue::new(&mut self.image_operations.hue_rotation)
                    .speed(1)
                    .range(0.0..=360.0),
            );

            if ui.button("hue rotate").clicked() {
                match self.selection {
                    Some(selection) => {
                        let min_x = selection.min.x as u32 - self.img_position.min.x as u32;
                        let min_y = selection.min.y as u32 - self.img_position.min.y as u32;
                        let max_x = selection.max.x as u32 - self.img_position.min.x as u32;
                        let max_y = selection.max.y as u32 - self.img_position.min.y as u32;
                        let cropped_img = self.img.crop(min_x, min_y, max_x - min_x, max_y - min_y);
                        let inner = cropped_img.huerotate(self.image_operations.hue_rotation);
                        self.img.copy_from(&inner, min_x, min_y).unwrap();
                    }
                    None => {
                        self.img = self.img.huerotate(self.image_operations.hue_rotation);
                    }
                }
            }
        });
    }
}
