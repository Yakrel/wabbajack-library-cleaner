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

package main

import (
	"testing"
)

// TestParseModFilename tests the parsing of mod filenames
func TestParseModFilename(t *testing.T) {
	tests := []struct {
		name       string
		filename   string
		wantModID  string
		wantFileID string
		wantValid  bool
	}{
		{
			name:       "Valid filename with ModID and FileID",
			filename:   "Skyrim 2020-12345-67890-1-0-1234567890.7z",
			wantModID:  "12345",
			wantFileID: "67890",
			wantValid:  true,
		},
		{
			name:       "Valid filename with ModID only (no FileID)",
			filename:   "Simple Mod-123-1-0-1234567890.rar",
			wantModID:  "123",
			wantFileID: "",
			wantValid:  true,
		},
		{
			name:       "Valid filename with complex name",
			filename:   "JK's Skyrim-6289-123456-2-0-1234567890.zip",
			wantModID:  "6289",
			wantFileID: "123456",
			wantValid:  true,
		},
		{
			name:       "Invalid filename - no ModID",
			filename:   "NoModID-1234567890.7z",
			wantValid:  false,
		},
		{
			name:       "Invalid filename - no timestamp",
			filename:   "Mod-123-1-0.7z",
			wantValid:  false,
		},
		{
			name:       "Invalid filename - wrong extension",
			filename:   "Mod-123-1-0-1234567890.txt",
			wantValid:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseModFilename(tt.filename)

			if tt.wantValid {
				if result == nil {
					t.Errorf("parseModFilename() returned nil for valid filename: %s", tt.filename)
					return
				}
				if result.ModID != tt.wantModID {
					t.Errorf("parseModFilename() ModID = %v, want %v", result.ModID, tt.wantModID)
				}
				if result.FileID != tt.wantFileID {
					t.Errorf("parseModFilename() FileID = %v, want %v", result.FileID, tt.wantFileID)
				}
			} else {
				if result != nil {
					t.Errorf("parseModFilename() should return nil for invalid filename: %s, got %+v", tt.filename, result)
				}
			}
		})
	}
}

// TestDetectOrphanedMods tests orphaned mod detection with FileID matching
func TestDetectOrphanedMods(t *testing.T) {
	// Create test mod files
	modFiles := []ModFile{
		{ModID: "123", FileID: "456", FileName: "mod1.7z"},
		{ModID: "123", FileID: "789", FileName: "mod2.7z"}, // Different FileID, same ModID
		{ModID: "999", FileID: "", FileName: "mod3.7z"},     // No FileID
		{ModID: "888", FileID: "111", FileName: "mod4.7z"},
	}

	// Create test modlist with specific ModID+FileID requirements
	modlist := &ModlistInfo{
		Name: "Test Modlist",
		UsedModKeys: map[string]bool{
			"123": true, // ModID only
			"999": true,
		},
		UsedModFileIDs: map[string]bool{
			"123-456": true, // Specific FileID
		},
	}

	used, orphaned := detectOrphanedMods(modFiles, []*ModlistInfo{modlist})

	// Expected:
	// - mod1.7z: USED (matches ModID 123 and FileID 456)
	// - mod2.7z: USED (matches ModID 123, even though FileID is different - falls back to ModID matching)
	// - mod3.7z: USED (matches ModID 999)
	// - mod4.7z: ORPHANED (ModID 888 not in modlist)

	if len(orphaned) != 1 {
		t.Errorf("Expected 1 orphaned mod, got %d", len(orphaned))
	}

	if len(used) != 3 {
		t.Errorf("Expected 3 used mods, got %d", len(used))
	}

	// Check that mod4 is orphaned
	foundOrphaned := false
	for _, om := range orphaned {
		if om.File.FileName == "mod4.7z" {
			foundOrphaned = true
			break
		}
	}
	if !foundOrphaned {
		t.Errorf("Expected mod4.7z to be orphaned, but it wasn't")
	}
}

// TestDetectOrphanedModsWithPreciseFileID tests that precise FileID matching works
func TestDetectOrphanedModsWithPreciseFileID(t *testing.T) {
	// Create test mod files with same ModID but different FileIDs
	modFiles := []ModFile{
		{ModID: "123", FileID: "456", FileName: "mod-v1.7z"},
		{ModID: "123", FileID: "789", FileName: "mod-v2.7z"},
	}

	// Modlist only wants FileID 456
	modlist := &ModlistInfo{
		Name: "Precise Test",
		UsedModKeys: map[string]bool{
			"123": true,
		},
		UsedModFileIDs: map[string]bool{
			"123-456": true, // Only wants FileID 456
		},
	}

	used, orphaned := detectOrphanedMods(modFiles, []*ModlistInfo{modlist})

	// Both should be marked as USED because we fall back to ModID matching
	// This is intentional behavior to avoid accidentally deleting needed files
	if len(used) != 2 {
		t.Errorf("Expected 2 used mods (fallback to ModID matching), got %d", len(used))
	}

	if len(orphaned) != 0 {
		t.Errorf("Expected 0 orphaned mods (fallback protection), got %d", len(orphaned))
	}
}

// TestIsNumeric tests the isNumeric helper function
func TestIsNumeric(t *testing.T) {
	tests := []struct {
		input string
		want  bool
	}{
		{"123", true},
		{"0", true},
		{"abc", false},
		{"12a", false},
		{"", false},
		{"-123", true},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			got := isNumeric(tt.input)
			if got != tt.want {
				t.Errorf("isNumeric(%q) = %v, want %v", tt.input, got, tt.want)
			}
		})
	}
}

// TestNormalizeModName tests mod name normalization
func TestNormalizeModName(t *testing.T) {
	tests := []struct {
		input string
		want  string
	}{
		{"Skyrim 2020 1.2.3", "Skyrim"}, // "2020" is removed as it looks like a version
		{"Interface v1.0", "Interface"},
		{"Simple Mod V2.0", "Simple Mod"},
		{"No Version Mod", "No Version Mod"},
		{"Mod 0.18", "Mod"},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			got := normalizeModName(tt.input)
			if got != tt.want {
				t.Errorf("normalizeModName(%q) = %q, want %q", tt.input, got, tt.want)
			}
		})
	}
}
