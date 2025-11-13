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
	"fmt"
	"path/filepath"
	"strings"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/widget"
)

// GUIApp holds the GUI application state
type GUIApp struct {
	app             fyne.App
	window          fyne.Window
	wabbajackDir    string
	downloadsDir    string
	useRecycleBin   bool
	outputText      *widget.Entry
	statusLabel     *widget.Label
	wabbajackLabel  *widget.Label
	downloadsLabel  *widget.Label
	recycleBinCheck *widget.Check
}

// NewGUIApp creates and initializes the GUI application
func NewGUIApp() *GUIApp {
	a := app.NewWithID("com.yakrel.wabbajack-library-cleaner")
	w := a.NewWindow("Wabbajack Library Cleaner")
	w.Resize(fyne.NewSize(900, 700))

	guiApp := &GUIApp{
		app:    a,
		window: w,
	}

	guiApp.setupUI()
	return guiApp
}

// setupUI creates the user interface
func (g *GUIApp) setupUI() {
	// Title
	title := widget.NewLabelWithStyle(
		"Wabbajack Library Cleaner",
		fyne.TextAlignCenter,
		fyne.TextStyle{Bold: true},
	)

	// Directory selection section
	g.wabbajackLabel = widget.NewLabel("Wabbajack Directory: (Not selected)")
	selectWabbajackBtn := widget.NewButton("Select Wabbajack Directory", func() {
		g.selectWabbajackDir()
	})

	g.downloadsLabel = widget.NewLabel("Downloads Directory: (Not selected)")
	selectDownloadsBtn := widget.NewButton("Select Downloads Directory", func() {
		g.selectDownloadsDir()
	})

	dirSection := container.NewVBox(
		widget.NewLabelWithStyle("Step 1: Select Directories", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabel("Select the directory containing .wabbajack files:"),
		g.wabbajackLabel,
		selectWabbajackBtn,
		widget.NewSeparator(),
		widget.NewLabel("Select the directory containing game mod folders (Skyrim, Fallout4, etc.):"),
		g.downloadsLabel,
		selectDownloadsBtn,
	)

	// Options section
	g.recycleBinCheck = widget.NewCheck("Send deleted files to Recycle Bin (instead of permanent deletion)", func(checked bool) {
		g.useRecycleBin = checked
	})
	g.recycleBinCheck.SetChecked(true) // Default to recycle bin for safety
	g.useRecycleBin = true

	optionsSection := container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Step 2: Options", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		g.recycleBinCheck,
	)

	// Action buttons section
	scanOldVersionsBtn := widget.NewButton("Scan for Old Versions (Dry-run)", func() {
		g.scanOldVersions(false)
	})

	cleanOldVersionsBtn := widget.NewButton("Clean Old Versions", func() {
		g.scanOldVersions(true)
	})

	scanOrphanedBtn := widget.NewButton("Scan for Orphaned Mods (Dry-run)", func() {
		g.scanOrphanedMods(false)
	})

	cleanOrphanedBtn := widget.NewButton("Clean Orphaned Mods", func() {
		g.scanOrphanedMods(true)
	})

	statsBtn := widget.NewButton("View Statistics", func() {
		g.viewStats()
	})

	actionsSection := container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Step 3: Actions", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		container.NewGridWithColumns(2,
			scanOldVersionsBtn,
			cleanOldVersionsBtn,
		),
		container.NewGridWithColumns(2,
			scanOrphanedBtn,
			cleanOrphanedBtn,
		),
		statsBtn,
	)

	// Output section
	g.outputText = widget.NewMultiLineEntry()
	g.outputText.SetPlaceHolder("Output will appear here...")
	g.outputText.Wrapping = fyne.TextWrapWord
	outputScroll := container.NewScroll(g.outputText)
	outputScroll.SetMinSize(fyne.NewSize(850, 300))

	clearBtn := widget.NewButton("Clear Output", func() {
		g.outputText.SetText("")
	})

	outputSection := container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Output", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		outputScroll,
		clearBtn,
	)

	// Status bar
	g.statusLabel = widget.NewLabel("Ready")

	// Main layout
	content := container.NewVBox(
		title,
		widget.NewSeparator(),
		dirSection,
		optionsSection,
		actionsSection,
		outputSection,
		widget.NewSeparator(),
		g.statusLabel,
	)

	scrollContent := container.NewScroll(content)
	g.window.SetContent(scrollContent)
}

