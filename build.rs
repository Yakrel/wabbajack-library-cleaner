// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

fn main() {
    // Check if the TARGET we are compiling for is Windows
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("winres/icon_main.ico");

        // Explicitly set windres tool for cross-compilation on Linux
        #[cfg(unix)]
        {
            res.set_toolkit_path("/usr/bin");
            res.set_windres_path("x86_64-w64-mingw32-windres");
            res.set_ar_path("x86_64-w64-mingw32-ar");
        }

        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
        }
    }
}
