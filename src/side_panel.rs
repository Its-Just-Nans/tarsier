//! Side panel

use bladvak::eframe::egui::{self, Color32, Pos2};
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::{AppError, ErrorManager};
use image::Rgba;
use image::{ColorType, DynamicImage, GenericImage, GenericImageView, Pixel, imageops::FilterType};
use imageproc::filter::median_filter;
use std::sync::Arc;

use crate::TarsierApp;
use crate::document::Document;

/// Image settings
#[derive(Debug)]
pub(crate) struct Others {
    /// Convert to color
    pub(crate) convert_to: ColorType,
}

impl Default for Others {
    fn default() -> Self {
        Self {
            convert_to: ColorType::Rgba8,
        }
    }
}

/// resize settings
#[derive(Clone, Debug)]
pub(crate) struct Resize {
    /// new width
    pub(crate) nwidth: u32,
    /// new height
    pub(crate) nheight: u32,
    /// filter
    pub(crate) filter: FilterType,
}

impl Default for Resize {
    fn default() -> Self {
        Self {
            nwidth: 500,
            nheight: 500,
            filter: FilterType::Lanczos3,
        }
    }
}

/// Image opterations settings
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ImageOperations {
    /// Blur value
    pub(crate) blur: f32,
    /// Hue rotation value
    pub(crate) hue_rotation: i32,
    /// Brighten value
    pub(crate) brighten: i32,
    /// Contrast value
    pub(crate) contrast: f32,
    /// Others
    #[serde(skip)]
    pub(crate) other: Others,
    /// resize
    #[serde(skip)]
    pub(crate) resize: Resize,
    /// cut color
    cut_color: egui::Color32,
    /// tolerance
    cut_tolerance: i16,
}

impl Default for ImageOperations {
    fn default() -> Self {
        Self {
            blur: 10.0,
            hue_rotation: 50,
            brighten: 50,
            contrast: 1.0,
            other: Others::default(),
            resize: Resize::default(),
            cut_color: Color32::from_rgb_additive(50, 50, 50),
            cut_tolerance: i16::MAX,
        }
    }
}

