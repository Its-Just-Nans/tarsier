#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod central_panel;
mod errors;
mod file;
mod side_panel;
mod top_panel;
mod windows;

pub use app::TarsierApp;
