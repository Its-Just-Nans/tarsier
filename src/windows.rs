//! Windows to display in Tarsier
use crate::TarsierApp;

/// Togglers for windows
#[derive(serde::Deserialize, serde::Serialize)]
pub struct WindowsManager {
    /// Right panel toggle
    pub right_panel: bool,
    /// Selection windows toggle
    pub selection_window: bool,
    /// Debug and inspection toggle
    pub show_inspection: bool,
}

impl Default for WindowsManager {
    /// Create a new WindowsData
    fn default() -> Self {
        Self {
            right_panel: true,
            selection_window: true,
            show_inspection: false,
        }
    }
}

impl TarsierApp {
    /// Show the windows
    pub fn windows(&mut self, ctx: &egui::Context) {
        self.selection_window(ctx);
    }

    /// Display the selection windows
    pub fn selection_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Selection")
            .open(&mut self.windows.selection_window)
            .show(ctx, |ui| match self.selection {
                Some(rect) => {
                    let width = rect.width().abs() as u32;
                    let height = rect.height().abs() as u32;
                    ui.label(format!("Width: {width}"));
                    ui.label(format!("Height: {height}"));
                    ui.label(format!("Min: {:?}", rect.left_top()));
                    ui.label(format!("Max: {:?}", rect.right_bottom()));
                }
                None => {
                    ui.label("No selection");
                }
            });
        egui::Window::new("Inspection")
            .open(&mut self.windows.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
    }
}
