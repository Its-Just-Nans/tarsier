//! Central panel
use bladvak::eframe::egui::{
    self, Color32, ColorImage, Id, Image, ImageData, Modal, Pos2, Sense, TextureOptions, Vec2,
};
use bladvak::errors::ErrorManager;
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{TarsierApp, edit_mode::EditMode};

impl TarsierApp {
    /// Show the central panel
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            egui::Area::new("center".into())
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(concat!("Welcome to ", env!("CARGO_PKG_NAME")));
                        ui.label("No document opened");
                    });
                });
            return;
        };
        let mut rect = document.scene_rect;
        egui::Scene::new()
            .zoom_range(0.0..=f32::INFINITY)
            .show(ui, &mut rect, |ui| {
                let Some(document) = self.documents.get_current_doc_mut() else {
                    return;
                };
                let painter = ui.painter();
                let bg_r: egui::Response = ui.response();
                if bg_r.rect.is_finite() {
                    self.grid.draw(&bg_r.rect, painter);
                }

                let size = [document.img.width() as _, document.img.height() as _];
                let image_texture = document.texture.get_or_insert_with(|| {
                    let image_buffer = document.img.to_rgba8();
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
                let is_dark_theme = ui.ctx().global_style().visuals.dark_mode;
                ui.painter().rect_stroke(
                    response.rect,
                    0.0, // corner radius
                    egui::Stroke::new(
                        1.0,
                        if is_dark_theme {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        },
                    ),
                    egui::StrokeKind::Outside,
                );
                let img_position = response.rect;
                let viewport = ui.ctx().viewport_rect();
                let ecart_x = img_position.min.x + viewport.min.x;
                let ecart_y = img_position.min.y + viewport.min.y;

                let painter = ui.painter();
                if let EditMode::Drawing = self.mode.current
                    && let Some(pos) = response.hover_pos()
                {
                    painter.circle(
                        pos,
                        self.mode.drawing.pen_radius as f32,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(1.0, Color32::BLACK),
                    );
                }

                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if !document.selection.is_selecting {
                            document.selection.rectangle = None;
                        }
                        let pos =
                            pos - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                        let correct_pos = Pos2::new(
                            pos.x.round().clamp(0.0, size[0] as f32),
                            pos.y.round().clamp(0.0, size[1] as f32),
                        );
                        match self.mode.current {
                            EditMode::Cursor => {
                                // no nothing
                            }
                            EditMode::Selection => {
                                document.selection.rectangle =
                                    if let Some(_rect) = document.selection.rectangle {
                                        Some(egui::Rect::from_two_pos(
                                            document.selection.start_selection,
                                            correct_pos,
                                        ))
                                    } else {
                                        document.selection.start_selection = correct_pos;
                                        Some(egui::Rect::from_two_pos(correct_pos, correct_pos))
                                    };
                                document.selection.is_selecting = true;
                            }
                            EditMode::Drawing => {
                                #[allow(clippy::cast_possible_truncation)]
                                if self.mode.drawing.drawing_continuous_line
                                    && let Some(last_post) = document.selection.last_drawing_point
                                {
                                    let pts = points_between(last_post, correct_pos);
                                    for p in pts {
                                        self.draw_point(p.x as i32, p.y as i32);
                                    }
                                } else {
                                    self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                                }
                                if let Some(document) = self.documents.get_current_doc_mut() {
                                    document.selection.last_drawing_point = Some(correct_pos);
                                }
                            }
                        }
                    }
                } else {
                    document.selection.is_selecting = false;
                    document.selection.last_drawing_point = None;
                }
                let Some(document) = self.documents.get_current_doc_mut() else {
                    return;
                };

                if response.clicked() {
                    document.selection.rectangle = None;
                    match self.mode.current {
                        EditMode::Cursor => {
                            // do nothing
                        }
                        EditMode::Selection => {
                            document.selection.rectangle = None;
                        }
                        EditMode::Drawing => {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let pos = pos
                                    - Vec2::new(ecart_x - viewport.min.x, ecart_y - viewport.min.y);
                                let correct_pos = Pos2::new(
                                    pos.x.round().clamp(0.0, size[0] as f32),
                                    pos.y.round().clamp(0.0, size[1] as f32),
                                );
                                #[allow(clippy::cast_possible_truncation)]
                                self.draw_point(correct_pos.x as i32, correct_pos.y as i32);
                                if let Some(document) = self.documents.get_current_doc_mut() {
                                    document.selection.last_drawing_point = None;
                                }
                            }
                        }
                    }
                }
                let Some(document) = self.documents.get_current_doc_mut() else {
                    return;
                };
                if let Some(selection) = document.selection.rectangle {
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
                        egui::Stroke::new(1.0, Color32::BLACK),
                        egui::StrokeKind::Outside,
                    );
                    // rect above selection
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            Pos2::new(min_pos.x, min_pos.y),
                            Pos2::new(size[0] as f32 + min_pos.x, rect_selection.min.y),
                        ),
                        0.0,
                        Color32::from_black_alpha(50),
                    );
                    // rect below selection
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            Pos2::new(min_pos.x, rect_selection.max.y),
                            Pos2::new(size[0] as f32 + min_pos.x, size[1] as f32 + min_pos.y),
                        ),
                        0.0,
                        Color32::from_black_alpha(50),
                    );
                    // rect left of selection
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            Pos2::new(min_pos.x, rect_selection.min.y),
                            Pos2::new(rect_selection.min.x, rect_selection.max.y),
                        ),
                        0.0,
                        Color32::from_black_alpha(50),
                    );
                    // rect right of selection
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            Pos2::new(rect_selection.max.x, rect_selection.min.y),
                            Pos2::new(size[0] as f32 + min_pos.x, rect_selection.max.y),
                        ),
                        0.0,
                        Color32::from_black_alpha(50),
                    );
                }
            });
        if let Some(document) = self.documents.get_current_doc_mut() {
            document.scene_rect = rect;
        }
    }

    /// show the new image modal
    pub(crate) fn show_new_image_modal(&mut self, ui: &mut egui::Ui) {
        if self.settings.new_image.is_open {
            let modal = Modal::new(Id::new("Modal new image")).show(ui.ctx(), |ui| {
                ui.label("Create a new image");
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.new_image.width).range(1..=4000),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.new_image.height).range(1..=4000),
                    );
                });
                ui.label("Color type");
                Self::combo_box_color_type(ui, &mut self.settings.new_image.color_type);
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
                                self.settings.new_image.width,
                                self.settings.new_image.height,
                                self.settings.new_image.color_type,
                            );
                            self.new_file(PathBuf::from("new.png"), new_img, None);
                            modal_ui.close();
                        }
                    },
                );
            });
            if modal.should_close() {
                self.settings.new_image.is_open = false;
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