// appendOutput adds text to the output area
func (g *GUIApp) appendOutput(text string) {
	current := g.outputText.Text
	if current != "" {
		current += "\n"
	}
	g.outputText.SetText(current + text)
	g.outputText.CursorRow = len(strings.Split(g.outputText.Text, "\n"))
}

// setStatus updates the status label
func (g *GUIApp) setStatus(text string) {
	g.statusLabel.SetText(text)
}

// selectWabbajackDir opens a dialog to select the wabbajack directory
func (g *GUIApp) selectWabbajackDir() {
	dialog.ShowFolderOpen(func(uri fyne.ListableURI, err error) {
		if err != nil {
			dialog.ShowError(err, g.window)
			return
		}
		if uri == nil {
			return
		}
		g.wabbajackDir = uri.Path()
		g.wabbajackLabel.SetText("Wabbajack Directory: " + g.wabbajackDir)
		g.appendOutput("Selected wabbajack directory: " + g.wabbajackDir)
		logInfo("User selected wabbajack directory: %s", g.wabbajackDir)
	}, g.window)
}

// selectDownloadsDir opens a dialog to select the downloads directory
func (g *GUIApp) selectDownloadsDir() {
	dialog.ShowFolderOpen(func(uri fyne.ListableURI, err error) {
		if err != nil {
			dialog.ShowError(err, g.window)
			return
		}
		if uri == nil {
			return
		}
		g.downloadsDir = uri.Path()
		g.downloadsLabel.SetText("Downloads Directory: " + g.downloadsDir)
		g.appendOutput("Selected downloads directory: " + g.downloadsDir)
		logInfo("User selected downloads directory: %s", g.downloadsDir)
	}, g.window)
}

// validateDirectories checks if required directories are selected
func (g *GUIApp) validateDirectories() bool {
	if g.downloadsDir == "" {
		dialog.ShowError(fmt.Errorf("Please select the Downloads Directory first"), g.window)
		return false
	}
	return true
}

// scanOldVersions handles old version scanning/cleaning
func (g *GUIApp) scanOldVersions(deleteMode bool) {
	if !g.validateDirectories() {
		return
	}

	g.setStatus("Scanning for old versions...")
	g.appendOutput("\n=== Scanning for Old Versions ===")

	gameFolders, err := getGameFolders(g.downloadsDir)
	if err != nil || len(gameFolders) == 0 {
		dialog.ShowError(fmt.Errorf("No game folders found in: %s", g.downloadsDir), g.window)
		g.setStatus("Error: No game folders found")
		return
	}

	g.appendOutput(fmt.Sprintf("Found %d game folder(s):", len(gameFolders)))
	for _, folder := range gameFolders {
		g.appendOutput("  - " + filepath.Base(folder))
	}

	// Show folder selection dialog
	folderNames := make([]string, len(gameFolders))
	for i, folder := range gameFolders {
		folderNames[i] = filepath.Base(folder)
	}

	var selectedFolder string
	folderSelect := widget.NewSelect(folderNames, func(value string) {
		selectedFolder = value
	})
	folderSelect.PlaceHolder = "Select a folder to scan..."

	confirmDialog := dialog.NewCustomConfirm(
		"Select Game Folder",
		"Scan",
		"Cancel",
		container.NewVBox(
			widget.NewLabel("Select which game folder to scan:"),
			folderSelect,
		),
		func(confirmed bool) {
			if !confirmed || selectedFolder == "" {
				g.setStatus("Cancelled")
				return
			}

			// Find the full path
			var folderPath string
			for _, folder := range gameFolders {
				if filepath.Base(folder) == selectedFolder {
					folderPath = folder
					break
				}
			}

			g.performOldVersionScan(folderPath, deleteMode)
		},
		g.window,
	)
	confirmDialog.Resize(fyne.NewSize(400, 200))
	confirmDialog.Show()
}

