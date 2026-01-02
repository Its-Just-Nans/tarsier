//! Central panel
use bladvak::eframe::egui::{
    self, ColorImage, Id, Image, ImageData, Modal, Pos2, Sense, TextureOptions, Vec2,
};
use bladvak::errors::ErrorManager;
use image::DynamicImage;
use std::sync::Arc;

use crate::{TarsierApp, side_panel::EditMode};

impl TarsierApp {
    /// Show the central panel
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
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
                    if !self.cursor_info.is_selecting {
                        self.cursor_info.selection = None;
                    }
                    let pos = pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                    let correct_pos = Pos2::new(
                        pos.x.round().clamp(0.0, size[0] as f32),
                        pos.y.round().clamp(0.0, size[1] as f32),
                    );
                    match self.image_operations.mode {
                        EditMode::Selection => {
                            self.cursor_info.selection =
                                if let Some(_rect) = self.cursor_info.selection {
                                    Some(egui::Rect::from_two_pos(
                                        self.cursor_info.start_selection,
                                        correct_pos,
                                    ))
                                } else {
                                    self.cursor_info.start_selection = correct_pos;
                                    Some(egui::Rect::from_two_pos(correct_pos, correct_pos))
                                };
                            self.cursor_info.is_selecting = true;
                        }
                        EditMode::Drawing => {
                            #[allow(clippy::cast_possible_truncation)]
                            if self.image_operations.drawing_continuous_line
                                && let Some(last_post) = self.cursor_info.last_drawing_point
                            {
                                let pts = points_between(last_post, correct_pos);
                                for p in pts {
                                    self.draw_point(p.x as i32, p.y as i32);
                                }
                            } else {
                                self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                            }
                            self.cursor_info.last_drawing_point = Some(correct_pos);
                        }
                    }
                }
            } else {
                self.cursor_info.is_selecting = false;
                self.cursor_info.last_drawing_point = None;
            }
            if response.clicked() {
                self.cursor_info.selection = None;
                match self.image_operations.mode {
                    EditMode::Selection => {
                        self.cursor_info.selection = None;
                    }
                    EditMode::Drawing => {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let pos =
                                pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                            let correct_pos = Pos2::new(
                                pos.x.round().clamp(0.0, size[0] as f32),
                                pos.y.round().clamp(0.0, size[1] as f32),
                            );
                            #[allow(clippy::cast_possible_truncation)]
                            self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                            self.cursor_info.last_drawing_point = None;
                        }
                    }
                }
            }
            if let Some(selection) = self.cursor_info.selection {
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

        if self.new_image.is_open {
            let modal = Modal::new(Id::new("Modal new image")).show(ui.ctx(), |ui| {
                ui.label("Create a new image");
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(egui::DragValue::new(&mut self.new_image.width).range(1..=4000));
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(egui::DragValue::new(&mut self.new_image.height).range(1..=4000));
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
                                self.new_image.height,
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

/// Calculate all points between two points
fn points_between(a: Pos2, b: Pos2) -> Vec<Pos2> {
    let mut points = Vec::new();

    let (mut x1, mut y1) = (a.x, a.y);
    let (x2, y2) = (b.x, b.y);

    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1.0 } else { -1.0 };
    let sy = if y1 < y2 { 1.0 } else { -1.0 };
    let mut err = dx + dy;

    let error_margin = 0.01;
    loop {
        points.push(Pos2 { x: x1, y: y1 });

        if (x1 - x2).abs() < error_margin && (y1 - y2).abs() < error_margin {
            break;
        }

        let e2 = 2.0 * err;

        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }

    points
}
