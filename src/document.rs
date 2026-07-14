//! Document

use std::path::PathBuf;

use bladvak::eframe::egui;
use image::DynamicImage;

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
    pub(crate) save_path: Option<PathBuf>,
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document").finish()
    }
}

/// Documents
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Documents {
    /// current index
    pub(crate) current_idx: usize,
    /// documents
    inner: Vec<Document>,
}

impl Default for Documents {
    fn default() -> Self {
        Self {
            current_idx: 0,
            inner: vec![Document::default()],
        }
    }
}

impl Documents {
    /// get current document as mutable
    pub(crate) fn get_current_doc_mut(&mut self) -> Option<&mut Document> {
        if self.inner.is_empty() {
            return None;
        }
        let idx = self.current_idx % self.inner.len();
        Some(&mut self.inner[idx])
    }

    /// add a new document
    #[allow(unused)]
    pub(crate) fn push(&mut self, document: Document) {
        self.inner.push(document);
        self.current_idx = self.inner.len() - 1;
    }

    /// iter on documents
    #[allow(unused)]
    pub(crate) fn iter(&mut self) -> std::slice::Iter<'_, Document> {
        self.inner.iter()
    }

    /// Remove a document
    #[allow(unused)]
    pub(crate) fn remove(&mut self, index: usize) {
        self.inner.remove(index);
        self.current_idx = self.current_idx.saturating_sub(1);
    }

    /// Check if is some
    #[allow(unused)]
    pub(crate) fn is_some(&self) -> bool {
        !self.inner.is_empty()
    }
}
