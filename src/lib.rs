//! Tarsier

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]

mod app;
mod central_panel;
mod errors;
mod file;
mod settings;
mod side_panel;
mod top_panel;
mod windows;

pub use app::TarsierApp;
