package main

import (
"fmt"
"log"
)

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
}
