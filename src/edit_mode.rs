//! Edition mode

use bladvak::eframe::egui;

/// Drawing mode
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
pub(crate) struct DrawingMode {
    /// Drawing mode
    pub drawing_blend: bool,
    /// Continuous line when drawing when dragged
    pub drawing_continuous_line: bool,
    /// Others settings
    /// Pen radius
    pub pen_radius: u32,
    /// Pen color
    pub(crate) pen_color: [u8; 4],
}

impl Default for DrawingMode {
    fn default() -> Self {
        Self {
            drawing_blend: false,
            drawing_continuous_line: true,
            pen_radius: 1,
            pen_color: [0, 0, 0, 255],
        }
    }
}

impl DrawingMode {
    /// Button to draw settings
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, max_radius: u32) {
        ui.add(egui::Slider::new(&mut self.pen_radius, 1..=max_radius / 4))
            .on_hover_text("Pen radius");
        let [r, g, b, a] = self.pen_color;
        let mut color = egui::Color32::from_rgba_premultiplied(r, g, b, a);
        egui::color_picker::color_edit_button_srgba(
            ui,
            &mut color,
            egui::color_picker::Alpha::OnlyBlend,
        )
        .on_hover_text("Pen color");
        self.pen_color = [color.r(), color.g(), color.b(), color.a()];
        ui.horizontal(|ui| {
            ui.label("Pixel mode:");
            if ui
                .selectable_label(!self.drawing_blend, "Replace")
                .clicked()
            {
                self.drawing_blend = false;
            }
            if ui.selectable_label(self.drawing_blend, "Blend").clicked() {
                self.drawing_blend = true;
            }
        });
        ui.checkbox(&mut self.drawing_continuous_line, "Continuous line");
    }
}

/// Edit mode
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub(crate) struct Mode {
    /// current mode
    pub(crate) current: EditMode,
    /// drawing mode
    pub(crate) drawing: DrawingMode,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            current: EditMode::Cursor,
            drawing: DrawingMode::default(),
        }
    }
}

/// Mode
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy)]
pub(crate) enum EditMode {
    /// Cursor
    Cursor,
    /// Selection mode
    Selection,
    /// Drawing mode
    Drawing,
}

impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Cursor => write!(f, "Cursor"),
            EditMode::Selection => write!(f, "Selection"),
            EditMode::Drawing => write!(f, "Drawing"),
        }
    }
}

/// Cursor state
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SelectionState {
    /// Selection rectangle
    #[serde(skip)]
    pub rectangle: Option<egui::Rect>,

    /// Start selection position
    #[serde(skip)]
    pub start_selection: egui::Pos2,

    /// Start selection position
    #[serde(skip)]
    pub last_drawing_point: Option<egui::Pos2>,

    /// Selection as windows
    pub cursor_op_as_window: bool,

    /// Remove selection after operation
    pub remove_selection_after_op: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            rectangle: None,
            cursor_op_as_window: false,
            start_selection: egui::Pos2::ZERO,
            last_drawing_point: None,
            remove_selection_after_op: false,
        }
    }
}
