//! Document

use bladvak::eframe::egui::{self, Color32};
use bladvak::utils::document::DocumentTrait;
use image::DynamicImage;
use std::path::{Path, PathBuf};

use crate::edit_mode::SelectionState;

/// Document for one image
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct Document {
    /// Current image
    #[serde(skip)]
    pub(crate) img: DynamicImage,
    /// Save of the image
    #[serde(skip)]
    pub(crate) saved_img: DynamicImage,
    /// Image texture
    #[serde(skip)]
    pub(crate) texture: Option<egui::TextureHandle>,
    /// Exif of the image
    #[serde(skip)]
    pub(crate) exif: Option<exif::Exif>,
    /// Path to save the image
    pub(crate) filename: PathBuf,
    /// selection
    pub(crate) selection: SelectionState,
    /// scene rect
    pub(crate) scene_rect: egui::Rect,
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("filename", &self.filename)
            .field("selection", &self.selection)
            .finish_non_exhaustive()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            img: DynamicImage::default(),
            saved_img: DynamicImage::default(),
            texture: None,
            exif: None,
            filename: PathBuf::new(),
            selection: SelectionState::default(),
            scene_rect: egui::Rect::NAN,
        }
    }
}

impl DocumentTrait for Document {
    fn path(&self) -> &Path {
        &self.filename
    }
}

impl Document {
    /// Get the color at a position
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub(crate) fn get_color_at(&self, pos: egui::Pos2) -> Option<(u32, u32, Color32)> {
        use image::GenericImageView;

        let x = pos.x.floor() as u32;
        let y = pos.y.floor() as u32;
        if x < self.img.width() && y < self.img.height() {
            let c = self.img.get_pixel(x, y);
            return Some((
                x,
                y,
                Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
            ));
        }
        None
    }
}