// performOldVersionScan performs the actual scanning
func (g *GUIApp) performOldVersionScan(folderPath string, deleteMode bool) {
	g.appendOutput(fmt.Sprintf("\nScanning: %s", filepath.Base(folderPath)))

	duplicates, err := scanFolder(folderPath)
	if err != nil {
		dialog.ShowError(fmt.Errorf("Scan failed: %v", err), g.window)
		g.setStatus("Scan failed")
		return
	}

	if len(duplicates) == 0 {
		g.appendOutput("No old versions found!")
		g.setStatus("No old versions found")
		dialog.ShowInformation("Scan Complete", "No old versions found!", g.window)
		return
	}

	// Calculate totals
	totalFiles := 0
	totalSpace := int64(0)
	for _, group := range duplicates {
		totalFiles += len(group.Files) - 1
		totalSpace += group.SpaceToFree
	}

	g.appendOutput(fmt.Sprintf("\nFound %d groups with old versions", len(duplicates)))
	g.appendOutput(fmt.Sprintf("Total: %d old files, %s to free", totalFiles, formatSize(totalSpace)))

	// Show some examples
	count := 0
	for _, group := range duplicates {
		if count >= 5 {
			g.appendOutput(fmt.Sprintf("... and %d more groups", len(duplicates)-5))
			break
		}
		newest := group.Files[group.NewestIdx]
		g.appendOutput(fmt.Sprintf("\n%s - %s", group.ModKey, newest.FileName))
		g.appendOutput(fmt.Sprintf("  Will delete %d old version(s), saving %s",
			len(group.Files)-1, formatSize(group.SpaceToFree)))
		count++
	}

	if deleteMode {
		g.confirmAndDelete(func() {
			deleted, freed := g.deleteOldVersionsWithRecycleBin(duplicates)
			g.appendOutput(fmt.Sprintf("\nDeleted: %d files", deleted))
			g.appendOutput(fmt.Sprintf("Space freed: %s", formatSize(freed)))
			g.setStatus(fmt.Sprintf("Completed: %d files deleted", deleted))

			dialog.ShowInformation("Cleanup Complete",
				fmt.Sprintf("Deleted %d files\nSpace freed: %s", deleted, formatSize(freed)),
				g.window)
		})
	} else {
		g.setStatus(fmt.Sprintf("Scan complete: %d old files found", totalFiles))
		dialog.ShowInformation("Scan Complete",
			fmt.Sprintf("Found %d old files\nPotential space: %s", totalFiles, formatSize(totalSpace)),
			g.window)
	}
}

