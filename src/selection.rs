//! Selection

use bladvak::eframe::egui;
use egui::{Image, ImageSource};

use crate::TarsierApp;
use crate::selection::egui::Color32;

impl TarsierApp {
    /// Crop icon
    const CROP_ICON: ImageSource<'_> = egui::include_image!("../assets/icon_crop.png");

    /// show ui of selection
    pub(crate) fn selection_ui(&mut self, ui: &mut egui::Ui) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let mut reset_rect = false;
        match &mut document.selection.rectangle {
            Some(rect) => {
                if ui
                    .label(format!(
                        "Selection: {:.0}x{:.0}",
                        rect.width().abs(),
                        rect.height().abs()
                    ))
                    .on_hover_text("Click to clear selection")
                    .clicked()
                {
                    reset_rect = true;
                }
                let right = rect.right();
                let bottom = rect.bottom();
                #[allow(clippy::cast_precision_loss)]
                ui.horizontal(|ui| {
                    ui.label("Min: ");
                    ui.add(egui::DragValue::new(rect.left_mut()).range(0.0..=right));
                    ui.add(egui::DragValue::new(rect.top_mut()).range(0.0..=bottom));
                });
                #[allow(clippy::cast_precision_loss)]
                ui.horizontal(|ui| {
                    ui.label("Size");
                    let mut width = rect.width().abs();
                    let left = rect.left();
                    let max_width = (document.img.width() as f32) - left;
                    if ui
                        .add(egui::DragValue::new(&mut width).range(0.0..=max_width))
                        .changed()
                    {
                        rect.set_width(width);
                    }
                    ui.label("x");
                    let mut height = rect.height().abs();
                    let top = rect.top();
                    let max_height = (document.img.height() as f32) - top;
                    if ui
                        .add(egui::DragValue::new(&mut height).range(0.0..=max_height))
                        .changed()
                    {
                        rect.set_height(height);
                    }
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
                let cropped_img = document
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
