use std::{io, sync::Arc};

#[derive(Debug)]
pub struct TarsierError {
    pub message: String,
    pub source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl Clone for TarsierError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            source: self.source.clone(),
        }
    }
}

impl TarsierError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            source: None,
        }
    }
}

impl From<String> for TarsierError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<io::Error> for TarsierError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<image::ImageError> for TarsierError {
    fn from(error: image::ImageError) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

#[derive(Debug, Default)]
pub struct ErrorManager {
    pub errors: Vec<TarsierError>,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_error(&mut self, error: TarsierError) {
        self.errors.push(error);
    }

    pub fn handle_error<T>(&mut self, error: Result<T, impl Into<TarsierError>>) -> Option<T> {
        match error {
            Ok(value) => Some(value),
            Err(e) => {
                self.add_error(e.into());
                None
            }
        }
    }
}
