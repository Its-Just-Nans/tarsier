#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use bladvak::app::Bladvak;
use tarsier::TarsierApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    Bladvak::<TarsierApp>::bladvak_main(&include_bytes!("../assets/icon-256.png")[..])
}

#[cfg(target_arch = "wasm32")]
fn main() {
    Bladvak::<TarsierApp>::bladvak_main()
}
