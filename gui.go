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
	"os"
	"path/filepath"
	"strings"
	"time"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/widget"
	nativedialog "github.com/sqweek/dialog"
)

// GUIApp holds the GUI application state
type GUIApp struct {
	app               fyne.App
	window            fyne.Window
	wabbajackDir      string
	downloadsDir      string
	moveToBackup      bool
	modlistInfos      []*ModlistInfo
	modlistChecks     []*widget.Check
	outputText        *widget.Entry
	statusLabel       *widget.Label
	wabbajackLabel    *widget.Label
	downloadsLabel    *widget.Label
	backupPathLabel   *widget.Label
	backupFolderCheck *widget.Check
	modlistContainer  *fyne.Container
	actionsContainer  *fyne.Container
	progressBar       *widget.ProgressBar
	scrollContainer   *container.Scroll
	progressSection   *fyne.Container
}

// NewGUIApp creates and initializes the GUI application
func NewGUIApp() *GUIApp {
	a := app.NewWithID("com.yakrel.wabbajack-library-cleaner")
	w := a.NewWindow("Wabbajack Library Cleaner")
	w.Resize(fyne.NewSize(1200, 900))
	w.CenterOnScreen() // Center window on screen

	// Set window icon
	icon, err := fyne.LoadResourceFromPath("winres/icon_main.png")
	if err == nil {
		w.SetIcon(icon)
	}

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
		"Wabbajack Library Cleaner v2.0",
		fyne.TextAlignCenter,
		fyne.TextStyle{Bold: true},
	)
	subtitle := widget.NewLabelWithStyle(
		"Clean orphaned mods and old versions from your Wabbajack downloads",
		fyne.TextAlignCenter,
		fyne.TextStyle{},
	)

	// Step 1: Wabbajack root folder selection
	g.wabbajackLabel = widget.NewLabel("Wabbajack Folder: (Not selected)")
	selectWabbajackBtn := widget.NewButton("📁 Select Wabbajack Folder", func() {
		g.selectWabbajackDir()
	})

	// Modlist checkboxes container (initially hidden)
	g.modlistContainer = container.NewVBox()

	dirSection1 := container.NewVBox(
		widget.NewLabelWithStyle("Step 1: Select Wabbajack Folder", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabel("Select your Wabbajack installation folder (where Wabbajack.exe is located)"),
		widget.NewLabel("Example: D:\\Wabbajack or D:\\Games\\Wabbajack"),
		widget.NewLabel("💡 The tool will automatically scan all version folders for modlists"),
		g.wabbajackLabel,
		selectWabbajackBtn,
		g.modlistContainer,
	)

	// Step 2: Downloads folder selection
	g.downloadsLabel = widget.NewLabel("Downloads Folder: (Not selected)")
	selectDownloadsBtn := widget.NewButton("📁 Select Downloads Folder", func() {
		g.selectDownloadsDir()
	})

	dirSection2 := container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Step 2: Select Downloads Folder", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabel("Select your downloads folder (e.g., F:\\Wabbajack or F:\\Wabbajack\\Fallout 4)"),
		widget.NewLabel("💡 You can select either the parent folder or a specific game folder"),
		g.downloadsLabel,
		selectDownloadsBtn,
	)

	// Step 3: Options
	g.backupPathLabel = widget.NewLabel("Deleted files will be moved to: (Select downloads folder first)")
	g.backupPathLabel.Wrapping = fyne.TextWrapWord

	g.backupFolderCheck = widget.NewCheck("💾 Move to deletion folder (can be restored later)", func(checked bool) {
		g.moveToBackup = checked
	})
	g.backupFolderCheck.SetChecked(true) // Default to deletion folder for safety
	g.moveToBackup = true

	optionsSection := container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Step 3: Deletion Options", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		g.backupPathLabel,
		g.backupFolderCheck,
	)

	// Step 4: Action buttons (PRIMARY: Orphaned Mods, SECONDARY: Old Versions)
	// Primary Actions - Orphaned Mods Cleanup
	scanOrphanedBtn := widget.NewButton("🔍 Scan for Orphaned Mods", func() {
		g.scanOrphanedMods(false)
	})
	scanOrphanedBtn.Importance = widget.HighImportance

	cleanOrphanedBtn := widget.NewButton("🧹 Clean Orphaned Mods", func() {
		g.scanOrphanedMods(true)
	})
	cleanOrphanedBtn.Importance = widget.HighImportance

	// Secondary Actions - Old Versions Cleanup
	scanOldVersionsBtn := widget.NewButton("🔍 Scan for Old Versions", func() {
		g.scanOldVersions(false)
	})

	cleanOldVersionsBtn := widget.NewButton("🧹 Clean Old Versions", func() {
		g.scanOldVersions(true)
	})

	// Other Actions
	statsBtn := widget.NewButton("📊 View Statistics", func() {
		g.viewStats()
	})

	aboutBtn := widget.NewButton("📖 About", func() {
		g.showAbout()
	})

	g.actionsContainer = container.NewVBox(
		widget.NewSeparator(),
		widget.NewLabelWithStyle("Step 4: Cleanup Actions", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabelWithStyle("PRIMARY: Orphaned Mods Cleanup", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabel("Remove mods not used by selected modlists (major space savings)"),
		container.NewGridWithColumns(2,
			scanOrphanedBtn,
			cleanOrphanedBtn,
		),
		widget.NewSeparator(),
		widget.NewLabelWithStyle("SECONDARY: Old Versions Cleanup", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		widget.NewLabel("⚠️ Warning: Some modlists may require old versions! Check carefully before cleaning."),
		container.NewGridWithColumns(2,
			scanOldVersionsBtn,
			cleanOldVersionsBtn,
		),
		widget.NewSeparator(),
		container.NewGridWithColumns(2,
			statsBtn,
			aboutBtn,
		),
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

	// Status bar and progress (placed after actions for visibility)
	g.statusLabel = widget.NewLabel("Ready")
	g.progressBar = widget.NewProgressBar()
	g.progressBar.Hide() // Initially hidden

	g.progressSection = container.NewVBox(
		widget.NewSeparator(),
		g.statusLabel,
		g.progressBar,
	)

	// Footer with branding
	footerLabel := widget.NewLabelWithStyle(
		"Made by Berkay Yetgin | github.com/Yakrel/wabbajack-library-cleaner",
		fyne.TextAlignCenter,
		fyne.TextStyle{Italic: true},
	)

	footerSection := container.NewVBox(
		widget.NewSeparator(),
		footerLabel,
	)

	// Main layout
	content := container.NewVBox(
		title,
		subtitle,
		widget.NewSeparator(),
		dirSection1,
		dirSection2,
		optionsSection,
		g.actionsContainer,
		g.progressSection,
		outputSection,
		footerSection,
	)

	g.scrollContainer = container.NewScroll(content)
	g.window.SetContent(g.scrollContainer)
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

// selectWabbajackDir opens a dialog to select the wabbajack root directory and auto-scans all version folders
func (g *GUIApp) selectWabbajackDir() {
	// Don't search automatically - causes freezing on slow drives
	// User can manually navigate to Wabbajack folder

	dialogBuilder := nativedialog.Directory().Title("Select Wabbajack Folder (where Wabbajack.exe is)")

	path, err := dialogBuilder.Browse()
	if err != nil {
		if err.Error() != "Cancelled" {
			dialog.ShowError(err, g.window)
			logError("Error selecting wabbajack directory: %v", err)
		}
		return
	}

	// Validate path
	if !isValidPath(path) {
		dialog.ShowError(fmt.Errorf("invalid directory path"), g.window)
		logError("Invalid wabbajack directory path: %s", path)
		return
	}

	g.wabbajackDir = path
	g.wabbajackLabel.SetText("Wabbajack Folder: " + g.wabbajackDir)
	g.appendOutput("\n=== Scanning Wabbajack Installation ===")
	g.appendOutput("Selected Wabbajack folder: " + g.wabbajackDir)
	logInfo("User selected wabbajack directory: %s", g.wabbajackDir)

	// Scan all version folders for modlists
	g.scanAllVersionsForModlists()
}

// scanAllVersionsForModlists scans all version folders in Wabbajack root and merges modlists (newest version priority)
func (g *GUIApp) scanAllVersionsForModlists() {
	g.setStatus("Scanning version folders...")

	// Find all version folders
	entries, err := os.ReadDir(g.wabbajackDir)
	if err != nil {
		g.appendOutput(fmt.Sprintf("❌ Error reading Wabbajack folder: %v", err))
		dialog.ShowError(err, g.window)
		return
	}

	// Collect all modlist paths from all version folders
	type modlistEntry struct {
		path    string
		version string
	}
	modlistMap := make(map[string]modlistEntry) // key: modlist filename (without @@), value: latest version path

	var versionFolders []string
	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}

		// Check if this folder has downloaded_mod_lists
		modlistsPath := filepath.Join(g.wabbajackDir, entry.Name(), "downloaded_mod_lists")
		if _, err := os.Stat(modlistsPath); err == nil {
			versionFolders = append(versionFolders, entry.Name())

			// Scan for .wabbajack files in this version
			wbFiles, err := findWabbajackFiles(modlistsPath)
			if err != nil {
				continue
			}

			g.appendOutput(fmt.Sprintf("Version %s: Found %d modlist(s)", entry.Name(), len(wbFiles)))

			for _, wbFile := range wbFiles {
				// Extract modlist name (before @@)
				baseName := filepath.Base(wbFile)
				parts := strings.Split(baseName, "@@")
				modlistKey := baseName
				if len(parts) > 0 {
					modlistKey = parts[0]
				}

				// Keep the latest version (last folder alphabetically = newest version)
				existing, exists := modlistMap[modlistKey]
				if !exists || entry.Name() > existing.version {
					modlistMap[modlistKey] = modlistEntry{
						path:    wbFile,
						version: entry.Name(),
					}
				}
			}
		}
	}

	if len(versionFolders) == 0 {
		g.appendOutput("❌ No version folders with downloaded_mod_lists found")
		g.appendOutput("⚠️ Make sure you selected the Wabbajack root folder (where Wabbajack.exe is)")
		dialog.ShowError(fmt.Errorf("No version folders found in: %s", g.wabbajackDir), g.window)
		return
	}

	if len(modlistMap) == 0 {
		g.appendOutput("❌ No .wabbajack files found in any version folder")
		dialog.ShowError(fmt.Errorf("No .wabbajack files found"), g.window)
		return
	}

	g.appendOutput(fmt.Sprintf("\n📊 Total unique modlists found: %d", len(modlistMap)))

	// Parse the selected modlists (newest version of each)
	g.modlistInfos = nil
	for modlistKey, entry := range modlistMap {
		info, err := parseWabbajackFile(entry.path)
		if err != nil {
			g.appendOutput(fmt.Sprintf("⚠️ Failed to parse %s: %v", modlistKey, err))
			logError("Failed to parse %s: %v", entry.path, err)
			continue
		}
		g.appendOutput(fmt.Sprintf("  ✓ %s (%d mods) [v%s]", info.Name, info.ModCount, entry.version))
		g.modlistInfos = append(g.modlistInfos, info)
	}

	if len(g.modlistInfos) == 0 {
		g.appendOutput("❌ Failed to parse any modlist files")
		dialog.ShowError(fmt.Errorf("Failed to parse any modlist files"), g.window)
		return
	}

	// Create checkboxes for each modlist
	g.modlistContainer.Objects = nil
	g.modlistChecks = nil

	g.modlistContainer.Add(widget.NewSeparator())
	g.modlistContainer.Add(widget.NewLabelWithStyle("Select Active Modlists:", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}))
	g.modlistContainer.Add(widget.NewLabel("Check the modlists you are currently using:"))

	for _, info := range g.modlistInfos {
		check := widget.NewCheck(fmt.Sprintf("%s (%d mods)", info.Name, info.ModCount), nil)
		check.SetChecked(true) // Default to all selected
		g.modlistChecks = append(g.modlistChecks, check)
		g.modlistContainer.Add(check)
	}

	// Add select all/none buttons
	buttonRow := container.NewGridWithColumns(2,
		widget.NewButton("Select All", func() {
			for _, check := range g.modlistChecks {
				check.SetChecked(true)
			}
		}),
		widget.NewButton("Deselect All", func() {
			for _, check := range g.modlistChecks {
				check.SetChecked(false)
			}
		}),
	)
	g.modlistContainer.Add(buttonRow)

	g.modlistContainer.Refresh()
	g.setStatus(fmt.Sprintf("Found %d modlists", len(g.modlistInfos)))
}

// selectDownloadsDir opens a dialog to select the downloads directory
func (g *GUIApp) selectDownloadsDir() {
	path, err := nativedialog.Directory().Title("Select Downloads Folder").Browse()
	if err != nil {
		if err.Error() != "Cancelled" {
			dialog.ShowError(err, g.window)
			logError("Error selecting downloads directory: %v", err)
		}
		return
	}

	// Validate path
	if !isValidPath(path) {
		dialog.ShowError(fmt.Errorf("invalid directory path"), g.window)
		logError("Invalid downloads directory path: %s", path)
		return
	}

	g.downloadsDir = path
	g.downloadsLabel.SetText("Downloads Directory: " + g.downloadsDir)
	g.appendOutput("Selected downloads directory: " + g.downloadsDir)
	logInfo("User selected downloads directory: %s", g.downloadsDir)

	// Update path label with timestamp example
	timestamp := time.Now().Format("2006-01-02_15-04-05")
	backupPath := filepath.Join(g.downloadsDir, "WLC_Deleted", timestamp)
	g.backupPathLabel.SetText("Deleted files will be moved to: " + backupPath)
}

// validateDirectories checks if required directories are selected
func (g *GUIApp) validateDirectories() bool {
	if g.wabbajackDir == "" {
		dialog.ShowError(fmt.Errorf("Please select the Modlist Folder first (Step 1)"), g.window)
		return false
	}
	if g.downloadsDir == "" {
		dialog.ShowError(fmt.Errorf("Please select the Downloads Folder first (Step 2)"), g.window)
		return false
	}
	if len(g.modlistInfos) == 0 {
		dialog.ShowError(fmt.Errorf("No modlists found. Please select a folder containing .wabbajack files"), g.window)
		return false
	}
	return true
}

// getSelectedModlists returns the modlists that are checked
func (g *GUIApp) getSelectedModlists() []*ModlistInfo {
	var selected []*ModlistInfo
	for i, check := range g.modlistChecks {
		if check.Checked && i < len(g.modlistInfos) {
			selected = append(selected, g.modlistInfos[i])
		}
	}
	return selected
}

// scanOldVersions handles old version scanning/cleaning
func (g *GUIApp) scanOldVersions(deleteMode bool) {
	if !g.validateDirectories() {
		return
	}

	g.setStatus("Scanning for old versions...")
	g.appendOutput("\n=== Scanning for Old Versions ===")

	// Scroll to progress section
	g.scrollContainer.ScrollToBottom()

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
		"Select Folder to Scan",
		"Scan",
		"Cancel",
		container.NewVBox(
			widget.NewLabel("Select which mod folder to scan for old versions:"),
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

// scanOrphanedMods handles orphaned mods scanning/cleaning using pre-selected modlists
func (g *GUIApp) scanOrphanedMods(deleteMode bool) {
	if !g.validateDirectories() {
		return
	}

	// Get selected modlists
	activeModlists := g.getSelectedModlists()
	if len(activeModlists) == 0 {
		dialog.ShowError(fmt.Errorf("No modlists selected. Please check at least one modlist from the list above."), g.window)
		return
	}

	// Show progress and disable buttons during scan
	g.progressBar.Show()
	g.setStatus("Scanning for orphaned mods...")
	g.appendOutput("\n=== Scanning for Orphaned Mods ===")
	g.appendOutput(fmt.Sprintf("Using %d selected modlist(s):", len(activeModlists)))
	for _, ml := range activeModlists {
		g.appendOutput(fmt.Sprintf("  ✓ %s", ml.Name))
	}

	// Run scan in goroutine to prevent UI freeze
	go func() {
		defer func() {
			g.progressBar.Hide()
		}()

		// Perform the scan
		g.performOrphanedScan(activeModlists, deleteMode)
	}()
}

// performOrphanedScan performs the actual orphaned mods scan
func (g *GUIApp) performOrphanedScan(activeModlists []*ModlistInfo, deleteMode bool) {
	// Show progress bar for scanning phase
	g.progressBar.Show()
	g.progressBar.Min = 0
	g.progressBar.Max = 100
	g.progressBar.SetValue(0)

	// Scroll to progress section to make it visible
	g.scrollContainer.ScrollToBottom()

	// Get game folders
	g.setStatus("Getting game folders...")
	g.progressBar.SetValue(10)
	gameFolders, err := getGameFolders(g.downloadsDir)
	if err != nil {
		g.progressBar.Hide()
		dialog.ShowError(err, g.window)
		return
	}

	// Collect all mod files
	g.setStatus("Collecting mod files...")
	g.progressBar.SetValue(30)
	g.appendOutput("\nCollecting mod files...")
	allModFiles, err := getAllModFiles(gameFolders)
	if err != nil {
		g.progressBar.Hide()
		dialog.ShowError(err, g.window)
		return
	}
	g.appendOutput(fmt.Sprintf("Found %d mod files", len(allModFiles)))

	// Detect orphaned mods
	g.setStatus("Analyzing mod usage...")
	g.progressBar.SetValue(60)
	g.appendOutput("Analyzing...")
	usedMods, orphanedMods := detectOrphanedMods(allModFiles, activeModlists)

	g.progressBar.SetValue(90)
	g.setStatus("Calculating results...")

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

	g.progressBar.SetValue(100)
	g.progressBar.Hide()

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
	if g.moveToBackup {
		confirmText = "Files will be moved to a timestamped deletion folder.\n\nYou can restore them later if needed.\n\nContinue?"
	} else {
		confirmText = "⚠️ WARNING ⚠️\n\nThis will PERMANENTLY DELETE files!\n\nAre you absolutely sure?"
	}

	dialog.ShowConfirm(
		"Confirm Deletion",
		confirmText,
		func(confirmed bool) {
			if confirmed {
				g.progressBar.Show()
				g.scrollContainer.ScrollToBottom()

				// Run deletion in goroutine to prevent UI freeze
				go func() {
					onConfirm()
				}()
			} else {
				g.setStatus("Cancelled")
			}
		},
		g.window,
	)
}

// deleteModFilesWithRecycleBin is a common function to delete mod files with backup folder option
func (g *GUIApp) deleteModFilesWithRecycleBin(files []ModFile) (int, int64) {
	deletedCount := 0
	spaceFreed := int64(0)
	totalFiles := len(files)

	// Create backup folder with timestamp if moveToBackup is enabled
	var backupDir string
	if g.moveToBackup {
		timestamp := time.Now().Format("2006-01-02_15-04-05")
		backupDir = filepath.Join(g.downloadsDir, "WLC_Deleted", timestamp)
		err := os.MkdirAll(backupDir, 0755)
		if err != nil {
			g.appendOutput(fmt.Sprintf("❌ Failed to create backup folder: %v", err))
			logError("Failed to create backup folder: %v", err)
			g.progressBar.Hide()
			return 0, 0
		}
		g.appendOutput(fmt.Sprintf("📁 Created backup folder: %s", backupDir))
	}

	// Show progress bar if we have files to delete
	if totalFiles > 0 {
		g.progressBar.Min = 0
		g.progressBar.Max = float64(totalFiles)
		g.progressBar.SetValue(0)
		g.progressBar.Show()
	}

	for i, file := range files {
		// Update progress bar
		g.progressBar.SetValue(float64(i + 1))

		// Update status with detailed info
		percentage := int(float64(i+1) / float64(totalFiles) * 100)
		g.setStatus(fmt.Sprintf("Processing: %d/%d files (%d%%)", i+1, totalFiles, percentage))

		if isFileLocked(file.FullPath) {
			g.appendOutput(fmt.Sprintf("⚠ Skipped (locked): %s", file.FileName))
			continue
		}

		var err error
		if g.moveToBackup {
			// Move to backup folder
			backupPath := filepath.Join(backupDir, file.FileName)
			err = os.Rename(file.FullPath, backupPath)
		} else {
			err = deleteFile(file.FullPath)
		}

		if err != nil {
			g.appendOutput(fmt.Sprintf("✗ Failed to process: %s - %v", file.FileName, err))
			logError("Failed to process %s: %v", file.FullPath, err)
			continue
		}

		deletedCount++
		spaceFreed += file.Size
		if g.moveToBackup {
			g.appendOutput(fmt.Sprintf("✓ Moved: %s (%s)", file.FileName, formatSize(file.Size)))
			logInfo("Moved to backup: %s (%s)", file.FileName, formatSize(file.Size))
		} else {
			g.appendOutput(fmt.Sprintf("✓ Deleted: %s (%s)", file.FileName, formatSize(file.Size)))
			logInfo("Deleted: %s (%s)", file.FileName, formatSize(file.Size))
		}

		// Handle .meta file
		metaPath := file.FullPath + ".meta"
		if fileExists(metaPath) {
			var metaErr error
			if g.moveToBackup {
				backupMetaPath := filepath.Join(backupDir, file.FileName+".meta")
				metaErr = os.Rename(metaPath, backupMetaPath)
			} else {
				metaErr = deleteFile(metaPath)
			}
			if metaErr != nil {
				g.appendOutput(fmt.Sprintf("⚠ Failed to process .meta file: %s - %v", filepath.Base(metaPath), metaErr))
				logWarning("Failed to process .meta file: %s - %v", metaPath, metaErr)
			}
		}
	}

	// Hide progress bar when done
	g.progressBar.Hide()

	return deletedCount, spaceFreed
}

// deleteOldVersionsWithRecycleBin deletes old versions with recycle bin option
func (g *GUIApp) deleteOldVersionsWithRecycleBin(duplicates map[string]*ModGroup) (int, int64) {
	var filesToDelete []ModFile

	for _, group := range duplicates {
		// Collect old versions (everything before the newest)
		for i := 0; i < group.NewestIdx; i++ {
			filesToDelete = append(filesToDelete, group.Files[i])
		}
	}

	return g.deleteModFilesWithRecycleBin(filesToDelete)
}

// deleteOrphanedModsWithRecycleBin deletes orphaned mods with recycle bin option
func (g *GUIApp) deleteOrphanedModsWithRecycleBin(orphanedMods []OrphanedMod) (int, int64) {
	var filesToDelete []ModFile

	for _, om := range orphanedMods {
		filesToDelete = append(filesToDelete, om.File)
	}

	return g.deleteModFilesWithRecycleBin(filesToDelete)
}

// showAbout displays the About dialog
func (g *GUIApp) showAbout() {
	aboutText := `Wabbajack Library Cleaner v2.0

© 2025 Berkay Yetgin

GitHub: github.com/Yakrel/wabbajack-library-cleaner

Licensed under GNU General Public License v3.0
This is free and open source software.`

	dialog.ShowInformation("About", aboutText, g.window)
}

// Run starts the GUI application
func (g *GUIApp) Run() {
	g.window.ShowAndRun()
}
