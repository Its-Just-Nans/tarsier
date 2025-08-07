//! Settings component

use egui::{Context, Id, Modal};

/// Settings object
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Settings {
    /// Is setting modal open
    pub open: bool,

    /// Minimum width for the sidebar
    pub min_width_sidebar: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open: false,
            min_width_sidebar: 200.0, // Default minimum width for the sidebar
        }
    }
}

impl Settings {
    /// Show settings Ui
    pub fn show(&mut self, ctx: &Context, ui_fn: impl FnOnce(&mut egui::Ui)) {
        if self.open {
            let modal = Modal::new(Id::new("Modal settings")).show(ctx, |ui| {
                ui.label(format!("{} settings", env!("CARGO_PKG_NAME")));
                ui.separator();
                ui_fn(ui);
                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |modal_ui| {
                        if modal_ui.button("Close").clicked() {
                            modal_ui.close();
                        }
                    },
                );
            });
            if modal.should_close() {
                self.open = false;
            }
        }
    }
}
