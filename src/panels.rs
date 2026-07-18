//! Tarsier Panels
use std::path::PathBuf;

use bladvak::{app::BladvakPanel, eframe::egui, utils::grid::Grid};

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
        _ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
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
        ui.horizontal(|ui| {
            ui.label(format!("{} settings", app.grid.title()));
            ui.button("⟳").clicked().then(|| {
                app.grid = Grid::default();
            });
            ui.checkbox(&mut app.grid.is_enabled, "");
        });
        app.grid.show_settings(ui);
        ui.separator();
        if ui.button("Default image").clicked() {
            let (img, cursor) = TarsierApp::load_default_image();
            app.new_file(PathBuf::from("tarsier.png"), img, Some(cursor));
        }
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        error_manager: &mut bladvak::ErrorManager,
    ) {
        app.image_info(ui, error_manager);
    }
}

/// Panel for cursor operations
#[derive(Debug)]
pub(crate) struct CursorInfo;

impl BladvakPanel for CursorInfo {
    type App = TarsierApp;

    fn has_settings(&self) -> bool {
        true
    }
    fn ui_settings(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        ui.checkbox(
            &mut app.settings.remove_selection_after_op,
            "Remove selection after change",
        );
        ui.horizontal(|ui| {
            ui.label("Color selection");
            ui.color_edit_button_srgba(&mut app.settings.color_selection);
        });
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
