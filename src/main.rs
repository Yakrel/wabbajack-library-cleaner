#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use eframe::egui;
use egui::IconData;
use wabbajack_library_cleaner::gui::WabbajackCleanerApp;
use std::io::Cursor;

fn load_icon() -> Option<IconData> {
    // Embed the icon directly into the binary
    let icon_bytes = include_bytes!("../winres/icon_main.png");

    let image = match image::ImageReader::new(Cursor::new(icon_bytes)).with_guessed_format() {
        Ok(reader) => match reader.decode() {
            Ok(img) => img,
            Err(e) => {
                log::warn!("Failed to decode embedded icon: {}", e);
                return None;
            }
        },
        Err(e) => {
            log::warn!("Failed to guess embedded icon format: {}", e);
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
