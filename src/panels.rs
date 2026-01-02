//! Tarsier Panels
use bladvak::{app::BladvakPanel, eframe::egui};

use crate::TarsierApp;

/// Panel for image operations
#[derive(Debug)]
pub(crate) struct ImageOperationsPanel;

impl BladvakPanel for ImageOperationsPanel {
    type App = TarsierApp;

    fn name(&self) -> &'static str {
        "Image operations"
    }

    fn has_settings(&self) -> bool {
        false
    }

    fn ui_settings(
        &self,
        _app: &mut Self::App,
        _ui: &mut bladvak::eframe::egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut bladvak::eframe::egui::Ui,
        error_manager: &mut bladvak::ErrorManager,
    ) {
        app.image_operations(ui, error_manager);
    }
}

/// Panel for image information
#[derive(Debug)]
pub(crate) struct ImageInfo;

impl BladvakPanel for ImageInfo {
    type App = TarsierApp;

    fn name(&self) -> &'static str {
        "Image infos"
    }

    fn has_settings(&self) -> bool {
        true
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui_settings(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        if ui.button("Default image").clicked() {
            let (img, cursor) = TarsierApp::load_default_image();
            app.update_file(img, Some(cursor));
        }
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        app.image_info(ui);
    }
}

/// Panel for cursor operations
#[derive(Debug)]
pub(crate) struct CursorInfo;

impl BladvakPanel for CursorInfo {
    type App = TarsierApp;

    fn has_settings(&self) -> bool {
        false
    }
    fn ui_settings(
        &self,
        _app: &mut Self::App,
        _ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
    }

    fn has_ui(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "Cursor operations"
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        app.cursor_ui(ui);
    }
}
