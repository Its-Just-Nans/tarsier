use image::ImageReader;

use crate::TarsierApp;

pub struct File {
    pub path: String,
    pub data: Vec<u8>,
}

impl TarsierApp {
    pub fn handle_files(&mut self, ctx: &egui::Context) {
        if let Some(result) = &self.file_upload {
            match &result.ready() {
                Some(Ok(File { data, .. })) => {
                    let img = ImageReader::new(std::io::Cursor::new(data)).with_guessed_format();
                    if let Some(img) = self.error_manager.handle_error(img) {
                        let decoded = img.decode();
                        if let Some(img) = self.error_manager.handle_error(decoded) {
                            self.base_img = img.clone();
                            self.img = img;
                        }
                    }
                    self.file_upload = None;
                }
                Some(Err(e)) => {
                    self.error_manager.add_error(e.clone());
                    self.file_upload = None;
                }
                None => {}
            }
        }
        if let Some(file) = self.dropped_files.first() {
            self.selection = None;
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = file.path.as_deref() {
                    if let Some(img) = self.error_manager.handle_error(ImageReader::open(path)) {
                        if let Some(img) = self.error_manager.handle_error(img.decode()) {
                            self.base_img = img.clone();
                            self.img = img;
                        }
                    }
                    self.dropped_files.clear();
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                use std::io::Cursor;
                if let Some(bytes) = file.bytes.as_deref() {
                    if let Some(img) = self
                        .error_manager
                        .handle_error(ImageReader::new(Cursor::new(bytes)).with_guessed_format())
                    {
                        if let Some(img) = self.error_manager.handle_error(img.decode()) {
                            self.base_img = img.clone();
                            self.img = img;
                        }
                    }
                    self.dropped_files.clear();
                }
            }
        }
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // read the first file
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
    }
}
