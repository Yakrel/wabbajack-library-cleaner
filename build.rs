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
        
        // If we are cross-compiling from Linux, help winres find the tool
        // (Though strictly speaking usually not needed if in PATH, but safe to do)
        #[cfg(unix)]
        {
            // If the standard mingw windres is available, usage is automatic usually,
            // but we can enforce it if needed. For now let's rely on defaults 
            // as 'winres' is quite good at finding 'x86_64-w64-mingw32-windres'
        }

        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
        }
    }
}