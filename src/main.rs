use std::error::Error;

use eframe::egui::Vec2;

use crate::downloader::ui::Ui;

mod downloader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let mut options = eframe::NativeOptions::default();
    options.resizable = false;
    options.initial_window_size = Some(Vec2::new(305.0,300.0));
    eframe::run_native("Update Downloader", options, Box::new(|cc| Box::new(Ui::new(cc))))?;
    Ok(())
}
