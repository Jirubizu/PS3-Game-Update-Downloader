use std::sync::{Arc, Mutex};
use std::thread;

use eframe::{egui, Frame};
use eframe::egui::{Button, Context, Vec2};

use crate::downloader::{Downloader, PatchTitle};

// Design :)
/*
    ========================
    |       Downloader     |
    ------------------------
    | [       input      ] |
    |----------------------|
    |        (v1.05)       |
    |        (v1.20)       |
    |        (v1.40)       |
    |        (v1.50)       |
    |----------------------|
    |     (Find Updates)   |
    ========================
*/

pub struct Ui {
    search_term: String,
    patch: Arc<Mutex<PatchTitle>>,
    downloader: Downloader,
    widget_size: Vec2,
}

impl Ui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            search_term: "".to_string(),
            patch: Arc::new(Mutex::new(PatchTitle { title: "".to_string(), packages: Default::default() })),
            downloader: Downloader::default(),
            widget_size: Vec2{ x: 290.0, y: 20.0 },
        }
    }

    fn is_downloading(&self) -> (bool, f32) {
        let progress = *self.downloader.download_progress.lock().unwrap();
        if !(progress <= 0.0) {
            return (true, progress);
        }
        return (false, progress);
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_sized(self.widget_size, egui::TextEdit::singleline(&mut self.search_term));
            {
                let tmp = self.patch.lock().unwrap();
                ui.label(&tmp.title);
                if !&tmp.title.is_empty() {
                    ui.add_sized(self.widget_size, egui::Separator::default());
                }
            }

            ui.vertical(|vui| {
                let title = &self.patch.lock().unwrap();
                for (_, package) in &title.packages {
                    let size: f64 = (package.size.parse::<u64>().unwrap() / 1_000_000) as f64;
                    let formatted_text = format!("Version: {}, {:2} MB", package.version, size);
                    if vui.add_enabled(!self.is_downloading().0, Button::new(formatted_text).min_size(self.widget_size)).clicked() {
                        let downloader = self.downloader.clone();
                        let url = package.url.clone();
                        let filename = format!("{}-v{}.pkg", title.title, package.version);
                        thread::spawn(move || {
                            downloader.download_file(filename,url);
                        });
                    };
                }
            });

            {
                let tmp = self.patch.lock().unwrap();
                if !&tmp.title.is_empty() {
                    ui.add_sized(self.widget_size, egui::Separator::default());
                }
            }
             egui::TopBottomPanel::bottom("bot_panel").exact_height(30.0).show_separator_line(false).show(ctx, |ui| {
                 ui.with_layout(egui::Layout::bottom_up(egui::Align::BOTTOM), |lui| {
                     lui.vertical(|hui| {
                         let downloading = self.is_downloading();
                         if downloading.0 {
                             let _ = hui.add(egui::ProgressBar::new(downloading.1).animate(true));
                         } else {
                             if hui.add_sized(self.widget_size, Button::new("Find Updates")).clicked() {
                                 let search_term = self.search_term.clone();
                                 let downloader = self.downloader.clone();
                                 let patch = Arc::clone(&self.patch);
                                 thread::spawn(move || {
                                     let mut p = patch.lock().unwrap();
                                     *p = downloader.find(search_term).unwrap();
                                 });
                             };
                         }
                     });
                 });
             })
        });
    }
}