// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        // The icon path is relative to the Cargo.toml file
        // We assume the icon is at ../winres/icon_main.ico or we will point to it
        // Note: winres requires an .ico file for the main application icon
        res.set_icon("../winres/icon_main.ico");
        
        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
            // Don't fail the build if icon is missing, just print error
        }
    }
}
