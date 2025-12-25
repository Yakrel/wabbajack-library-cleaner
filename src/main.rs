// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

mod core;
mod gui;

use std::path::Path;
use eframe::egui;
use egui::IconData;
use gui::WabbajackCleanerApp;

fn load_icon() -> Option<IconData> {
    // Try to load the icon from the winres directory relative to CWD
    let icon_path = Path::new("winres/icon_main.png");
    
    if !icon_path.exists() {
        log::warn!("Icon file not found at {:?}", icon_path);
        return None;
    }

    let image = match image::ImageReader::open(icon_path) {
        Ok(reader) => {
            match reader.with_guessed_format() {
                Ok(r) => match r.decode() {
                    Ok(img) => img,
                    Err(e) => {
                        log::warn!("Failed to decode icon file: {}", e);
                        return None;
                    }
                },
                Err(e) => {
                    log::warn!("Failed to guess image format: {}", e);
                    return None;
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to open icon file: {}", e);
            return None;
        }
    };

    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let rgba_data = rgba.into_raw();

    Some(IconData {
        rgba: rgba_data,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();
    
    log::info!("=== Wabbajack Library Cleaner Started ===");
    
    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Wabbajack Library Cleaner")
            .with_icon(icon.unwrap_or_default()), // Use loaded icon or default empty
        ..Default::default()
    };
    
    eframe::run_native(
        "Wabbajack Library Cleaner",
        options,
        Box::new(|cc| Ok(Box::new(WabbajackCleanerApp::new(cc)))),
    )
}