// scanOrphanedMods handles orphaned mods scanning/cleaning
func (g *GUIApp) scanOrphanedMods(deleteMode bool) {
	if !g.validateDirectories() {
		return
	}

	if g.wabbajackDir == "" {
		dialog.ShowError(fmt.Errorf("Please select the Wabbajack Directory first"), g.window)
		return
	}

	g.setStatus("Scanning for orphaned mods...")
	g.appendOutput("\n=== Scanning for Orphaned Mods ===")

	// Find wabbajack files
	wabbajackFiles, err := findWabbajackFiles(g.wabbajackDir)
	if err != nil || len(wabbajackFiles) == 0 {
		dialog.ShowError(fmt.Errorf("No .wabbajack files found in: %s", g.wabbajackDir), g.window)
		g.setStatus("Error: No wabbajack files found")
		return
	}

	g.appendOutput(fmt.Sprintf("Found %d modlist file(s)", len(wabbajackFiles)))

	// Parse modlists
	var modlistInfos []*ModlistInfo
	for _, wbFile := range wabbajackFiles {
		info, err := parseWabbajackFile(wbFile)
		if err != nil {
			g.appendOutput(fmt.Sprintf("Failed to parse %s: %v", filepath.Base(wbFile), err))
			continue
		}
		g.appendOutput(fmt.Sprintf("  ✓ %s (%d mods)", info.Name, info.ModCount))
		modlistInfos = append(modlistInfos, info)
	}

	if len(modlistInfos) == 0 {
		dialog.ShowError(fmt.Errorf("Failed to parse any modlist files"), g.window)
		g.setStatus("Error: No valid modlists")
		return
	}

	// Show modlist selection dialog
	g.showModlistSelection(modlistInfos, deleteMode)
}

// showModlistSelection shows a dialog to select active modlists
func (g *GUIApp) showModlistSelection(modlistInfos []*ModlistInfo, deleteMode bool) {
	checks := make([]*widget.Check, len(modlistInfos))
	checkBoxes := container.NewVBox()

	for i, info := range modlistInfos {
		idx := i
		checks[i] = widget.NewCheck(fmt.Sprintf("%s (%d mods)", info.Name, info.ModCount), nil)
		checkBoxes.Add(checks[idx])
	}

	selectAllBtn := widget.NewButton("Select All", func() {
		for _, check := range checks {
			check.SetChecked(true)
		}
	})

	deselectAllBtn := widget.NewButton("Deselect All", func() {
		for _, check := range checks {
			check.SetChecked(false)
		}
	})

	buttonRow := container.NewGridWithColumns(2, selectAllBtn, deselectAllBtn)

	content := container.NewVBox(
		widget.NewLabel("Select which modlists you are CURRENTLY USING:"),
		widget.NewSeparator(),
		checkBoxes,
		widget.NewSeparator(),
		buttonRow,
	)

	scrollContent := container.NewScroll(content)
	scrollContent.SetMinSize(fyne.NewSize(400, 300))

	confirmDialog := dialog.NewCustomConfirm(
		"Select Active Modlists",
		"Continue",
		"Cancel",
		scrollContent,
		func(confirmed bool) {
			if !confirmed {
				g.setStatus("Cancelled")
				return
			}

			// Get selected modlists
			var activeModlists []*ModlistInfo
			for i, check := range checks {
				if check.Checked {
					activeModlists = append(activeModlists, modlistInfos[i])
				}
			}

			if len(activeModlists) == 0 {
				dialog.ShowError(fmt.Errorf("Please select at least one modlist"), g.window)
				return
			}

			g.performOrphanedScan(activeModlists, deleteMode)
		},
		g.window,
	)
	confirmDialog.Resize(fyne.NewSize(500, 500))
	confirmDialog.Show()
}

