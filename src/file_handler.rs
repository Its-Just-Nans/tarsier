//! File handler
use crate::errors::AppError;
use image::ImageReader;
use poll_promise::Promise;

/// File object
pub struct File {
    /// File data
    pub data: Vec<u8>,
}

/// File Handler
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FileHandler {
    /// Dropped_files handler
    #[serde(skip)]
    pub dropped_files: Vec<egui::DroppedFile>,

    /// File upload handling
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<File, AppError>>>,
}

impl FileHandler {
    /// Handle the file
    #[cfg(target_arch = "wasm32")]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_local(async {
            let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
            if let Some(curr_file) = file_selected {
                let buf = curr_file.read().await;
                return Ok(File { data: buf });
            }
            // no file selected
            Err(AppError::new_fake("Upload: no file Selected".to_string()))
        }));
    }

    /// Handle the file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_thread("slow", move || {
            if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                // read file as string
                if let Some(path) = path_buf.to_str() {
                    let buf = std::fs::read(path);
                    let buf = match buf {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("{e:?}");
                            return Err(AppError::new(e.to_string()));
                        }
                    };
                    return Ok(File { data: buf });
                }
            }
            // no file selected
            Err(AppError::new_fake("Upload: no file Selected".to_string()))
        }))
    }

    /// Handle file upload
    fn handle_file_upload(&mut self) -> Result<Option<image::DynamicImage>, AppError> {
        let res = match &self.file_upload {
            Some(result) => match result.ready() {
                Some(Ok(File { data, .. })) => {
                    let res =
                        match ImageReader::new(std::io::Cursor::new(data)).with_guessed_format() {
                            Ok(img) => match img.decode() {
                                Ok(img) => Ok(Some(img)),
                                Err(err) => Err(err.into()),
                            },
                            Err(err) => Err(err.into()),
                        };
                    self.file_upload = None; // reset file_upload
                    res
                }
                Some(Err(e)) => {
                    let err = e.clone();
                    self.file_upload = None; // reset file_upload
                    Err(err)
                }
                None => Ok(None),
            },
            None => Ok(None),
        };
        res
    }

    /// Handle file dropped
    fn handle_file_dropped(&mut self) -> Result<Option<image::DynamicImage>, AppError> {
        if self.dropped_files.is_empty() {
            return Ok(None);
        }
        let file = self.dropped_files.remove(0);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = file.path.as_deref() {
                let img_reader = ImageReader::open(path)?;
                let img = img_reader.decode()?;
                return Ok(Some(img));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use std::io::Cursor;
            if let Some(bytes) = file.bytes.as_deref() {
                let img_reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
                let img = img_reader.decode()?;
                return Ok(Some(img));
            }
        }
        Ok(None)
    }

    /// Handle the files
    pub fn handle_files(
        &mut self,
        ctx: &egui::Context,
    ) -> Result<Option<image::DynamicImage>, AppError> {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // read the first file
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
        if let Some(img) = self.handle_file_upload()? {
            return Ok(Some(img));
        }
        if let Some(img) = self.handle_file_dropped()? {
            return Ok(Some(img));
        }
        Ok(None)
    }
}