impl TarsierApp {
    /// Image info
    pub(crate) fn image_info(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut bladvak::ErrorManager,
    ) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        ui.heading("Image Info");
        ui.label(format!(
            "Size: {}x{}",
            document.img.width(),
            document.img.height()
        ));
        ui.label(format!("Format: {:?}", document.img.color()));
        match &document.exif {
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
        }
        if let Some(document) = self.documents.get_current_doc_mut()
            && ui.button("Copy image").clicked()
            && let Err(e) = bladvak::utils::set_image_in_clipboard(
                ui.ctx(),
                document.img.width() as usize,
                document.img.height() as usize,
                document.img.to_rgba8().as_flat_samples().as_slice(),
            )
        {
            error_manager.add_error(e);
        }
    }

    /// Combo box for color type selection
    pub(crate) fn combo_box_color_type(ui: &mut egui::Ui, value: &mut ColorType) {
        egui::ComboBox::from_id_salt("convert_box")
            .selected_text(format!("{value:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(value, ColorType::L8, format!("{:?}", ColorType::L8));
                ui.selectable_value(value, ColorType::L16, format!("{:?}", ColorType::L16));
                ui.selectable_value(value, ColorType::La8, format!("{:?}", ColorType::La8));
                ui.selectable_value(value, ColorType::La16, format!("{:?}", ColorType::La16));
                ui.selectable_value(value, ColorType::Rgb8, format!("{:?}", ColorType::Rgb8));
                ui.selectable_value(value, ColorType::Rgb16, format!("{:?}", ColorType::Rgb16));
                ui.selectable_value(value, ColorType::Rgb32F, format!("{:?}", ColorType::Rgb32F));
                ui.selectable_value(value, ColorType::Rgba8, format!("{:?}", ColorType::Rgba8));
                ui.selectable_value(value, ColorType::Rgba16, format!("{:?}", ColorType::Rgba16));
                ui.selectable_value(
                    value,
                    ColorType::Rgba32F,
                    format!("{:?}", ColorType::Rgba32F),
                );
            });
    }

    /// Button for convert
    fn button_convert(&mut self, ui: &mut egui::Ui) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        ui.label("Convert");
        Self::combo_box_color_type(ui, &mut self.image_operations.other.convert_to);
        if ui.button("Convert").clicked() {
            let new_img = match self.image_operations.other.convert_to {
                ColorType::L8 => document.img.to_luma8().into(),
                ColorType::L16 => document.img.to_luma16().into(),
                ColorType::La8 => document.img.to_luma_alpha8().into(),
                ColorType::La16 => document.img.to_luma_alpha16().into(),
                ColorType::Rgb8 => document.img.to_rgb8().into(),
                ColorType::Rgb16 => document.img.to_rgb16().into(),
                ColorType::Rgb32F => document.img.to_rgb32f().into(),
                ColorType::Rgba16 => document.img.to_rgba16().into(),
                ColorType::Rgba32F => document.img.to_rgba32f().into(),
                ColorType::Rgba8 | _ => document.img.to_rgba8().into(),
            };
            self.update_image(new_img);
        }
    }

    /// Side panel content
    pub(crate) fn image_operations(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.quick_operations(ui, error_manager);
        self.button_convert(ui);
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
            self.apply_op(
                |img| {
                    let color = img.color();
                    let inner = img.grayscale();
                    match color {
                        ColorType::L8 => inner.to_luma8().into(),
                        ColorType::L16 => inner.to_luma16().into(),
                        ColorType::La8 => inner.to_luma_alpha8().into(),
                        ColorType::La16 => inner.to_luma_alpha16().into(),
                        ColorType::Rgb8 => inner.to_rgb8().into(),
                        ColorType::Rgb16 => inner.to_rgb16().into(),
                        ColorType::Rgba16 => inner.to_rgba16().into(),
                        ColorType::Rgba8 | _ => inner.to_rgba8().into(),
                    }
                },
                error_manager,
            );
        }
        ui.separator();
        self.show_basic_ops(ui, error_manager);
        ui.separator();
        self.show_resize(ui, error_manager);
        ui.separator();
        self.show_channels(ui, error_manager);
        ui.separator();
        self.show_median_filter(ui, error_manager);
        ui.separator();
        self.show_cut_color(ui, error_manager);
    }

    /// show basic operations
    pub(crate) fn show_basic_ops(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
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

    /// show median filter
    fn show_median_filter(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if ui.button("Median filter").clicked() {
            let radius = 3;
            self.apply_op(
                |img| {
                    let inner = img.to_luma8();
                    let inner = median_filter(&inner, radius, radius);
                    DynamicImage::ImageLuma8(inner)
                },
                error_manager,
            );
        }
    }

    /// show cut color
    fn show_cut_color(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.add(egui::Slider::new(
            &mut self.image_operations.cut_tolerance,
            0..=254,
        ));
        ui.color_edit_button_srgba(&mut self.image_operations.cut_color);
        if ui.button("Cut color").clicked() {
            let tolerance = self.image_operations.cut_tolerance;
            let target = self.image_operations.cut_color.to_array();
            self.apply_op(
                |img| {
                    let mut res = img.clone().to_rgba8();
                    for (x, y, pixel) in img.pixels() {
                        let [r, g, b, a] = pixel.0;
                        let diff_r = (i16::from(r) - i16::from(target[0])).abs();
                        let diff_g = (i16::from(g) - i16::from(target[1])).abs();
                        let diff_b = (i16::from(b) - i16::from(target[2])).abs();
                        let matches =
                            diff_r <= tolerance && diff_b <= tolerance && diff_g <= tolerance;

                        let color = if matches {
                            Rgba([r, g, b, 0]) // transparent
                        } else {
                            Rgba([r, g, b, a])
                        };
                        res.put_pixel(x, y, color);
                    }
                    DynamicImage::ImageRgba8(res)
                },
                error_manager,
            );
        }
    }

    /// show channels
    fn show_channels(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.collapsing("Channels", |ui| {
            if ui.button("Red").clicked() {
                self.apply_op(
                    |img| -> DynamicImage {
                        let mut c = img.clone();
                        for x in 0..c.width() {
                            for y in 0..c.height() {
                                let mut px = c.get_pixel(x, y);
                                px.channels_mut()[1] = 0;
                                px.channels_mut()[2] = 0;
                                c.put_pixel(x, y, px);
                            }
                        }
                        c
                    },
                    error_manager,
                );
            }
            if ui.button("Green").clicked() {
                self.apply_op(
                    |img| -> DynamicImage {
                        let mut c = img.clone();
                        for x in 0..c.width() {
                            for y in 0..c.height() {
                                let mut px = c.get_pixel(x, y);
                                px.channels_mut()[0] = 0;
                                px.channels_mut()[2] = 0;
                                c.put_pixel(x, y, px);
                            }
                        }
                        c
                    },
                    error_manager,
                );
            }
            if ui.button("Blue").clicked() {
                self.apply_op(
                    |img| -> DynamicImage {
                        let mut c = img.clone();
                        for x in 0..c.width() {
                            for y in 0..c.height() {
                                let mut px = c.get_pixel(x, y);
                                px.channels_mut()[0] = 0;
                                px.channels_mut()[1] = 0;
                                c.put_pixel(x, y, px);
                            }
                        }
                        c
                    },
                    error_manager,
                );
            }
        });
    }

    /// show resize ui
    fn show_resize(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.collapsing("Resize", |ui| {
            ui.add(egui::DragValue::new(
                &mut self.image_operations.resize.nwidth,
            ));
            ui.add(egui::DragValue::new(
                &mut self.image_operations.resize.nheight,
            ));
            egui::ComboBox::from_id_salt("convert_box")
                .selected_text(display_filter_type(&self.image_operations.resize.filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.image_operations.resize.filter,
                        FilterType::Nearest,
                        format!("{:?}", FilterType::Nearest),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.resize.filter,
                        FilterType::Triangle,
                        format!("{:?}", FilterType::Triangle),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.resize.filter,
                        FilterType::CatmullRom,
                        format!("{:?}", FilterType::CatmullRom),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.resize.filter,
                        FilterType::Gaussian,
                        format!("{:?}", FilterType::Gaussian),
                    );
                    ui.selectable_value(
                        &mut self.image_operations.resize.filter,
                        FilterType::Lanczos3,
                        format!("{:?}", FilterType::Lanczos3),
                    );
                });
            if ui.button("Resize").clicked() {
                let resize = self.image_operations.resize.clone();
                self.apply_op(
                    |img| img.resize(resize.nwidth, resize.nheight, resize.filter),
                    error_manager,
                );
            }
        });
    }

    /// Apply operation
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub(crate) fn apply_op<F>(&mut self, func: F, error_manager: &mut ErrorManager)
    where
        F: Fn(&DynamicImage) -> DynamicImage,
    {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let current_selection = document.selection.rectangle.map(|selection| {
            (
                selection.min.x as u32,
                selection.min.y as u32,
                selection.max.x as u32,
                selection.max.y as u32,
            )
        });
        if let Some(selection) = current_selection {
            let cropped_img = document.img.crop(
                selection.0,
                selection.1,
                selection.2 - selection.0,
                selection.3 - selection.1,
            );
            let img_color = document.img.color();
            let inner = func(&cropped_img);
            let new_color = inner.color();
            if new_color != img_color {
                document.img = match new_color {
                    ColorType::L8 => document.img.to_luma8().into(),
                    ColorType::L16 => document.img.to_luma16().into(),
                    ColorType::La8 => document.img.to_luma_alpha8().into(),
                    ColorType::La16 => document.img.to_luma_alpha16().into(),
                    ColorType::Rgb8 => document.img.to_rgb8().into(),
                    ColorType::Rgb16 => document.img.to_rgb16().into(),
                    ColorType::Rgba16 => document.img.to_rgba16().into(),
                    ColorType::Rgba8 | _ => document.img.to_rgba8().into(),
                };
            }

            if let Err(e) = document.img.copy_from(&inner, selection.0, selection.1) {
                error_manager.add_error(AppError::new_with_source(
                    "Cannot update selected image part",
                    Arc::new(e),
                ));
            }
            self.updated_image();
        } else {
            let new_img = func(&document.img);
            self.update_image(new_img);
        }
    }

    /// Button to show the outline
    #[allow(clippy::similar_names)]
    pub(crate) fn button_outline(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
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
    #[allow(clippy::cast_sign_loss)]
    pub(crate) fn draw_point(&mut self, x_center: u32, y_center: u32) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let radius = self.mode.drawing.pen_radius;
        let min_y = y_center.saturating_sub(radius);
        for y in min_y..=(y_center + radius) {
            let min_x = x_center.saturating_sub(radius);
            for x in min_x..=(x_center + radius) {
                if x.saturating_sub(x_center).pow(2)
                    + y.saturating_sub(y_center).pow(2)
                    + x_center.saturating_sub(x).pow(2)
                    + y_center.saturating_sub(y).pow(2)
                    <= radius.pow(2)
                {
                    // Ensure pixel is within bounds
                    if x < document.img.width() && y < document.img.height() {
                        #[allow(clippy::cast_precision_loss)]
                        if let Some(rect) = document.selection.rectangle
                            && !rect.contains(Pos2::new(x as f32, y as f32))
                        {
                            continue;
                        }
                        draw_single_point(
                            document,
                            x,
                            y,
                            self.mode.drawing.pen_color,
                            self.mode.drawing.drawing_blend,
                        );
                    }
                }
            }
        }
        self.updated_image();
    }
}

/// draw a single point
fn draw_single_point(
    document: &mut Document,
    x: u32,
    y: u32,
    pen_color: [u8; 4],
    drawing_blend: bool,
) {
    let color = image::Rgba(pen_color);
    if drawing_blend {
        let mut current_pixel = document.img.get_pixel(x, y);
        current_pixel.blend(&color);
        document.img.put_pixel(x, y, current_pixel);
    } else {
        document.img.put_pixel(x, y, color);
    }
}

/// display a `FilterType`
fn display_filter_type(filter_type: &FilterType) -> &str {
    match filter_type {
        FilterType::Nearest => "Nearest",
        FilterType::Triangle => "Triangle",
        FilterType::CatmullRom => "CatmullRom",
        FilterType::Gaussian => "Gaussian",
        FilterType::Lanczos3 => "Lanczos3",
    }
}
