//! Document

use bladvak::eframe::egui;
use bladvak::utils::document::DocumentTrait;
use image::DynamicImage;
use std::path::{Path, PathBuf};

use crate::edit_mode::SelectionState;

/// Document for one image
#[derive(serde::Deserialize, serde::Serialize, Default)]
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
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("filename", &self.filename)
            .field("selection", &self.selection)
            .finish_non_exhaustive()
    }
}

impl DocumentTrait for Document {
    fn path(&self) -> &Path {
        &self.filename
    }
}
