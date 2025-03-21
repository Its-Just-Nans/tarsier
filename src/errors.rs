use std::{io, sync::Arc};

#[derive(Debug, Clone)]
enum ErrorType {
    Normal,
    Fake,
}

#[derive(Debug)]
pub struct TarsierError {
    pub message: String,
    pub source: Option<Arc<dyn std::error::Error + Send + Sync>>,
    error_type: ErrorType,
}

impl Clone for TarsierError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            source: self.source.clone(),
            error_type: self.error_type.clone(),
        }
    }
}

impl TarsierError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            source: None,
            error_type: ErrorType::Normal,
        }
    }

    pub fn new_fake(message: String) -> Self {
        Self {
            message,
            source: None,
            error_type: ErrorType::Fake,
        }
    }

    pub fn is_fake(&self) -> bool {
        matches!(self.error_type, ErrorType::Fake)
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
            error_type: ErrorType::Normal,
        }
    }
}

impl From<image::ImageError> for TarsierError {
    fn from(error: image::ImageError) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
            error_type: ErrorType::Normal,
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
        if error.is_fake() {
            return;
        }
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
