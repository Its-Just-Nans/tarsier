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
