use std::fmt::Display;

use image::{ColorType, DynamicImage, GenericImage, GenericImageView, Pixel};

use crate::TarsierApp;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
pub enum EditMode {
    Selection,
    Drawing,
}

impl Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Selection => write!(f, "Selection"),
            EditMode::Drawing => write!(f, "Drawing"),
        }
    }
}

pub struct Others {
    pub convert_to: ColorType,
}

impl Default for Others {
    fn default() -> Self {
        Self {
            convert_to: ColorType::Rgba8,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ImageOperations {
    pub blur: f32,
    pub hue_rotation: i32,
    pub brighten: i32,
    pub contrast: f32,
    pub pen_radius: u32,
    pub pen_color: [u8; 4],
    pub mode: EditMode,
    pub drawing_mode: bool,
    #[serde(skip)]
    pub other: Others,
}

impl Default for ImageOperations {
    fn default() -> Self {
        Self {
            blur: 10.0,
            hue_rotation: 50,
            brighten: 50,
            contrast: 1.0,
            pen_radius: 1,
            pen_color: [0, 0, 0, 255],
            mode: EditMode::Selection,
            drawing_mode: false,
            other: Others {
                convert_to: ColorType::Rgba8,
            },
        }
    }
}

impl TarsierApp {
    pub fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("my_panel").show(ctx, |ui| {
            ui.heading("Image Info");
            ui.label(format!("Size: {}x{}", self.img.width(), self.img.height()));
            ui.label(format!("Format: {:?}", self.img.color()));

            ui.separator();
            ui.label("Convert");
            egui::ComboBox::from_id_salt("convert_box")
                .selected_text(format!("{:?}", self.image_operations.other.convert_to))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::L8,
                        format!("{:?}", ColorType::L8),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::L16,
                        format!("{:?}", ColorType::L16),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::La8,
                        format!("{:?}", ColorType::La8),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::La16,
                        format!("{:?}", ColorType::La16),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgb8,
                        format!("{:?}", ColorType::Rgb8),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgb16,
                        format!("{:?}", ColorType::Rgb16),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgb32F,
                        format!("{:?}", ColorType::Rgb32F),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgba8,
                        format!("{:?}", ColorType::Rgba8),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgba16,
                        format!("{:?}", ColorType::Rgba16),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.other.convert_to,
                        ColorType::Rgba32F,
                        format!("{:?}", ColorType::Rgba32F),
                    );
                });
            if ui.button("Convert").clicked() {
                self.img = match self.image_operations.other.convert_to {
                    ColorType::L8 => self.img.to_luma8().into(),
                    ColorType::L16 => self.img.to_luma16().into(),
                    ColorType::La8 => self.img.to_luma_alpha8().into(),
                    ColorType::La16 => self.img.to_luma_alpha16().into(),
                    ColorType::Rgb8 => self.img.to_rgb8().into(),
                    ColorType::Rgb16 => self.img.to_rgb16().into(),
                    ColorType::Rgb32F => self.img.to_rgb32f().into(),
                    ColorType::Rgba8 => self.img.to_rgba8().into(),
                    ColorType::Rgba16 => self.img.to_rgba16().into(),
                    ColorType::Rgba32F => self.img.to_rgba32f().into(),
                    _ => self.img.to_rgba8().into(),
                }
            }

            let current_selection = self.selection.map(|selection| {
                (
                    selection.min.x as u32,
                    selection.min.y as u32,
                    selection.max.x as u32,
                    selection.max.y as u32,
                )
            });

            ui.separator();
            self.button_outline(ui);
            ui.separator();
            self.button_grayscale(ui, &current_selection);
            ui.separator();
            self.button_invert(ui, &current_selection);
            ui.separator();
            self.button_blur(ui, &current_selection);
            ui.separator();
            ui.add(egui::Slider::new(
                &mut self.image_operations.hue_rotation,
                0..=360,
            ));

            if ui.button("hue rotate").clicked() {
                match current_selection {
                    Some(selection) => {
                        let cropped_img = self.img.crop(
                            selection.0,
                            selection.1,
                            selection.2 - selection.0,
                            selection.3 - selection.1,
                        );
                        let inner = cropped_img.huerotate(self.image_operations.hue_rotation);
                        self.img
                            .copy_from(&inner, selection.0, selection.1)
                            .unwrap();
                    }
                    None => {
                        self.img = self.img.huerotate(self.image_operations.hue_rotation);
                    }
                }
            }
            ui.separator();
            ui.add(egui::Slider::new(
                &mut self.image_operations.brighten,
                -100..=100,
            ));
            if ui.button("brighten").clicked() {
                match current_selection {
                    Some(selection) => {
                        let cropped_img = self.img.crop(
                            selection.0,
                            selection.1,
                            selection.2 - selection.0,
                            selection.3 - selection.1,
                        );
                        let inner = cropped_img.brighten(self.image_operations.brighten);
                        self.img
                            .copy_from(&inner, selection.0, selection.1)
                            .unwrap();
                    }
                    None => {
                        self.img = self.img.brighten(self.image_operations.brighten);
                    }
                }
            }
            ui.separator();
            ui.add(egui::Slider::new(
                &mut self.image_operations.contrast,
                -50.0..=50.0,
            ));
            if ui.button("contrast").clicked() {
                match current_selection {
                    Some(selection) => {
                        let cropped_img = self.img.crop(
                            selection.0,
                            selection.1,
                            selection.2 - selection.0,
                            selection.3 - selection.1,
                        );
                        let inner = cropped_img.adjust_contrast(self.image_operations.contrast);
                        self.img
                            .copy_from(&inner, selection.0, selection.1)
                            .unwrap();
                    }
                    None => {
                        self.img = self.img.adjust_contrast(self.image_operations.contrast);
                    }
                }
            }
            ui.separator();
            if self.image_operations.mode == EditMode::Drawing {
                ui.add(egui::Slider::new(
                    &mut self.image_operations.pen_radius,
                    1..=500,
                ));
                let [r, g, b, a] = self.image_operations.pen_color;
                let mut color = egui::Color32::from_rgba_premultiplied(r, g, b, a);
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::OnlyBlend,
                );
                self.image_operations.pen_color = [color.r(), color.g(), color.b(), color.a()];
                ui.checkbox(&mut self.image_operations.drawing_mode, "Blend");
                ui.separator();
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }

    pub fn button_outline(&mut self, ui: &mut egui::Ui) {
        if ui.button("outline").clicked() {
            let sobel_x = self.img.filter3x3(&[
                -1.0, 0.0, 1.0, //
                -2.0, 0.0, 2.0, //
                -1.0, 0.0, 1.0, //
            ]);
            let sobel_x2 = self.img.filter3x3(&[
                1.0, 0.0, -1.0, //
                2.0, 0.0, -2.0, //
                1.0, 0.0, -1.0, //
            ]);
            let sobel_y = self.img.filter3x3(&[
                -1.0, -2.0, -1.0, //
                0.0, 0.0, 0.0, //
                1.0, 2.0, 1.0, //
            ]);
            let sobel_y2 = self.img.filter3x3(&[
                1.0, 2.0, 1.0, //
                0.0, 0.0, 0.0, //
                -1.0, -2.0, -1.0, //
            ]);
            for y in 0..self.img.height() {
                for x in 0..self.img.width() {
                    let mut pixel = sobel_x.get_pixel(x, y);
                    let pixel_y = sobel_y.get_pixel(x, y);
                    pixel.blend(&pixel_y);
                    let pixel_x2 = sobel_x2.get_pixel(x, y);
                    pixel.blend(&pixel_x2);
                    let pixel_y2 = sobel_y2.get_pixel(x, y);
                    pixel.blend(&pixel_y2);
                    self.img.put_pixel(x, y, pixel);
                }
            }
        }
    }

    pub fn button_grayscale(
        &mut self,
        ui: &mut egui::Ui,
        current_selection: &Option<(u32, u32, u32, u32)>,
    ) {
        if ui.button("Grayscale").clicked() {
            match current_selection {
                Some(selection) => {
                    let cropped_img = self.img.crop(
                        selection.0,
                        selection.1,
                        selection.2 - selection.0,
                        selection.3 - selection.1,
                    );
                    let inner_mut = cropped_img.grayscale();
                    let inner_mut: DynamicImage = match self.img.color() {
                        ColorType::L8 => inner_mut.to_luma8().into(),
                        ColorType::L16 => inner_mut.to_luma16().into(),
                        ColorType::La8 => inner_mut.to_luma_alpha8().into(),
                        ColorType::La16 => inner_mut.to_luma_alpha16().into(),
                        ColorType::Rgb8 => inner_mut.to_rgb8().into(),
                        ColorType::Rgb16 => inner_mut.to_rgb16().into(),
                        ColorType::Rgba8 => inner_mut.to_rgba8().into(),
                        ColorType::Rgba16 => inner_mut.to_rgba16().into(),
                        _ => inner_mut.to_rgba8().into(),
                    };
                    self.img
                        .copy_from(&inner_mut, selection.0, selection.1)
                        .unwrap();
                }
                None => {
                    let full_img = self.img.grayscale();
                    self.img = match self.img.color() {
                        ColorType::L8 => full_img.to_luma8().into(),
                        ColorType::L16 => full_img.to_luma16().into(),
                        ColorType::La8 => full_img.to_luma_alpha8().into(),
                        ColorType::La16 => full_img.to_luma_alpha16().into(),
                        ColorType::Rgb8 => full_img.to_rgb8().into(),
                        ColorType::Rgb16 => full_img.to_rgb16().into(),
                        ColorType::Rgba8 => full_img.to_rgba8().into(),
                        ColorType::Rgba16 => full_img.to_rgba16().into(),
                        _ => full_img.to_rgba8().into(),
                    }
                }
            }
        }
    }

    pub fn button_invert(
        &mut self,
        ui: &mut egui::Ui,
        current_selection: &Option<(u32, u32, u32, u32)>,
    ) {
        if ui.button("invert").clicked() {
            match current_selection {
                Some(selection) => {
                    let mut cropped_img = self.img.crop(
                        selection.0,
                        selection.1,
                        selection.2 - selection.0,
                        selection.3 - selection.1,
                    );
                    cropped_img.invert();
                    self.img
                        .copy_from(&cropped_img, selection.0, selection.1)
                        .unwrap();
                }
                None => {
                    self.img.invert();
                }
            }
        }
    }

    pub fn button_blur(
        &mut self,
        ui: &mut egui::Ui,
        current_selection: &Option<(u32, u32, u32, u32)>,
    ) {
        ui.add(egui::Slider::new(
            &mut self.image_operations.blur,
            0.0..=100.0,
        ));
        if ui.button("Blur").clicked() {
            match current_selection {
                Some(selection) => {
                    let cropped_img = self.img.crop(
                        selection.0,
                        selection.1,
                        selection.2 - selection.0,
                        selection.3 - selection.1,
                    );
                    let inner_mut = cropped_img.blur(self.image_operations.blur);
                    self.img
                        .copy_from(&inner_mut, selection.0, selection.1)
                        .unwrap();
                }
                None => {
                    self.img = self.img.blur(self.image_operations.blur);
                }
            }
        }
    }

    pub fn draw_point(&mut self, x_center: i32, y_center: i32) {
        let radius = self.image_operations.pen_radius as i32;
        let color = image::Rgba(self.image_operations.pen_color);
        for y in (y_center - radius)..=(y_center + radius) {
            for x in (x_center - radius)..=(x_center + radius) {
                if (x - x_center).pow(2) + (y - y_center).pow(2) <= radius.pow(2) {
                    // Ensure pixel is within bounds
                    if x >= 0
                        && y >= 0
                        && x < self.img.width() as i32
                        && y < self.img.height() as i32
                    {
                        if self.image_operations.drawing_mode {
                            let mut current_pixel = self.img.get_pixel(x as u32, y as u32);
                            current_pixel.blend(&color);
                            self.img.put_pixel(x as u32, y as u32, current_pixel);
                        } else {
                            self.img.put_pixel(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }
    }
}
