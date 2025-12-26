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
	"os"
	"path/filepath"
	"testing"
)

// TestIsValidPath tests path validation
func TestIsValidPath(t *testing.T) {
	// Create a temporary directory for testing
	tempDir := t.TempDir()

	// Create a temporary file
	tempFile := filepath.Join(tempDir, "testfile.txt")
	if err := os.WriteFile(tempFile, []byte("test"), 0644); err != nil {
		t.Fatal(err)
	}

	tests := []struct {
		name string
		path string
		want bool
	}{
		{
			name: "Valid directory",
			path: tempDir,
			want: true,
		},
		{
			name: "Invalid - empty path",
			path: "",
			want: false,
		},
		{
			name: "Invalid - whitespace only",
			path: "   ",
			want: false,
		},
		{
			name: "Invalid - non-existent path",
			path: "/non/existent/path/12345",
			want: false,
		},
		{
			name: "Invalid - file instead of directory",
			path: tempFile,
			want: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := isValidPath(tt.path)
			if got != tt.want {
				t.Errorf("isValidPath(%q) = %v, want %v", tt.path, got, tt.want)
			}
		})
	}
}

// TestFileExists tests file existence checking
func TestFileExists(t *testing.T) {
	tempDir := t.TempDir()

	// Create a test file
	testFile := filepath.Join(tempDir, "exists.txt")
	if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
		t.Fatal(err)
	}

	tests := []struct {
		name string
		path string
		want bool
	}{
		{
			name: "Existing file",
			path: testFile,
			want: true,
		},
		{
			name: "Non-existing file",
			path: filepath.Join(tempDir, "notexists.txt"),
			want: false,
		},
		{
			name: "Existing directory",
			path: tempDir,
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := fileExists(tt.path)
			if got != tt.want {
				t.Errorf("fileExists(%q) = %v, want %v", tt.path, got, tt.want)
			}
		})
	}
}

// TestDeleteFile tests file deletion
func TestDeleteFile(t *testing.T) {
	tempDir := t.TempDir()

	// Create a test file
	testFile := filepath.Join(tempDir, "todelete.txt")
	if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
		t.Fatal(err)
	}

	// Verify file exists
	if !fileExists(testFile) {
		t.Fatal("Test file was not created")
	}

	// Delete the file
	err := deleteFile(testFile)
	if err != nil {
		t.Errorf("deleteFile() error = %v", err)
	}

	// Verify file no longer exists
	if fileExists(testFile) {
		t.Error("File still exists after deletion")
	}
}
