#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use bladvak::app::Bladvak;
use tarsier::TarsierApp;

fn main() -> bladvak::MainResult {
    Bladvak::<TarsierApp>::bladvak_main()
}