// performOrphanedScan performs the actual orphaned mods scan
func (g *GUIApp) performOrphanedScan(activeModlists []*ModlistInfo, deleteMode bool) {
	g.appendOutput("\nActive modlists:")
	for _, ml := range activeModlists {
		g.appendOutput("  ✓ " + ml.Name)
	}

	// Get game folders
	gameFolders, err := getGameFolders(g.downloadsDir)
	if err != nil {
		dialog.ShowError(err, g.window)
		return
	}

	// Collect all mod files
	g.appendOutput("\nCollecting mod files...")
	allModFiles, err := getAllModFiles(gameFolders)
	if err != nil {
		dialog.ShowError(err, g.window)
		return
	}
	g.appendOutput(fmt.Sprintf("Found %d mod files", len(allModFiles)))

	// Detect orphaned mods
	g.appendOutput("Analyzing...")
	usedMods, orphanedMods := detectOrphanedMods(allModFiles, activeModlists)

	// Calculate sizes
	usedSize := int64(0)
	for _, mod := range usedMods {
		usedSize += mod.Size
	}

	orphanedSize := int64(0)
	for _, om := range orphanedMods {
		orphanedSize += om.File.Size
	}

	g.appendOutput("\n=== RESULTS ===")
	g.appendOutput(fmt.Sprintf("✓ USED MODS: %d files (%s)", len(usedMods), formatSize(usedSize)))
	g.appendOutput(fmt.Sprintf("✗ ORPHANED MODS: %d files (%s)", len(orphanedMods), formatSize(orphanedSize)))

	if len(orphanedMods) == 0 {
		g.setStatus("No orphaned mods found")
		dialog.ShowInformation("Scan Complete", "No orphaned mods found!", g.window)
		return
	}

	// Show examples
	exampleCount := len(orphanedMods)
	if exampleCount > 10 {
		exampleCount = 10
	}
	g.appendOutput("\nExamples:")
	for i := 0; i < exampleCount; i++ {
		om := orphanedMods[i]
		g.appendOutput(fmt.Sprintf("  • %s (%s)", om.File.FileName, formatSize(om.File.Size)))
	}
	if len(orphanedMods) > 10 {
		g.appendOutput(fmt.Sprintf("  ... and %d more", len(orphanedMods)-10))
	}

	if deleteMode {
		g.confirmAndDelete(func() {
			deleted, freed := g.deleteOrphanedModsWithRecycleBin(orphanedMods)
			g.appendOutput(fmt.Sprintf("\nDeleted: %d files", deleted))
			g.appendOutput(fmt.Sprintf("Space freed: %s", formatSize(freed)))
			g.setStatus(fmt.Sprintf("Completed: %d files deleted", deleted))

			dialog.ShowInformation("Cleanup Complete",
				fmt.Sprintf("Deleted %d files\nSpace freed: %s", deleted, formatSize(freed)),
				g.window)
		})
	} else {
		g.setStatus(fmt.Sprintf("Scan complete: %d orphaned mods", len(orphanedMods)))
		dialog.ShowInformation("Scan Complete",
			fmt.Sprintf("Found %d orphaned mods\nPotential space: %s", len(orphanedMods), formatSize(orphanedSize)),
			g.window)
	}
}

// viewStats displays statistics
func (g *GUIApp) viewStats() {
	if !g.validateDirectories() {
		return
	}

	g.setStatus("Calculating statistics...")
	g.appendOutput("\n=== Library Statistics ===")

	gameFolders, err := getGameFolders(g.downloadsDir)
	if err != nil {
		dialog.ShowError(err, g.window)
		return
	}

	totalFiles := 0
	totalSize := int64(0)
	stats := make(map[string]struct {
		files int
		size  int64
	})

	for _, folder := range gameFolders {
		entries, err := getFilesInFolder(folder)
		if err != nil {
			continue
		}

		gameFiles := 0
		gameSize := int64(0)

		for _, entry := range entries {
			if entry.IsDir() {
				continue
			}

			filename := entry.Name()
			if !isWabbajackFile(filename) {
				continue
			}

			info, err := entry.Info()
			if err != nil {
				continue
			}

			gameFiles++
			gameSize += info.Size()
		}

		gameName := filepath.Base(folder)
		stats[gameName] = struct {
			files int
			size  int64
		}{gameFiles, gameSize}

		totalFiles += gameFiles
		totalSize += gameSize
	}

	g.appendOutput(fmt.Sprintf("\nTotal Files: %d", totalFiles))
	g.appendOutput(fmt.Sprintf("Total Size: %s", formatSize(totalSize)))
	g.appendOutput("\nBy Game:")

	for gameName, stat := range stats {
		g.appendOutput(fmt.Sprintf("  %s: %d files (%s)", gameName, stat.files, formatSize(stat.size)))
	}

	g.setStatus("Statistics calculated")
	dialog.ShowInformation("Statistics",
		fmt.Sprintf("Total: %d files (%s)", totalFiles, formatSize(totalSize)),
		g.window)
}

