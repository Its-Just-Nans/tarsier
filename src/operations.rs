//! Image operations

use bladvak::{
    ErrorManager,
    eframe::egui::{self, Color32, Image, ImageSource, include_image},
};

use crate::TarsierApp;

impl TarsierApp {
    /// Reset icon
    const RESET_ICON: ImageSource<'_> = include_image!("../assets/icon_x-circle.png");
    /// Save state icon
    const SAVE_STATE_ICON: ImageSource<'_> = include_image!("../assets/icon_check.png");
    /// Rotate icon
    const ROTATE_CCW_ICON: ImageSource<'_> = include_image!("../assets/icon_rotate_ccw.png");
    /// Rotate icon inverse
    const ROTATE_CW_ICON: ImageSource<'_> = include_image!("../assets/icon_rotate_cw.png");
    /// Flip horizontal icon
    const FLIP_H_ICON: ImageSource<'_> = include_image!("../assets/icon_flip_h.png");
    /// Flip vertical icon
    const FLIP_V_ICON: ImageSource<'_> = include_image!("../assets/icon_flip_v.png");

    /// Quick operations
    pub(crate) fn quick_operations(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        let is_dark_theme = ui.ctx().global_style().visuals.dark_mode;
        ui.horizontal(|ui| {
            let ico_image = Image::new(Self::RESET_ICON);
            if ui
                .add(egui::Button::image(if is_dark_theme {
                    ico_image
                } else {
                    ico_image.tint(Color32::BLACK)
                }))
                .on_hover_text("Reset the image to the saved state")
                .clicked()
            {
                let Some(document) = self.documents.get_current_doc_mut() else {
                    return;
                };
                let new_img = document.saved_img.clone();
                self.update_image(new_img);
            }
            let ico_image = Image::new(Self::SAVE_STATE_ICON);
            if ui
                .add(egui::Button::image(if is_dark_theme {
                    ico_image
                } else {
                    ico_image.tint(Color32::BLACK)
                }))
                .on_hover_text("Save the current image state")
                .clicked()
            {
                let Some(document) = self.documents.get_current_doc_mut() else {
                    return;
                };
                document.saved_img = document.img.clone();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            let ico_image = Image::new(Self::ROTATE_CW_ICON);
            if ui
                .add(egui::Button::image(if is_dark_theme {
                    ico_image
                } else {
                    ico_image.tint(Color32::BLACK)
                }))
                .on_hover_text("Rotate 90 degrees clockwise")
                .clicked()
            {
                self.apply_op(image::DynamicImage::rotate90, error_manager);
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
                self.apply_op(image::DynamicImage::rotate270, error_manager);
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
                self.apply_op(image::DynamicImage::fliph, error_manager);
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
                self.apply_op(image::DynamicImage::flipv, error_manager);
            }
        });
    }
}
