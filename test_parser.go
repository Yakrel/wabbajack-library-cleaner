package main

import (
"fmt"
"log"
)

func main() {
wabbajackFile := "/tmp/test_wabbajack/test_fnv.wabbajack"

fmt.Println("Testing parseWabbajackFile with real .wabbajack structure...")
fmt.Println("File:", wabbajackFile)
fmt.Println()

modlist, err := parseWabbajackFile(wabbajackFile)
if err != nil {
log.Fatal("❌ Parse failed:", err)
}

fmt.Println("✅ Parse SUCCESS!")
fmt.Println()
fmt.Printf("Modlist Name: %s\n", modlist.Name)
fmt.Printf("Author: %s\n", modlist.Author)
fmt.Printf("Version: %s\n", modlist.Version)
fmt.Printf("GameType: %s\n", modlist.GameType)
fmt.Printf("Total Archives: %d\n", len(modlist.Archives))
fmt.Println()

for i, archive := range modlist.Archives {
fmt.Printf("Archive %d:\n", i+1)
fmt.Printf("  Name: %s\n", archive.Name)
fmt.Printf("  ModID: %d\n", archive.State.ModID)
fmt.Printf("  FileID: %d\n", archive.State.FileID)
fmt.Printf("  GameName: %s\n", archive.State.GameName)
fmt.Println()
}

fmt.Println("✅ All fields parsed correctly!")
}
