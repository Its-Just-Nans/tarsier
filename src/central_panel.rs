//! Central panel
use std::sync::Arc;

use egui::{load::SizedTexture, ColorImage, Image, ImageData, Pos2, Sense, TextureOptions, Vec2};

use crate::{side_panel::EditMode, TarsierApp};

impl TarsierApp {
    /// Show the central panel
    pub fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let size = [self.img.width() as _, self.img.height() as _];
            let image_buffer = self.img.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            let svg_texture = ctx.load_texture(
                "img",
                ImageData::Color(Arc::new(image)),
                TextureOptions::default(),
            );

            let sized_texture = SizedTexture::from_handle(&svg_texture);

            egui::ScrollArea::both().show_viewport(ui, |ui, viewport| {
                let response = ui.add(Image::new(sized_texture).sense(Sense::click_and_drag()));
                let img_position = response.rect;
                let ecart_x = img_position.min.x + viewport.min.x;
                let ecart_y = img_position.min.y + viewport.min.y;

                let painter = ui.painter();
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if !self.is_selecting {
                            self.selection = None;
                        }
                        let pos =
                            pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
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
                                let pos = pos
                                    - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
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
        });
    }
}
