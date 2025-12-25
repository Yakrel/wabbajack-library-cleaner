// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

mod core;
mod gui;

use eframe::egui;
use gui::WabbajackCleanerApp;

fn main() -> eframe::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();
    
    log::info!("=== Wabbajack Library Cleaner Started ===");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Wabbajack Library Cleaner"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Wabbajack Library Cleaner",
        options,
        Box::new(|cc| Ok(Box::new(WabbajackCleanerApp::new(cc)))),
    )
}
