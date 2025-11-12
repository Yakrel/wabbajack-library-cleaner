// Copyright (C) 2025 Berkay Yetgin
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//go:build windows

package main

import (
	"os"
	"syscall"
)

func enableWindowsColors() {
	// Enable ANSI color support on Windows 10+
	kernel32 := syscall.NewLazyDLL("kernel32.dll")
	setConsoleMode := kernel32.NewProc("SetConsoleMode")
	
	var mode uint32
	handle := syscall.Handle(os.Stdout.Fd())
	
	// Get current console mode
	syscall.GetConsoleMode(handle, &mode)
	
	// Enable VIRTUAL_TERMINAL_PROCESSING (0x0004)
	mode |= 0x0004
	
	// Set new console mode
	setConsoleMode.Call(uintptr(handle), uintptr(mode))
}