// confirmAndDelete shows confirmation dialog before deletion
func (g *GUIApp) confirmAndDelete(onConfirm func()) {
	var confirmText string
	if g.useRecycleBin {
		confirmText = "Are you sure you want to send these files to the Recycle Bin?"
	} else {
		confirmText = "⚠️ WARNING ⚠️\n\nThis will PERMANENTLY DELETE files!\n\nAre you absolutely sure?"
	}

	dialog.ShowConfirm(
		"Confirm Deletion",
		confirmText,
		func(confirmed bool) {
			if confirmed {
				onConfirm()
			} else {
				g.setStatus("Cancelled")
			}
		},
		g.window,
	)
}

// deleteOldVersionsWithRecycleBin deletes old versions with recycle bin option
func (g *GUIApp) deleteOldVersionsWithRecycleBin(duplicates map[string]*ModGroup) (int, int64) {
	deletedCount := 0
	spaceFreed := int64(0)

	for _, group := range duplicates {
		for i := 0; i < group.NewestIdx; i++ {
			file := group.Files[i]

			if isFileLocked(file.FullPath) {
				g.appendOutput(fmt.Sprintf("⚠ Skipped (locked): %s", file.FileName))
				continue
			}

			var err error
			if g.useRecycleBin {
				err = moveToRecycleBin(file.FullPath)
			} else {
				err = deleteFile(file.FullPath)
			}

			if err != nil {
				g.appendOutput(fmt.Sprintf("✗ Failed to delete: %s - %v", file.FileName, err))
				logError("Failed to delete %s: %v", file.FullPath, err)
				continue
			}

			deletedCount++
			spaceFreed += file.Size
			g.appendOutput(fmt.Sprintf("✓ Deleted: %s (%s)", file.FileName, formatSize(file.Size)))
			logInfo("Deleted: %s (%s)", file.FileName, formatSize(file.Size))

			// Delete .meta file
			metaPath := file.FullPath + ".meta"
			if fileExists(metaPath) {
				if g.useRecycleBin {
					moveToRecycleBin(metaPath)
				} else {
					deleteFile(metaPath)
				}
			}
		}
	}

	return deletedCount, spaceFreed
}

// deleteOrphanedModsWithRecycleBin deletes orphaned mods with recycle bin option
func (g *GUIApp) deleteOrphanedModsWithRecycleBin(orphanedMods []OrphanedMod) (int, int64) {
	deletedCount := 0
	spaceFreed := int64(0)

	for _, om := range orphanedMods {
		file := om.File

		if isFileLocked(file.FullPath) {
			g.appendOutput(fmt.Sprintf("⚠ Skipped (locked): %s", file.FileName))
			continue
		}

		var err error
		if g.useRecycleBin {
			err = moveToRecycleBin(file.FullPath)
		} else {
			err = deleteFile(file.FullPath)
		}

		if err != nil {
			g.appendOutput(fmt.Sprintf("✗ Failed to delete: %s - %v", file.FileName, err))
			logError("Failed to delete %s: %v", file.FullPath, err)
			continue
		}

		deletedCount++
		spaceFreed += file.Size
		g.appendOutput(fmt.Sprintf("✓ Deleted: %s (%s)", file.FileName, formatSize(file.Size)))
		logInfo("Deleted: %s (%s)", file.FileName, formatSize(file.Size))

		// Delete .meta file
		metaPath := file.FullPath + ".meta"
		if fileExists(metaPath) {
			if g.useRecycleBin {
				moveToRecycleBin(metaPath)
			} else {
				deleteFile(metaPath)
			}
		}
	}

	return deletedCount, spaceFreed
}

// Run starts the GUI application
func (g *GUIApp) Run() {
	g.window.ShowAndRun()
}
