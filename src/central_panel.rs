//! Central panel
use bladvak::eframe::egui::{
    self, ColorImage, Id, Image, ImageData, Modal, Pos2, Sense, TextureOptions, Vec2,
};
use bladvak::errors::ErrorManager;
use image::DynamicImage;
use std::sync::Arc;

use crate::{side_panel::EditMode, TarsierApp};

impl TarsierApp {
    /// Show the central panel
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        egui::ScrollArea::both().show_viewport(ui, |ui, viewport| {
            let size = [self.img.width() as _, self.img.height() as _];
            let image_texture = self.texture.get_or_insert_with(|| {
                let image_buffer = self.img.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                ui.ctx().load_texture(
                    "img",
                    ImageData::Color(Arc::new(image)),
                    TextureOptions::default(),
                )
            });
            let response = ui.add(
                Image::new((image_texture.id(), image_texture.size_vec2()))
                    .sense(Sense::click_and_drag()),
            );
            let img_position = response.rect;
            let ecart_x = img_position.min.x + viewport.min.x;
            let ecart_y = img_position.min.y + viewport.min.y;

            let painter = ui.painter();
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if !self.is_selecting {
                        self.selection = None;
                    }
                    let pos = pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                    let correct_pos = Pos2::new(
                        pos.x.round().clamp(0.0, size[0] as f32),
                        pos.y.round().clamp(0.0, size[1] as f32),
                    );
                    match self.image_operations.mode {
                        EditMode::Selection => {
                            self.selection = match self.selection {
                                Some(_rect) => Some(egui::Rect::from_two_pos(
                                    self.start_selection,
                                    correct_pos,
                                )),
                                None => {
                                    self.start_selection = correct_pos;
                                    Some(egui::Rect::from_two_pos(correct_pos, correct_pos))
                                }
                            };
                            self.is_selecting = true;
                        }
                        EditMode::Drawing => {
                            self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                        }
                    }
                }
            } else {
                self.is_selecting = false;
            }
            if response.clicked() {
                self.selection = None;
                match self.image_operations.mode {
                    EditMode::Selection => {
                        self.selection = None;
                    }
                    EditMode::Drawing => {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let pos =
                                pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                            let correct_pos = Pos2::new(
                                pos.x.round().clamp(0.0, size[0] as f32),
                                pos.y.round().clamp(0.0, size[1] as f32),
                            );
                            self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                        }
                    }
                }
            }
            if let Some(selection) = self.selection {
                let min_pos = img_position.min;
                let rect_selection = egui::Rect::from_two_pos(
                    Pos2::new(
                        selection.min.x - viewport.min.x + ecart_x,
                        selection.min.y - viewport.min.y + ecart_y,
                    ),
                    Pos2::new(
                        selection.max.x - viewport.min.x + ecart_x,
                        selection.max.y - viewport.min.y + ecart_y,
                    ),
                );
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
        });
        {
            let mut open = self.cursor_op_as_window;
            egui::Window::new("Cursor operations")
                .open(&mut open)
                .show(ui.ctx(), |window_ui| {
                    self.cursor_ui(window_ui);
                });
            self.cursor_op_as_window = open;
        }

        {
            let mut open = self.image_info_as_window;
            egui::Window::new("Image info")
                .open(&mut open)
                .show(ui.ctx(), |window_ui| {
                    self.image_info(window_ui);
                });
            self.image_info_as_window = open;
        }

        {
            let mut open = self.image_operations.is_window;
            egui::Window::new("Operations")
                .open(&mut open)
                .show(ui.ctx(), |window_ui| {
                    self.image_operations(window_ui, error_manager);
                });
            self.image_operations.is_window = open;
        }
        if self.new_image.is_open {
            let modal = Modal::new(Id::new("Modal new image")).show(ui.ctx(), |ui| {
                ui.label("Create a new image");
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(egui::DragValue::new(&mut self.new_image.width).range(1..=4000));
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(egui::DragValue::new(&mut self.new_image.heigth).range(1..=4000));
                });
                ui.label("Color type");
                Self::combo_box_color_type(ui, &mut self.new_image.color_type);
                egui::Sides::new().show(
                    ui,
                    |modal_ui| {
                        if modal_ui.button("Cancel").clicked() {
                            modal_ui.close();
                        }
                    },
                    |modal_ui| {
                        if modal_ui.button("Create").clicked() {
                            let new_img = DynamicImage::new(
                                self.new_image.width,
                                self.new_image.heigth,
                                self.new_image.color_type,
                            );
                            self.update_file(new_img, None);
                            modal_ui.close();
                        }
                    },
                );
            });
            if modal.should_close() {
                self.new_image.is_open = false;
            }
        }
    }
}
