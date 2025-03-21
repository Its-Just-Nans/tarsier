use crate::TarsierApp;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    pub right_panel: bool,
    pub selection_window: bool,
    pub error_window: bool,
}

impl WindowsData {
    pub fn new() -> Self {
        Self {
            right_panel: true,
            selection_window: true,
            error_window: true,
        }
    }
}

impl TarsierApp {
    pub fn windows(&mut self, ctx: &egui::Context) {
        self.selection_window(ctx);
        self.error_window(ctx);
    }

    pub fn selection_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Selection")
            .open(&mut self.windows.selection_window)
            .show(ctx, |ui| {
                ui.label("Selection rect");
                ui.label(format!("{:?}", self.selection));
                ui.separator();
            });
    }

    pub fn error_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Errors")
            .open(&mut self.windows.error_window)
            .show(ctx, |ui| {
                for error in &self.error_manager.errors {
                    ui.label(error.message.clone());
                }
            });
    }
}
