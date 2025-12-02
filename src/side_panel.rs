//! Side panel
use std::{fmt::Display, sync::Arc};

use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::{AppError, ErrorManager};
use image::{ColorType, DynamicImage, GenericImage, GenericImageView, Pixel};

use crate::TarsierApp;

/// Mode
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
pub enum EditMode {
    /// Selection mode
    Selection,
    /// Drawing mode
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

/// Image settings
pub struct Others {
    /// Convert to color
    pub convert_to: ColorType,
}

impl Default for Others {
    fn default() -> Self {
        Self {
            convert_to: ColorType::Rgba8,
        }
    }
}

/// Image opterations settings
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ImageOperations {
    /// Blur value
    pub blur: f32,
    /// Hue rotation value
    pub hue_rotation: i32,
    /// Brighten value
    pub brighten: i32,
    /// Contrast value
    pub contrast: f32,
    /// Pen radius
    pub pen_radius: u32,
    /// Pen color
    pub pen_color: [u8; 4],
    /// Editor mode
    pub mode: EditMode,
    /// Drawing mode
    pub drawing_blend: bool,
    /// Others settings
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
            drawing_blend: false,
            other: Others {
                convert_to: ColorType::Rgba8,
            },
        }
    }
}

impl TarsierApp {
    /// Side panel
    pub(crate) fn app_side_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if !self.image_info_as_window {
            self.image_info(ui);
            ui.separator();
        }
        if !self.operations_as_window {
            self.image_operations(ui, error_manager);
            ui.separator();
        }
        if !self.cursor_op_as_window {
            self.cursor_ui(ui);
        }
    }

    /// Image info
    pub(crate) fn image_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Image Info");
        ui.label(format!("Size: {}x{}", self.img.width(), self.img.height()));
        ui.label(format!("Format: {:?}", self.img.color()));
        match &self.exif {
            Some(exif) => {
                ui.collapsing("Exif info", |ui| {
                    TableBuilder::new(ui)
                        .max_scroll_height(100.0)
                        .striped(true)
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::remainder())
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.label("Exif tag");
                            });
                            header.col(|ui| {
                                ui.label("IFD idx");
                            });
                            header.col(|ui| {
                                ui.label("exif value");
                            });
                        })
                        .body(|mut body| {
                            for field in exif.fields() {
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!("{}", field.tag));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", field.ifd_num));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            field.display_value().with_unit(exif)
                                        ));
                                    });
                                });
                            }
                        });
                });
            }
            None => {
                ui.label("No exif detected");
            }
        };
    }

    /// Side panel content
    pub(crate) fn image_operations(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
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
            let new_img = match self.image_operations.other.convert_to {
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
            };
            self.update_image(new_img);
        }
        ui.separator();
        self.button_outline(ui, error_manager);
        ui.separator();
        if ui.button("edge detection").clicked() {
            self.apply_op(
                |img| {
                    img.filter3x3(&[
                        0.0, -1.0, 0.0, //
                        -1.0, 4.0, -1.0, //
                        0.0, -1.0, 0.0, //
                    ])
                },
                error_manager,
            );
        }
        ui.separator();
        if ui.button("Grayscale").clicked() {
            let color = self.img.color();
            self.apply_op(
                |img| {
                    let inner = img.grayscale();
                    match color {
                        ColorType::L8 => inner.to_luma8().into(),
                        ColorType::L16 => inner.to_luma16().into(),
                        ColorType::La8 => inner.to_luma_alpha8().into(),
                        ColorType::La16 => inner.to_luma_alpha16().into(),
                        ColorType::Rgb8 => inner.to_rgb8().into(),
                        ColorType::Rgb16 => inner.to_rgb16().into(),
                        ColorType::Rgba8 => inner.to_rgba8().into(),
                        ColorType::Rgba16 => inner.to_rgba16().into(),
                        _ => inner.to_rgba8().into(),
                    }
                },
                error_manager,
            );
        }
        ui.separator();
        if ui.button("invert").clicked() {
            self.apply_op(
                |img| {
                    let mut copied_img = img.clone();
                    copied_img.invert();
                    copied_img
                },
                error_manager,
            );
        }
        ui.separator();
        ui.add(egui::Slider::new(
            &mut self.image_operations.blur,
            0.0..=100.0,
        ));
        if ui.button("Blur").clicked() {
            let blur = self.image_operations.blur;
            self.apply_op(|img| img.blur(blur), error_manager);
        }
        ui.separator();
        ui.add(egui::Slider::new(
            &mut self.image_operations.hue_rotation,
            0..=360,
        ));

        if ui.button("hue rotate").clicked() {
            let hue_rotation = self.image_operations.hue_rotation;
            self.apply_op(|img| img.huerotate(hue_rotation), error_manager);
        }
        ui.separator();
        ui.add(egui::Slider::new(
            &mut self.image_operations.brighten,
            -100..=100,
        ));
        if ui.button("brighten").clicked() {
            let brighten = self.image_operations.brighten;
            self.apply_op(|img| img.brighten(brighten), error_manager);
        }
        ui.separator();
        ui.add(egui::Slider::new(
            &mut self.image_operations.contrast,
            -50.0..=50.0,
        ));
        if ui.button("contrast").clicked() {
            let contrast = self.image_operations.contrast;
            self.apply_op(|img| img.adjust_contrast(contrast), error_manager);
        }
    }

    /// Apply operation
    pub(crate) fn apply_op<F>(&mut self, func: F, error_manager: &mut ErrorManager)
    where
        F: Fn(&DynamicImage) -> DynamicImage,
    {
        let current_selection = self.selection.map(|selection| {
            (
                selection.min.x as u32,
                selection.min.y as u32,
                selection.max.x as u32,
                selection.max.y as u32,
            )
        });
        match current_selection {
            Some(selection) => {
                let cropped_img = self.img.crop(
                    selection.0,
                    selection.1,
                    selection.2 - selection.0,
                    selection.3 - selection.1,
                );
                let inner = func(&cropped_img);
                if let Err(e) = self.img.copy_from(&inner, selection.0, selection.1) {
                    error_manager.add_error(AppError::new_with_source(Arc::new(e)));
                }
                self.updated_image();
            }
            None => {
                let new_img = func(&self.img);
                self.update_image(new_img);
            }
        }
    }

    /// Button to draw settings
    pub fn button_drawing(&mut self, ui: &mut egui::Ui) {
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
        ui.checkbox(&mut self.image_operations.drawing_blend, "Blend");
    }

    /// Button to show the outline
    pub fn button_outline(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if ui.button("sobel outline").clicked() {
            self.apply_op(
                |img| {
                    let mut img = img.clone();
                    let sobel_x = img.filter3x3(&[
                        -1.0, 0.0, 1.0, //
                        -2.0, 0.0, 2.0, //
                        -1.0, 0.0, 1.0, //
                    ]);
                    let sobel_x2 = img.filter3x3(&[
                        1.0, 0.0, -1.0, //
                        2.0, 0.0, -2.0, //
                        1.0, 0.0, -1.0, //
                    ]);
                    let sobel_y = img.filter3x3(&[
                        -1.0, -2.0, -1.0, //
                        0.0, 0.0, 0.0, //
                        1.0, 2.0, 1.0, //
                    ]);
                    let sobel_y2 = img.filter3x3(&[
                        1.0, 2.0, 1.0, //
                        0.0, 0.0, 0.0, //
                        -1.0, -2.0, -1.0, //
                    ]);
                    for y in 0..img.height() {
                        for x in 0..img.width() {
                            let mut pixel = sobel_x.get_pixel(x, y);
                            let pixel_y = sobel_y.get_pixel(x, y);
                            pixel.blend(&pixel_y);
                            let pixel_x2 = sobel_x2.get_pixel(x, y);
                            pixel.blend(&pixel_x2);
                            let pixel_y2 = sobel_y2.get_pixel(x, y);
                            pixel.blend(&pixel_y2);
                            img.put_pixel(x, y, pixel);
                        }
                    }
                    img
                },
                error_manager,
            );
        }
    }

    /// Draw a point
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
                        if self.image_operations.drawing_blend {
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
        self.updated_image();
    }
}
