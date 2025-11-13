//go:build ignore

package main

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"io"
	"log"
)

type ModlistModState struct {
	Type     string `json:"$type"`
	ModID    int64  `json:"ModID"`
	FileID   int64  `json:"FileID"`
	GameName string `json:"GameName"`
	Name     string `json:"Name"`
	Version  string `json:"Version"`
}

type ModlistArchive struct {
	State ModlistModState `json:"State"`
	Name  string          `json:"Name"`
	Size  int64           `json:"Size"`
	Hash  string          `json:"Hash"`
}

type Modlist struct {
	Name        string           `json:"Name"`
	Author      string           `json:"Author"`
	Description string           `json:"Description"`
	Archives    []ModlistArchive `json:"Archives"`
}

func parseWabbajackFile(filename string) (*Modlist, error) {
	r, err := zip.OpenReader(filename)
	if err != nil {
		return nil, err
	}
	defer r.Close()

	for _, f := range r.File {
		if f.Name == "modlist" {
			rc, err := f.Open()
			if err != nil {
				return nil, err
			}
			defer rc.Close()

			data, err := io.ReadAll(rc)
			if err != nil {
				return nil, err
			}

			var modlist Modlist
			if err := json.Unmarshal(data, &modlist); err != nil {
				return nil, err
			}

			return &modlist, nil
		}
	}

	return nil, fmt.Errorf("modlist file not found in archive")
}

func main() {
	fmt.Println("🧪 Testing .wabbajack parser...")

	modlist, err := parseWabbajackFile("test.wabbajack")
	if err != nil {
		log.Fatal("❌ Parser failed:", err)
	}

	fmt.Printf("✅ Parser SUCCESS!\n\n")
	fmt.Printf("Modlist Name: %s\n", modlist.Name)
	fmt.Printf("Author: %s\n", modlist.Author)
	fmt.Printf("Total Archives: %d\n\n", len(modlist.Archives))

	fmt.Println("📋 Found mods:")
	for i, archive := range modlist.Archives {
		fmt.Printf("  %d. %s\n", i+1, archive.Name)
		if archive.State.ModID > 0 {
			fmt.Printf("     ModID: %d, FileID: %d, Game: %s\n",
				archive.State.ModID, archive.State.FileID, archive.State.GameName)
		}
	}

	fmt.Println("\n✅ Our parser works correctly!")
}
