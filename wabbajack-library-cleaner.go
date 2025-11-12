
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
	"archive/zip"
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

// ANSI color codes
const (
	ColorReset  = "\033[0m"
	ColorRed    = "\033[31m"
	ColorGreen  = "\033[32m"
	ColorYellow = "\033[33m"
	ColorBlue   = "\033[34m"
	ColorPurple = "\033[35m"
	ColorCyan   = "\033[36m"
	ColorWhite  = "\033[37m"
	ColorBold   = "\033[1m"
)

// ModFile represents a parsed mod file
type ModFile struct {
	FileName  string
	FullPath  string
	ModName   string
	ModID     string
	Version   string
	Timestamp string
	Size      int64
	IsPatch   bool // True if file appears to be a patch/hotfix/update
}

// ModGroup represents a group of mod versions
type ModGroup struct {
	ModKey      string
	Files       []ModFile
	NewestIdx   int
	SpaceToFree int64
}

// Config holds program configuration
type Config struct {
	LogFile          *os.File
	MaxVersionsToKeep int
	MinFileSizeMB    float64
	SafeMode         bool
}

// ModlistArchive represents a parsed modlist archive entry
type ModlistArchive struct {
	Hash  string          `json:"Hash"`
	Name  string          `json:"Name"`
	Size  int64           `json:"Size"`
	State ModlistModState `json:"State"`
}

// ModlistModState represents the state field of a modlist archive
type ModlistModState struct {
	Type     string `json:"$type"`
	ModID    int64  `json:"ModID"`
	FileID   int64  `json:"FileID"`
	GameName string `json:"GameName"`
	Name     string `json:"Name"`
	Version  string `json:"Version"`
}

// Modlist represents a parsed .wabbajack file
type Modlist struct {
	Name     string             `json:"Name"`
	Version  string             `json:"Version"`
	Author   string             `json:"Author"`
	Archives []ModlistArchive   `json:"Archives"`
}

// ModlistInfo contains information about a modlist
type ModlistInfo struct {
	FilePath    string
	Name        string
	ModCount    int
	UsedModKeys map[string]bool // ModID-based keys for quick lookup
}

// OrphanedMod represents a mod file that's not used by any active modlist
type OrphanedMod struct {
	File ModFile
}

var config Config

var archiveExtensions = []string{".7z", ".zip", ".rar", ".tar", ".gz", ".exe"}

func main() {
	// Enable ANSI colors on Windows
	enableWindowsColors()

	// Catch panics only
	defer func() {
		if r := recover(); r != nil {
			fmt.Printf("\n%s[PANIC]%s Program crashed: %v\n", ColorRed, ColorReset, r)
			fmt.Printf("\n%sPress Enter to exit...%s", ColorYellow, ColorReset)
			bufio.NewReader(os.Stdin).ReadBytes('\n')
		}
	}()

	// Initialize logging
	initLogging()
	defer config.LogFile.Close()

	baseDir, err := os.Getwd()
	if err != nil {
		fmt.Printf("%s[ERROR]%s Failed to get working directory: %v\n", ColorRed, ColorReset, err)
		logError("Failed to get working directory: %v", err)
		waitForExit()
		return
	}

	logInfo("Program started in directory: %s", baseDir)

	fmt.Printf("%s%sWorking directory: %s%s\n", ColorBold, ColorCyan, baseDir, ColorReset)

	gameFolders, err := getGameFolders(baseDir)
	if err != nil {
		fmt.Printf("%s[ERROR]%s %v\n", ColorRed, ColorReset, err)
		waitForExit()
		return
	}

	// Check if we're inside a mod folder (no subfolders, but many archives)
	if len(gameFolders) == 0 {
		archiveCount := countArchivesInDir(baseDir)
		if archiveCount > 10 {
			fmt.Printf("\n%s[WARNING]%s It looks like you're inside a mod archives folder!\n", ColorYellow, ColorReset)
			fmt.Printf("Found %d archive files in current directory.\n\n", archiveCount)
			fmt.Printf("This tool is designed to scan multiple game mod folders at once.\n")
			fmt.Printf("Recommended: Place it in the parent directory.\n")
			fmt.Printf("  Example: %sF:\\Wabbajack\\%s (not inside F:\\Wabbajack\\Skyrim\\)\n\n", ColorCyan, ColorReset)

			fmt.Printf("Would you like to clean THIS folder anyway? (yes/no): ")
			scanner := bufio.NewScanner(os.Stdin)
			if scanner.Scan() && confirmInput(scanner.Text()) {
				// Add current directory as the only folder with a clear name
				gameFolders = append(gameFolders, baseDir)
				fmt.Printf("\n%s[OK]%s Processing current directory...\n", ColorGreen, ColorReset)
				logInfo("User chose to clean current directory: %s", baseDir)
			} else {
				fmt.Printf("\n%s[INFO]%s Please move the tool to the correct directory and run again.\n", ColorCyan, ColorReset)
				logInfo("User declined to clean current directory")
				waitForExit()
				return
			}
		} else {
			fmt.Printf("\n%s[ERROR]%s No game mod folders found in this directory!\n", ColorRed, ColorReset)
			fmt.Printf("\n%s[INFO]%s This tool should be run from your Wabbajack downloads directory.\n", ColorCyan, ColorReset)
			fmt.Printf("\n%sExpected directory structure:%s\n", ColorBold, ColorReset)
			fmt.Printf("  %sF:\\Wabbajack\\%s                    %s<-- Run the tool here%s\n", ColorCyan, ColorReset, ColorGreen, ColorReset)
			fmt.Printf("  ├─ Skyrim\\                     %s<-- Mod archives for Skyrim%s\n", ColorYellow, ColorReset)
			fmt.Printf("  │  ├─ ModName_v1.0.7z\n")
			fmt.Printf("  │  ├─ ModName_v1.1.7z\n")
			fmt.Printf("  │  └─ ...\n")
			fmt.Printf("  ├─ Fallout4\\                   %s<-- Mod archives for Fallout 4%s\n", ColorYellow, ColorReset)
			fmt.Printf("  └─ [other game folders]\\\n\n")
			fmt.Printf("%s[!] Note:%s These are NOT your game installation folders!\n", ColorYellow, ColorReset)
			fmt.Printf("    They are folders containing downloaded mod archives (.7z, .zip, etc.)\n\n")
			fmt.Printf("Current directory: %s%s%s\n", ColorYellow, baseDir, ColorReset)
			fmt.Printf("\nPlease navigate to the correct directory and try again.\n")
			logInfo("No game folders found in: %s", baseDir)
			waitForExit()
			return
		}
	}

	if len(gameFolders) > 0 {
		fmt.Printf("%s[OK]%s Found %d game folder(s):\n", ColorGreen, ColorReset, len(gameFolders))
		for i, folder := range gameFolders {
			fmt.Printf("  %d. %s\n", i+1, filepath.Base(folder))
		}
	}

	// Main menu loop
	scanner := bufio.NewScanner(os.Stdin)
	for {
		printMenu()
		fmt.Print("\nSelect option (1-6): ")

		if !scanner.Scan() {
			break
		}

		choice := strings.TrimSpace(scanner.Text())

		switch choice {
		case "1":
			// Scan folder - Dry run
			scanSpecificFolder(gameFolders, scanner, false)
		case "2":
			// Delete from specific folder
			scanSpecificFolder(gameFolders, scanner, true)
		case "3":
			// Scan for orphaned mods - Dry run
			scanOrphanedMods(baseDir, gameFolders, scanner, false)
		case "4":
			// Clean orphaned mods
			scanOrphanedMods(baseDir, gameFolders, scanner, true)
		case "5":
			// View statistics
			viewStatistics(gameFolders)
		case "6":
			fmt.Printf("\n%sGoodbye!%s\n", ColorCyan, ColorReset)
			logInfo("Program exited by user")
			return
		default:
			fmt.Printf("%s[ERROR]%s Invalid option!\n", ColorRed, ColorReset)
		}

		fmt.Printf("\n%sPress Enter to continue...%s", ColorBlue, ColorReset)
		scanner.Scan()
	}
}

func printMenu() {
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\n%35s%s%s\n", "", "WABBAJACK LIBRARY CLEANER", "")
	fmt.Printf("%s%s\n", strings.Repeat("=", 100), ColorReset)
	fmt.Printf("%s1.%s Scan folder (Dry-run) - Preview old versions\n", ColorBold, ColorReset)
	fmt.Printf("%s2.%s Clean folder - Delete old versions\n", ColorBold, ColorReset)
	fmt.Printf("\n%s3.%s Scan for orphaned mods (Dry-run) - Preview unused mods\n", ColorBold, ColorReset)
	fmt.Printf("%s4.%s Clean orphaned mods - Delete unused mods\n", ColorBold, ColorReset)
	fmt.Printf("\n%s5.%s View statistics\n", ColorBold, ColorReset)
	fmt.Printf("%s6.%s Exit\n", ColorBold, ColorReset)
	fmt.Printf("\n%s[!] Always run Dry-run first!%s\n", ColorYellow, ColorReset)
}

func waitForExit() {
	fmt.Printf("\n%sPress Enter to exit...%s", ColorYellow, ColorReset)
	bufio.NewReader(os.Stdin).ReadBytes('\n')
}

func getGameFolders(baseDir string) ([]string, error) {
	var folders []string

	entries, err := os.ReadDir(baseDir)
	if err != nil {
		return nil, err
	}

	for _, entry := range entries {
		if entry.IsDir() && !strings.HasPrefix(entry.Name(), ".") && !strings.HasPrefix(entry.Name(), "__") {
			folders = append(folders, filepath.Join(baseDir, entry.Name()))
		}
	}

	sort.Strings(folders)
	return folders, nil
}

func countArchivesInDir(dir string) int {
	count := 0
	entries, err := os.ReadDir(dir)
	if err != nil {
		return 0
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		ext := strings.ToLower(filepath.Ext(entry.Name()))
		for _, validExt := range archiveExtensions {
			if ext == validExt {
				count++
				break
			}
		}
	}

	return count
}

func parseModFilename(filename string) *ModFile {
	// Check if it has a valid archive extension
	ext := strings.ToLower(filepath.Ext(filename))
	hasValidExt := false
	for _, validExt := range archiveExtensions {
		if ext == validExt {
			hasValidExt = true
			break
		}
	}
	if !hasValidExt {
		return nil
	}

	// Remove extension
	nameWithoutExt := strings.TrimSuffix(filename, ext)

	// Split by dash
	parts := strings.Split(nameWithoutExt, "-")
	if len(parts) < 3 {
		return nil
	}

	// Last part should be timestamp (10+ digit number)
	timestamp := parts[len(parts)-1]
	if !isNumeric(timestamp) || len(timestamp) < 10 {
		return nil
	}

	// Find ModID (3-6 digit number in parts[1:len-1])
	modID := ""
	modIDIndex := -1

	for i := 1; i < len(parts)-1; i++ {
		if isNumeric(parts[i]) && len(parts[i]) >= 3 && len(parts[i]) <= 6 {
			modID = parts[i]
			modIDIndex = i
			break
		}
	}

	if modID == "" {
		return nil
	}

	// ModName = parts[0:modIDIndex]
	modName := strings.Join(parts[0:modIDIndex], "-")

	// Version = parts[modIDIndex+1:len-1]
	version := strings.Join(parts[modIDIndex+1:len(parts)-1], "-")

	return &ModFile{
		FileName:  filename,
		ModName:   modName,
		ModID:     modID,
		Version:   version,
		Timestamp: timestamp,
		IsPatch:   isPatchOrHotfix(filename),
	}
}

func isNumeric(s string) bool {
	_, err := strconv.Atoi(s)
	return err == nil
}

// normalizeModName removes version patterns from mod names to group versions together
// e.g., "Interface 1.3.6" and "Interface 1.4.0" both become "Interface"
func normalizeModName(modName string) string {
	// Remove trailing version patterns like " 1.3.6", " v1.2", " V2.0", etc.
	// Common patterns: " 1.2.3", " v1.2", " V1.2.3", " 0.18"

	// Split by space and remove trailing parts that look like versions
	parts := strings.Split(modName, " ")
	var cleanParts []string

	for _, part := range parts {
		// Check if this part looks like a version number
		if isVersionPattern(part) {
			// Stop here - everything after looks like version info
			break
		}
		cleanParts = append(cleanParts, part)
	}

	if len(cleanParts) == 0 {
		return modName // Return original if we can't clean it
	}

	return strings.Join(cleanParts, " ")
}

// isVersionPattern checks if a string looks like a version number
func isVersionPattern(s string) bool {
	s = strings.ToLower(s)

	// Remove leading 'v' or 'V'
	if strings.HasPrefix(s, "v") {
		s = s[1:]
	}

	// Check if it contains only digits and dots/dashes
	// e.g., "1.3.6", "0.18", "2-0"
	hasDigit := false
	for _, c := range s {
		if c >= '0' && c <= '9' {
			hasDigit = true
		} else if c != '.' && c != '-' && c != '_' {
			return false // Contains non-version characters
		}
	}

	return hasDigit
}

// isPatchOrHotfix detects if a filename indicates a patch/hotfix/update file
// These keywords suggest the file is a small update, not a full version
func isPatchOrHotfix(filename string) bool {
	lower := strings.ToLower(filename)
	
	patchKeywords := []string{
		"patch", "hotfix", "update", "fix", 
		"- patch", "-patch", " patch",
		"- hotfix", "-hotfix", " hotfix",
		"- update", "-update", " update",
		"- fix", "-fix", " fix",
	}
	
	for _, keyword := range patchKeywords {
		if strings.Contains(lower, keyword) {
			return true
		}
	}
	
	return false
}

// isFullOrMainFile detects if a filename indicates a full/main file
func isFullOrMainFile(filename string) bool {
	lower := strings.ToLower(filename)
	
	fullKeywords := []string{
		"main", "full", "complete", "- main", "-main", " main",
	}
	
	for _, keyword := range fullKeywords {
		if strings.Contains(lower, keyword) {
			return true
		}
	}
	
	return false
}

// extractPartIndicator detects part numbers in filenames to keep multi-part mods separate
// Examples: "-1-", "-2-", "Part 1", "Part 2", "(Part 1)", "Pt1", etc.
func extractPartIndicator(filename string) string {
	lower := strings.ToLower(filename)

	// Pattern 1: "-1-", "-2-", "-3-", etc. (most common)
	// Must NOT be preceded by a letter to avoid matching "Part 1-118893"
	// Should be followed by a letter (like "Meshes") or end of string
	for i := 1; i <= 20; i++ {
		pattern := fmt.Sprintf("-%d-", i)
		idx := strings.Index(lower, pattern)
		if idx != -1 {
			// Check what comes BEFORE the pattern
			if idx > 0 {
				prevChar := lower[idx-1]
				// If preceded by a letter or digit (like "Part 1-"), skip
				if (prevChar >= 'a' && prevChar <= 'z') || (prevChar >= '0' && prevChar <= '9') {
					continue // Not a valid part indicator
				}
			}

			// Check what comes after the pattern
			afterPattern := idx + len(pattern)
			if afterPattern >= len(lower) {
				// End of string - valid
				return pattern
			}
			// Must be followed by a non-digit (letter like "meshes" or space)
			nextChar := lower[afterPattern]
			if nextChar < '0' || nextChar > '9' {
				return pattern
			}
			// If followed by many digits (ModID/timestamp), skip
		}
	}

	// Pattern 2: "part 1", "part 2", "part1", "part2"
	// Search from the end to prioritize rightmost occurrence
	for i := 20; i >= 1; i-- {
		patterns := []string{
			fmt.Sprintf("part %d", i),
			fmt.Sprintf("part%d", i),
			fmt.Sprintf("(part %d)", i),
			fmt.Sprintf("pt%d", i),
			fmt.Sprintf("pt %d", i),
		}
		for _, pattern := range patterns {
			if strings.Contains(lower, pattern) {
				return fmt.Sprintf(":part%d", i)
			}
		}
	}

	// No part indicator found
	return ""
}

// hasSuspiciousVersionPattern detects if a group contains files with same version
// but likely different content (e.g., different variants or optional files)
func hasSuspiciousVersionPattern(group *ModGroup) bool {
	if len(group.Files) < 2 {
		return false
	}

	// Check for same version numbers but very different file sizes or close timestamps
	for i := 0; i < len(group.Files)-1; i++ {
		for j := i + 1; j < len(group.Files); j++ {
			file1 := group.Files[i]
			file2 := group.Files[j]

			// If versions are identical
			if file1.Version == file2.Version {
				// Check 1: File size difference > 10x (likely different content)
				sizeRatio := float64(file1.Size) / float64(file2.Size)
				if sizeRatio > 10.0 || sizeRatio < 0.1 {
					logWarning("Group %s: Same version '%s' but size diff >10x (%s vs %s)",
						group.ModKey, file1.Version,
						formatSize(file1.Size), formatSize(file2.Size))
					return true
				}

				// Check 2: Very close timestamps (< 1 hour apart) with same version
				// This suggests uploaded at same time as different variants
				ts1, _ := strconv.ParseInt(file1.Timestamp, 10, 64)
				ts2, _ := strconv.ParseInt(file2.Timestamp, 10, 64)
				timeDiff := ts2 - ts1
				if timeDiff < 0 {
					timeDiff = -timeDiff
				}

				oneHour := int64(3600)
				if timeDiff < oneHour {
					logWarning("Group %s: Same version '%s' uploaded within 1 hour (likely variants)",
						group.ModKey, file1.Version)
					return true
				}
			}
			
			// Check 3: Different descriptive keywords in filenames (even with different versions)
			// These indicate different content types: texture quality, mod parts, variants, etc.
			if hasConflictingDescriptors(file1.FileName, file2.FileName) {
				logWarning("Group %s: Files have conflicting descriptors - '%s' vs '%s' (likely different content)",
					group.ModKey, file1.FileName, file2.FileName)
				return true
			}
		}
	}

	return false
}

// hasConflictingDescriptors checks if two filenames have different content descriptors
// Returns true if files appear to be different variants/parts of the same mod
func hasConflictingDescriptors(filename1, filename2 string) bool {
	lower1 := strings.ToLower(filename1)
	lower2 := strings.ToLower(filename2)
	
	// All possible content descriptors - ANY difference suggests different content
	allDescriptors := []string{
		// Texture quality
		" 1k", " 2k", " 4k", " 8k", "-1k", "-2k", "-4k", "-8k",
		// Body types
		"cbbe", "uunp", "bhunp", "vanilla body", "bodyslide",
		// Mod components (usually separate files)
		" armor", " weapon", " clothes", " clothing", " hair", " gloves", " boots", " helmet", 
		" meshes", " textures", "-armor", "-weapon", "-clothes", "-hair", "-gloves",
		// File types/packaging
		" esp ", " esm ", " esl ", "esp-fe", "esp only", "esm only", "loose files", " bsa",
		// Compatibility/variants
		" compat", "compatibility", " aslal", "no worldspace", "worldspace edit", " performance",
		// Edition types
		" lite", " light", " full", " extended", " complete", " basic", " standard", " deluxe",
		// Clean variants
		" clean", " dirty", " gross",
		// Optional content
		" optional", " addon", " add-on", " expansion",
	}
	
	// Check if files have DIFFERENT descriptors
	descriptors1 := []string{}
	descriptors2 := []string{}
	
	for _, desc := range allDescriptors {
		if strings.Contains(lower1, desc) {
			descriptors1 = append(descriptors1, desc)
		}
		if strings.Contains(lower2, desc) {
			descriptors2 = append(descriptors2, desc)
		}
	}
	
	// If one file has descriptors but the other doesn't, they're likely different
	// (e.g., "FN 502 - No worldspace edits" vs "FN 502" are different variants)
	if (len(descriptors1) > 0 && len(descriptors2) == 0) || (len(descriptors1) == 0 && len(descriptors2) > 0) {
		return true
	}
	
	// If both have descriptors but they don't share any, they're different content
	if len(descriptors1) > 0 && len(descriptors2) > 0 {
		hasCommon := false
		for _, d1 := range descriptors1 {
			for _, d2 := range descriptors2 {
				if d1 == d2 {
					hasCommon = true
					break
				}
			}
			if hasCommon {
				break
			}
		}
		
		if !hasCommon {
			return true
		}
	}
	
	return false
}

func scanFolder(folderPath string) (map[string]*ModGroup, error) {
	fmt.Printf("\n%s[SCANNING]%s %s\n", ColorCyan, ColorReset, filepath.Base(folderPath))
	logInfo("Scanning folder: %s", folderPath)

	modGroups := make(map[string]*ModGroup)
	skipped := 0

	entries, err := os.ReadDir(folderPath)
	if err != nil {
		logError("Failed to read directory %s: %v", folderPath, err)
		return nil, err
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}

		filename := entry.Name()

		// Skip non-Wabbajack files (temp files, partial downloads, etc.)
		if !isWabbajackFile(filename) {
			skipped++
			continue
		}

		modFile := parseModFilename(filename)

		if modFile == nil {
			skipped++
			continue
		}

		fullPath := filepath.Join(folderPath, filename)
		info, err := os.Stat(fullPath)
		if err != nil {
			continue
		}

		modFile.FullPath = fullPath
		modFile.Size = info.Size()

		// ModKey = ModID + normalized ModName + part indicator (if exists)
		// Normalization removes version numbers from ModName
		// Part indicators keep multi-part mods separate (e.g., "-1-", "-2-", "Part 1")
		// This ensures:
		// - "Interface 1.3.6-27216" and "Interface 1.4.0-27216" ARE grouped (same base name)
		// - "Ysmir Hair-112480" and "Ysmir Armor-112480" are NOT grouped (different base names)
		// - "Rock Remesh -1- Meshes" and "Rock Remesh -2- Textures" are NOT grouped (different parts)
		normalizedName := normalizeModName(modFile.ModName)
		// Check both full filename AND modname for part indicators
		partIndicator := extractPartIndicator(modFile.FileName)
		if partIndicator == "" {
			partIndicator = extractPartIndicator(modFile.ModName)
		}
		modKey := modFile.ModID + ":" + normalizedName + partIndicator

		if group, exists := modGroups[modKey]; exists {
			group.Files = append(group.Files, *modFile)
		} else {
			modGroups[modKey] = &ModGroup{
				ModKey: modKey,
				Files:  []ModFile{*modFile},
			}
		}
	}

	if skipped > 0 {
		fmt.Printf("%s[WARN]%s Skipped %d files (couldn't parse naming pattern)\n", ColorYellow, ColorReset, skipped)
		logInfo("Skipped %d files in %s", skipped, folderPath)
	}

	// Find duplicates and calculate space
	duplicates := make(map[string]*ModGroup)

	for key, group := range modGroups {
		if len(group.Files) <= 1 {
			continue
		}

		// Safety check: Ensure files actually have different timestamps
		// If all files have same timestamp, they're the same file (not duplicates)
		uniqueTimestamps := make(map[string]bool)
		for _, f := range group.Files {
			uniqueTimestamps[f.Timestamp] = true
		}

		if len(uniqueTimestamps) <= 1 {
			// All files have same timestamp - skip this group
			logInfo("Skipped group %s: all files have same timestamp", key)
			continue
		}

		// Sort by timestamp, then version
		sort.Slice(group.Files, func(i, j int) bool {
			if group.Files[i].Timestamp != group.Files[j].Timestamp {
				return group.Files[i].Timestamp < group.Files[j].Timestamp
			}
			return group.Files[i].Version < group.Files[j].Version
		})

		// Additional safety: Check for same-version but different content files
		// These are likely different variants (e.g., "CLEAN" vs "GROSS", "ESP" vs "2K textures")
		// Skip if we detect suspicious patterns:
		if hasSuspiciousVersionPattern(group) {
			logWarning("Skipped group %s: detected same version with likely different content", key)
			continue
		}

		// PATCH/HOTFIX DETECTION:
		// If group contains both PATCH and MAIN/FULL files, they're NOT duplicates
		// Example: "AKM Complex - 1.0 - MAIN" (772 MB) vs "AKM Complex - 1.0.2 - PATCH" (715 KB)
		hasPatch := false
		hasMain := false
		
		for _, f := range group.Files {
			if f.IsPatch {
				hasPatch = true
			}
			if isFullOrMainFile(f.FileName) {
				hasMain = true
			}
		}
		
		// If we have both patch and main files, skip this group
		if hasPatch && hasMain {
			logWarning("Skipped group %s: contains both PATCH and MAIN files (not duplicates)", key)
			continue
		}
		
		// CRITICAL: If newest file is a patch/hotfix/update and significantly smaller than older versions
		// This likely means patch should be applied TO the old version, not replace it
		// Check this REGARDLESS of whether old files are labeled as "MAIN"
		newestFile := group.Files[len(group.Files)-1]
		shouldSkipPatch := false
		if newestFile.IsPatch && len(group.Files) > 1 {
			// Check size ratio - if patch is <10% of ANY older version size, this is suspicious
			for i := 0; i < len(group.Files)-1; i++ {
				oldFile := group.Files[i]
				sizeRatio := float64(newestFile.Size) / float64(oldFile.Size)
				
				if sizeRatio < 0.1 { // Patch is less than 10% of old version
					logWarning("Skipped group %s: newest file '%s' (%s) is %.2f%% size of '%s' (%s) - likely a patch that needs old file",
						key, newestFile.FileName, formatSize(newestFile.Size), 
						sizeRatio*100, oldFile.FileName, formatSize(oldFile.Size))
					shouldSkipPatch = true
					break
				}
			}
		}
		
		if shouldSkipPatch {
			continue
		}

		// Newest is last
		group.NewestIdx = len(group.Files) - 1

		// Calculate space to free (all except newest)
		for i := 0; i < group.NewestIdx; i++ {
			group.SpaceToFree += group.Files[i].Size
		}

		duplicates[key] = group
	}

	logInfo("Found %d mod groups with duplicates in %s", len(duplicates), folderPath)
	return duplicates, nil
}

func showDuplicatesReport(duplicates map[string]*ModGroup, gameName string) {
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\nDUPLICATE REPORT: %s", gameName)
	fmt.Printf("\n%s%s\n", strings.Repeat("=", 100), ColorReset)

	if len(duplicates) == 0 {
		fmt.Printf("%s[OK]%s No duplicates found!\n", ColorGreen, ColorReset)
		return
	}

	// Sort keys for consistent output
	keys := make([]string, 0, len(duplicates))
	for key := range duplicates {
		keys = append(keys, key)
	}
	sort.Strings(keys)

	count := 1
	for _, key := range keys {
		group := duplicates[key]

		fmt.Printf("\n%s%d. %s%s\n", ColorBold, count, group.ModKey, ColorReset)

		// Show newest (keep)
		newest := group.Files[group.NewestIdx]
		fmt.Printf("   %sKEEP:%s\n", ColorGreen, ColorReset)
		fmt.Printf("     └─ %s\n", newest.FileName)
		fmt.Printf("        Version: %s, Date: %s, Size: %s\n",
			newest.Version,
			timestampToDate(newest.Timestamp),
			formatSize(newest.Size))

		// Show old versions (delete)
		oldCount := len(group.Files) - 1
		fmt.Printf("\n   %sDELETE (%d old version(s)):%s\n", ColorRed, oldCount, ColorReset)
		for i := 0; i < group.NewestIdx; i++ {
			old := group.Files[i]
			fmt.Printf("     └─ %s\n", old.FileName)
			fmt.Printf("        Version: %s, Date: %s, Size: %s\n",
				old.Version,
				timestampToDate(old.Timestamp),
				formatSize(old.Size))
		}

		fmt.Printf("\n   %s[SPACE]%s Space to free: %s%s%s\n",
			ColorBlue, ColorReset, ColorYellow, formatSize(group.SpaceToFree), ColorReset)
		fmt.Printf("   %s\n", strings.Repeat("-", 95))

		count++
	}
}

func deleteOldVersions(duplicates map[string]*ModGroup, minSizeMB float64) (int, int64) {
	deletedCount := 0
	spaceFreed := int64(0)
	minSizeBytes := int64(minSizeMB * 1024 * 1024)

	for _, group := range duplicates {
		// Safety validation before deletion
		if !validateDeletionSafety(group) {
			fmt.Printf("%s[WARN]%s Skipping unsafe group: %s\n", ColorYellow, ColorReset, group.ModKey)
			logWarning("Skipped unsafe deletion for group: %s", group.ModKey)
			continue
		}

		// Validate mod group integrity
		if err := validateModGroup(group); err != nil {
			fmt.Printf("%s[WARN]%s Skipping invalid group %s: %v\n", ColorYellow, ColorReset, group.ModKey, err)
			logWarning("Invalid mod group %s: %v", group.ModKey, err)
			continue
		}

		// Delete all except newest
		for i := 0; i < group.NewestIdx; i++ {
			file := group.Files[i]

			// Check minimum size filter
			if file.Size < minSizeBytes {
				logInfo("Skipping %s: below minimum size (%s < %s)",
					file.FileName, formatSize(file.Size), formatSize(minSizeBytes))
				continue
			}

			// Additional safety: Verify file still exists
			if _, err := os.Stat(file.FullPath); os.IsNotExist(err) {
				logWarning("File no longer exists: %s", file.FullPath)
				continue
			}

			// Check if file is locked
			if isFileLocked(file.FullPath) {
				fmt.Printf("%s[WARN]%s File is locked (in use): %s\n", ColorYellow, ColorReset, file.FileName)
				logWarning("File locked: %s", file.FullPath)
				continue
			}

			// Log deletion attempt
			logInfo("Attempting to delete: %s (Size: %s)", file.FileName, formatSize(file.Size))

			// Delete main file
			err := os.Remove(file.FullPath)
			if err != nil {
				fmt.Printf("%s[ERROR]%s Failed to delete %s: %v\n", ColorRed, ColorReset, file.FileName, err)
				logError("Failed to delete %s: %v", file.FullPath, err)
				continue
			}

			deletedCount++
			spaceFreed += file.Size
			fmt.Printf("%s[OK]%s Deleted: %s (%s)\n", ColorGreen, ColorReset, file.FileName, formatSize(file.Size))
			logInfo("Successfully deleted: %s (%s)", file.FileName, formatSize(file.Size))

			// Delete .meta file
			metaPath := file.FullPath + ".meta"
			if _, err := os.Stat(metaPath); err == nil {
				if err := os.Remove(metaPath); err == nil {
					fmt.Printf("%s[OK]%s Deleted: %s.meta\n", ColorGreen, ColorReset, file.FileName)
					logInfo("Deleted meta file: %s.meta", file.FileName)
				} else {
					logWarning("Failed to delete meta file: %s.meta - %v", file.FileName, err)
				}
			}
		}
	}

	logInfo("Deletion complete: %d files, %s freed", deletedCount, formatSize(spaceFreed))
	return deletedCount, spaceFreed
}

func scanAllFolders(folders []string, deleteMode bool) {
	totalFiles := 0
	totalSpace := int64(0)
	totalDeleted := 0
	totalFreed := int64(0)

	for _, folder := range folders {
		duplicates, err := scanFolder(folder)
		if err != nil {
			fmt.Printf("%s[ERROR]%s Failed to scan %s: %v\n", ColorRed, ColorReset, filepath.Base(folder), err)
			continue
		}

		showDuplicatesReport(duplicates, filepath.Base(folder))

		// Count files and space
		for _, group := range duplicates {
			oldCount := len(group.Files) - 1
			totalFiles += oldCount
			totalSpace += group.SpaceToFree
		}

		if deleteMode {
			deleted, freed := deleteOldVersions(duplicates, 0)
			totalDeleted += deleted
			totalFreed += freed
		}
	}

	// Summary
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\nTOTAL SUMMARY")
	fmt.Printf("\n%s%s\n", strings.Repeat("=", 100), ColorReset)

	if deleteMode {
		fmt.Printf("Total deleted: %s%d%s files\n", ColorYellow, totalDeleted, ColorReset)
		fmt.Printf("Total space freed: %s%s%s\n", ColorYellow, formatSize(totalFreed), ColorReset)
	} else {
		fmt.Printf("Total old versions found: %s%d%s\n", ColorYellow, totalFiles, ColorReset)
		fmt.Printf("Total space to free: %s%s%s\n", ColorYellow, formatSize(totalSpace), ColorReset)
	}
}

func scanSpecificFolder(folders []string, scanner *bufio.Scanner, deleteMode bool) {
	fmt.Printf("\n%s[FOLDERS]%s Available folders:\n", ColorGreen, ColorReset)
	for i, folder := range folders {
		fmt.Printf("  %d. %s\n", i+1, filepath.Base(folder))
	}

	fmt.Print("\nSelect folder number: ")
	if !scanner.Scan() {
		return
	}

	choice, err := strconv.Atoi(strings.TrimSpace(scanner.Text()))
	if err != nil || choice < 1 || choice > len(folders) {
		fmt.Printf("%s[ERROR]%s Invalid selection!\n", ColorRed, ColorReset)
		return
	}

	selectedFolder := folders[choice-1]

	if deleteMode {
		fmt.Printf("\n%s[!] This will DELETE old versions from %s! Continue? (yes/no): %s",
			ColorRed, filepath.Base(selectedFolder), ColorReset)
		if !scanner.Scan() || !confirmInput(scanner.Text()) {
			fmt.Printf("%s[CANCELLED]%s Operation cancelled.\n", ColorYellow, ColorReset)
			return
		}
	}

	duplicates, err := scanFolder(selectedFolder)
	if err != nil {
		fmt.Printf("%s[ERROR]%s Failed to scan folder: %v\n", ColorRed, ColorReset, err)
		return
	}

	showDuplicatesReport(duplicates, filepath.Base(selectedFolder))

	if deleteMode {
		deleted, freed := deleteOldVersions(duplicates, 0)
		fmt.Printf("\n%s[OK] Completed!%s\n", ColorGreen, ColorReset)
		fmt.Printf("Deleted: %d files\n", deleted)
		fmt.Printf("Space freed: %s\n", formatSize(freed))
	} else {
		// Calculate totals
		totalFiles := 0
		totalSpace := int64(0)
		for _, group := range duplicates {
			totalFiles += len(group.Files) - 1
			totalSpace += group.SpaceToFree
		}
		fmt.Printf("\n%sTotal: %d old versions, %s to free%s\n",
			ColorYellow, totalFiles, formatSize(totalSpace), ColorReset)
	}
}

func deleteWithSizeFilter(folders []string, scanner *bufio.Scanner) {
	fmt.Print("\nEnter minimum file size in MB (only files >= this size will be deleted): ")
	if !scanner.Scan() {
		return
	}

	minSize, err := strconv.ParseFloat(strings.TrimSpace(scanner.Text()), 64)
	if err != nil || minSize < 0 {
		fmt.Printf("%s[ERROR]%s Invalid size value!\n", ColorRed, ColorReset)
		return
	}

	fmt.Printf("\n%s[!] This will DELETE old versions >= %.2f MB from ALL folders! Continue? (yes/no): %s",
		ColorRed, minSize, ColorReset)
	if !scanner.Scan() || !confirmInput(scanner.Text()) {
		fmt.Printf("%s[CANCELLED]%s Operation cancelled.\n", ColorYellow, ColorReset)
		return
	}

	totalDeleted := 0
	totalFreed := int64(0)

	for _, folder := range folders {
		fmt.Printf("\n%s[PROCESSING]%s %s\n", ColorBlue, ColorReset, filepath.Base(folder))

		duplicates, err := scanFolder(folder)
		if err != nil {
			fmt.Printf("%s[ERROR]%s Failed to scan %s: %v\n", ColorRed, ColorReset, filepath.Base(folder), err)
			continue
		}

		deleted, freed := deleteOldVersions(duplicates, minSize)
		totalDeleted += deleted
		totalFreed += freed
	}

	fmt.Printf("\n%s[OK] Completed!%s\n", ColorGreen, ColorReset)
	fmt.Printf("Total deleted: %d files\n", totalDeleted)
	fmt.Printf("Total space freed: %s\n", formatSize(totalFreed))
}

func confirmInput(input string) bool {
	input = strings.ToLower(strings.TrimSpace(input))
	return input == "yes" || input == "y" || input == "evet" || input == "e"
}

func timestampToDate(timestamp string) string {
	ts, err := strconv.ParseInt(timestamp, 10, 64)
	if err != nil {
		return "Unknown"
	}
	t := time.Unix(ts, 0)
	return t.Format("2006-01-02 15:04")
}

func formatSize(bytes int64) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}
	div, exp := int64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.2f %cB", float64(bytes)/float64(div), "KMGTPE"[exp])
}

// Logging functions
func initLogging() {
	logFileName := fmt.Sprintf("wabbajack-library-cleaner_%s.log", time.Now().Format("2006-01-02_15-04-05"))
	var err error
	config.LogFile, err = os.OpenFile(logFileName, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0666)
	if err != nil {
		log.Printf("Warning: Could not create log file: %v", err)
		config.LogFile = nil
		return
	}
	log.SetOutput(config.LogFile)
	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)
	logInfo("=== Wabbajack Cleanup Tool Started ===")
}

func logInfo(format string, args ...interface{}) {
	if config.LogFile != nil {
		log.Printf("[INFO] "+format, args...)
	}
}

func logWarning(format string, args ...interface{}) {
	if config.LogFile != nil {
		log.Printf("[WARN] "+format, args...)
	}
}

func logError(format string, args ...interface{}) {
	if config.LogFile != nil {
		log.Printf("[ERROR] "+format, args...)
	}
}

// Safety check: Ensure we're not deleting the newest file
func validateDeletionSafety(group *ModGroup) bool {
	if len(group.Files) <= 1 {
		logWarning("Group %s has only 1 file, skipping", group.ModKey)
		return false
	}

	newest := group.Files[group.NewestIdx]

	// Check if newest file actually exists
	if _, err := os.Stat(newest.FullPath); os.IsNotExist(err) {
		logError("Newest file doesn't exist: %s", newest.FullPath)
		return false
	}

	// Ensure we're not trying to delete the newest
	for i := 0; i < group.NewestIdx; i++ {
		if group.Files[i].Timestamp >= newest.Timestamp {
			logError("Safety check failed: Found file with timestamp >= newest: %s vs %s",
				group.Files[i].Timestamp, newest.Timestamp)
			return false
		}
	}

	return true
}

// Check if file is locked (being used by another process)
func isFileLocked(path string) bool {
	file, err := os.OpenFile(path, os.O_RDWR, 0666)
	if err != nil {
		// File might be locked or we don't have permissions
		return true
	}
	file.Close()
	return false
}

// Validate that the file matches expected Wabbajack pattern
func isWabbajackFile(filename string) bool {
	// Must have valid extension
	hasValidExt := false
	for _, ext := range archiveExtensions {
		if strings.HasSuffix(strings.ToLower(filename), ext) {
			hasValidExt = true
			break
		}
	}
	if !hasValidExt {
		return false
	}

	// Should contain at least one dash and a number (ModID)
	if !strings.Contains(filename, "-") {
		return false
	}

	// Should not be a temporary or partial file
	lowerName := strings.ToLower(filename)
	if strings.Contains(lowerName, ".part") ||
		strings.Contains(lowerName, ".tmp") ||
		strings.Contains(lowerName, ".download") ||
		strings.HasPrefix(lowerName, "~") {
		return false
	}

	return true
}

// Check available disk space before deletion
func checkDiskSpace(path string) (int64, error) {
	// For Windows, we can check via syscall, but for simplicity
	// we'll just log a warning if we can't determine it
	logWarning("Disk space check not implemented, proceeding with caution")
	return 0, nil
}

// Validate that files in a group are actually duplicates
func validateModGroup(group *ModGroup) error {
	if len(group.Files) < 2 {
		return fmt.Errorf("group has less than 2 files")
	}

	baseModID := group.Files[0].ModID

	// Ensure all files have same ModID
	// Note: ModName can vary between versions (e.g., "FO4Edit 4.1.5" vs "FO4Edit 4.0.4")
	// so we only validate that ModID matches
	for i, file := range group.Files {
		if file.ModID != baseModID {
			return fmt.Errorf("file %d has different ModID: %s vs %s", i, file.ModID, baseModID)
		}
	}

	return nil
}

// findWabbajackFiles searches for .wabbajack files in the base directory
func findWabbajackFiles(baseDir string) ([]string, error) {
	var wabbajackFiles []string

	entries, err := os.ReadDir(baseDir)
	if err != nil {
		return nil, err
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		if strings.HasSuffix(strings.ToLower(entry.Name()), ".wabbajack") {
			wabbajackFiles = append(wabbajackFiles, filepath.Join(baseDir, entry.Name()))
		}
	}

	return wabbajackFiles, nil
}

// parseWabbajackFile parses a .wabbajack file (ZIP archive) and extracts modlist information
func parseWabbajackFile(filePath string) (*ModlistInfo, error) {
	logInfo("Parsing wabbajack file: %s", filePath)

	reader, err := zip.OpenReader(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to open wabbajack file: %w", err)
	}
	defer reader.Close()

	// Find and read the "modlist" file (JSON without extension)
	var modlistFile *zip.File
	for _, file := range reader.File {
		if file.Name == "modlist" {
			modlistFile = file
			break
		}
	}

	if modlistFile == nil {
		return nil, fmt.Errorf("modlist file not found in archive")
	}

	rc, err := modlistFile.Open()
	if err != nil {
		return nil, fmt.Errorf("failed to open modlist file: %w", err)
	}
	defer rc.Close()

	data, err := io.ReadAll(rc)
	if err != nil {
		return nil, fmt.Errorf("failed to read modlist file: %w", err)
	}

	var modlist Modlist
	if err := json.Unmarshal(data, &modlist); err != nil {
		return nil, fmt.Errorf("failed to parse modlist JSON: %w", err)
	}

	// Build a map of used mod keys (ModID-based)
	usedModKeys := make(map[string]bool)
	for _, archive := range modlist.Archives {
		if archive.State.ModID > 0 {
			// Use ModID as the key for matching
			modKey := fmt.Sprintf("%d", archive.State.ModID)
			usedModKeys[modKey] = true
		}
	}

	info := &ModlistInfo{
		FilePath:    filePath,
		Name:        modlist.Name,
		ModCount:    len(modlist.Archives),
		UsedModKeys: usedModKeys,
	}

	logInfo("Parsed modlist '%s': %d archives, %d unique ModIDs", modlist.Name, len(modlist.Archives), len(usedModKeys))
	return info, nil
}

// getAllModFiles collects all mod files from game folders
func getAllModFiles(gameFolders []string) ([]ModFile, error) {
	var allFiles []ModFile

	for _, folder := range gameFolders {
		entries, err := os.ReadDir(folder)
		if err != nil {
			logWarning("Failed to read folder %s: %v", folder, err)
			continue
		}

		for _, entry := range entries {
			if entry.IsDir() {
				continue
			}

			filename := entry.Name()
			if !isWabbajackFile(filename) {
				continue
			}

			modFile := parseModFilename(filename)
			if modFile == nil {
				continue
			}

			fullPath := filepath.Join(folder, filename)
			info, err := os.Stat(fullPath)
			if err != nil {
				continue
			}

			modFile.FullPath = fullPath
			modFile.Size = info.Size()
			allFiles = append(allFiles, *modFile)
		}
	}

	return allFiles, nil
}

// detectOrphanedMods compares mod files with active modlists and finds orphaned mods
func detectOrphanedMods(modFiles []ModFile, activeModlists []*ModlistInfo) (used []ModFile, orphaned []OrphanedMod) {
	// Build a combined set of all used ModIDs from active modlists
	usedModIDs := make(map[string]bool)
	for _, modlist := range activeModlists {
		for modKey := range modlist.UsedModKeys {
			usedModIDs[modKey] = true
		}
	}

	logInfo("Total unique ModIDs in active modlists: %d", len(usedModIDs))

	// Classify each mod file
	for _, modFile := range modFiles {
		if usedModIDs[modFile.ModID] {
			used = append(used, modFile)
		} else {
			orphaned = append(orphaned, OrphanedMod{
				File: modFile,
			})
		}
	}

	logInfo("Classification complete: %d used, %d orphaned", len(used), len(orphaned))
	return used, orphaned
}

// scanOrphanedMods implements the orphaned mods detection feature
func scanOrphanedMods(baseDir string, gameFolders []string, scanner *bufio.Scanner, deleteMode bool) {
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\n%35s%s%s\n", "", "MODLIST-BASED CLEANUP", "")
	fmt.Printf("%s%s\n", strings.Repeat("=", 100), ColorReset)

	// Step 1: Find .wabbajack files
	wabbajackFiles, err := findWabbajackFiles(baseDir)
	if err != nil {
		fmt.Printf("%s[ERROR]%s Failed to search for wabbajack files: %v\n", ColorRed, ColorReset, err)
		logError("Failed to search for wabbajack files: %v", err)
		return
	}

	if len(wabbajackFiles) == 0 {
		fmt.Printf("\n%s[ERROR]%s No .wabbajack files found in directory!\n", ColorRed, ColorReset)
		fmt.Printf("\n%s[INFO]%s This feature requires .wabbajack modlist files to work.\n", ColorCyan, ColorReset)
		fmt.Printf("Place your .wabbajack files in: %s%s%s\n", ColorYellow, baseDir, ColorReset)
		fmt.Printf("\nExample:\n")
		fmt.Printf("  %sF:\\Wabbajack\\%s\n", ColorCyan, ColorReset)
		fmt.Printf("  ├─ Uranium Fever.wabbajack\n")
		fmt.Printf("  ├─ FAnomaly.wabbajack\n")
		fmt.Printf("  ├─ Skyrim\\          %s<-- Mod archives%s\n", ColorYellow, ColorReset)
		fmt.Printf("  └─ Fallout4\\        %s<-- Mod archives%s\n", ColorYellow, ColorReset)
		logInfo("No wabbajack files found in: %s", baseDir)
		return
	}

	fmt.Printf("\n%s[FOUND]%s Detected %d modlist file(s):\n", ColorGreen, ColorReset, len(wabbajackFiles))

	// Step 2: Parse all wabbajack files
	var modlistInfos []*ModlistInfo
	for i, wbFile := range wabbajackFiles {
		fmt.Printf("  %d. %s ... ", i+1, filepath.Base(wbFile))
		
		info, err := parseWabbajackFile(wbFile)
		if err != nil {
			fmt.Printf("%s[FAILED]%s %v\n", ColorRed, ColorReset, err)
			logError("Failed to parse %s: %v", wbFile, err)
			continue
		}
		
		fmt.Printf("%s[OK]%s (%d mods)\n", ColorGreen, ColorReset, info.ModCount)
		modlistInfos = append(modlistInfos, info)
	}

	if len(modlistInfos) == 0 {
		fmt.Printf("\n%s[ERROR]%s Failed to parse any modlist files!\n", ColorRed, ColorReset)
		return
	}

	// Step 3: Let user select which modlists they're using
	fmt.Printf("\n%s[SELECT]%s Which modlists are you CURRENTLY USING?\n", ColorCyan, ColorReset)
	for i, info := range modlistInfos {
		fmt.Printf("  [%d] %s (%d mods)\n", i+1, info.Name, info.ModCount)
	}
	fmt.Printf("\n%sEnter numbers separated by commas (e.g., 1,2,3) or 'all': %s", ColorBold, ColorReset)

	if !scanner.Scan() {
		return
	}

	selection := strings.TrimSpace(scanner.Text())
	var activeModlists []*ModlistInfo

	if strings.ToLower(selection) == "all" {
		activeModlists = modlistInfos
	} else {
		selections := strings.Split(selection, ",")
		for _, sel := range selections {
			idx, err := strconv.Atoi(strings.TrimSpace(sel))
			if err != nil || idx < 1 || idx > len(modlistInfos) {
				fmt.Printf("%s[WARN]%s Invalid selection: %s\n", ColorYellow, ColorReset, sel)
				continue
			}
			activeModlists = append(activeModlists, modlistInfos[idx-1])
		}
	}

	if len(activeModlists) == 0 {
		fmt.Printf("%s[ERROR]%s No valid modlists selected!\n", ColorRed, ColorReset)
		return
	}

	fmt.Printf("\n%s[SELECTED]%s Active modlists:\n", ColorGreen, ColorReset)
	for _, ml := range activeModlists {
		fmt.Printf("  ✓ %s\n", ml.Name)
	}

	// Step 4: Scan all mod files
	fmt.Printf("\n%s[SCANNING]%s Collecting mod files from game folders...\n", ColorCyan, ColorReset)
	allModFiles, err := getAllModFiles(gameFolders)
	if err != nil {
		fmt.Printf("%s[ERROR]%s Failed to collect mod files: %v\n", ColorRed, ColorReset, err)
		return
	}

	fmt.Printf("%s[OK]%s Found %d mod files\n", ColorGreen, ColorReset, len(allModFiles))

	// Step 5: Detect orphaned mods
	fmt.Printf("\n%s[ANALYZING]%s Detecting orphaned mods...\n", ColorCyan, ColorReset)
	usedMods, orphanedMods := detectOrphanedMods(allModFiles, activeModlists)

	// Step 6: Display results
	showOrphanedReport(usedMods, orphanedMods, activeModlists)

	// Step 7: Delete if in delete mode
	if deleteMode && len(orphanedMods) > 0 {
		totalSize := int64(0)
		for _, om := range orphanedMods {
			totalSize += om.File.Size
		}

		fmt.Printf("\n%s[WARNING]%s This will DELETE %d orphaned mods (%s)!\n",
			ColorRed, ColorReset, len(orphanedMods), formatSize(totalSize))
		fmt.Printf("%sType 'DELETE' (in uppercase) to confirm: %s", ColorRed, ColorReset)

		if !scanner.Scan() {
			return
		}

		if strings.TrimSpace(scanner.Text()) != "DELETE" {
			fmt.Printf("%s[CANCELLED]%s Operation cancelled.\n", ColorYellow, ColorReset)
			logInfo("Orphaned mod deletion cancelled by user")
			return
		}

		deleteOrphanedMods(orphanedMods)
	}
}

// showOrphanedReport displays a detailed report of used and orphaned mods
func showOrphanedReport(usedMods []ModFile, orphanedMods []OrphanedMod, activeModlists []*ModlistInfo) {
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\nRESULTS")
	fmt.Printf("\n%s%s\n", strings.Repeat("=", 100), ColorReset)

	// Calculate totals
	usedSize := int64(0)
	for _, mod := range usedMods {
		usedSize += mod.Size
	}

	orphanedSize := int64(0)
	for _, om := range orphanedMods {
		orphanedSize += om.File.Size
	}

	// Show used mods summary
	fmt.Printf("\n%s✓ USED MODS:%s %d mods (%s)\n", ColorGreen, ColorReset, len(usedMods), formatSize(usedSize))
	fmt.Printf("  These mods are used by your active modlist(s):\n")
	for _, ml := range activeModlists {
		fmt.Printf("    • %s\n", ml.Name)
	}

	// Show orphaned mods summary
	fmt.Printf("\n%s✗ ORPHANED MODS:%s %d mods (%s)\n", ColorRed, ColorReset, len(orphanedMods), formatSize(orphanedSize))
	if len(orphanedMods) == 0 {
		fmt.Printf("  %sNo orphaned mods found! Your library is clean.%s\n", ColorGreen, ColorReset)
	} else {
		fmt.Printf("  These mods are NOT used by any of your active modlists.\n")
		fmt.Printf("  They may be from deleted or inactive modlists.\n\n")

		// Show some examples (up to 10)
		exampleCount := len(orphanedMods)
		if exampleCount > 10 {
			exampleCount = 10
		}

		fmt.Printf("  %sExamples:%s\n", ColorYellow, ColorReset)
		for i := 0; i < exampleCount; i++ {
			om := orphanedMods[i]
			fmt.Printf("    • %s (%s)\n", om.File.FileName, formatSize(om.File.Size))
		}

		if len(orphanedMods) > 10 {
			fmt.Printf("    ... and %d more\n", len(orphanedMods)-10)
		}
	}

	fmt.Printf("\n%s%s\n", strings.Repeat("=", 100), ColorReset)
}

// deleteOrphanedMods deletes orphaned mod files
func deleteOrphanedMods(orphanedMods []OrphanedMod) {
	deletedCount := 0
	spaceFreed := int64(0)

	fmt.Printf("\n%s[DELETING]%s Starting deletion...\n", ColorCyan, ColorReset)

	for _, om := range orphanedMods {
		file := om.File

		// Verify file still exists
		if _, err := os.Stat(file.FullPath); os.IsNotExist(err) {
			logWarning("File no longer exists: %s", file.FullPath)
			continue
		}

		// Check if file is locked
		if isFileLocked(file.FullPath) {
			fmt.Printf("%s[WARN]%s File is locked (in use): %s\n", ColorYellow, ColorReset, file.FileName)
			logWarning("File locked: %s", file.FullPath)
			continue
		}

		// Delete main file
		err := os.Remove(file.FullPath)
		if err != nil {
			fmt.Printf("%s[ERROR]%s Failed to delete %s: %v\n", ColorRed, ColorReset, file.FileName, err)
			logError("Failed to delete %s: %v", file.FullPath, err)
			continue
		}

		deletedCount++
		spaceFreed += file.Size
		fmt.Printf("%s[OK]%s Deleted: %s (%s)\n", ColorGreen, ColorReset, file.FileName, formatSize(file.Size))
		logInfo("Successfully deleted orphaned mod: %s (%s)", file.FileName, formatSize(file.Size))

		// Delete .meta file
		metaPath := file.FullPath + ".meta"
		if _, err := os.Stat(metaPath); err == nil {
			if err := os.Remove(metaPath); err == nil {
				fmt.Printf("%s[OK]%s Deleted: %s.meta\n", ColorGreen, ColorReset, file.FileName)
				logInfo("Deleted meta file: %s.meta", file.FileName)
			} else {
				logWarning("Failed to delete meta file: %s.meta - %v", file.FileName, err)
			}
		}
	}

	fmt.Printf("\n%s[COMPLETED]%s\n", ColorGreen, ColorReset)
	fmt.Printf("Total deleted: %s%d%s files\n", ColorYellow, deletedCount, ColorReset)
	fmt.Printf("Total space freed: %s%s%s\n", ColorYellow, formatSize(spaceFreed), ColorReset)
	logInfo("Orphaned mod deletion complete: %d files, %s freed", deletedCount, formatSize(spaceFreed))
}

// viewStatistics displays statistics about the mod library
func viewStatistics(gameFolders []string) {
	fmt.Printf("\n%s%s", ColorPurple, strings.Repeat("=", 100))
	fmt.Printf("\n%35s%s%s\n", "", "LIBRARY STATISTICS", "")
	fmt.Printf("%s%s\n", strings.Repeat("=", 100), ColorReset)

	totalFiles := 0
	totalSize := int64(0)
	gameStats := make(map[string]struct {
		files int
		size  int64
	})

	for _, folder := range gameFolders {
		entries, err := os.ReadDir(folder)
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
		gameStats[gameName] = struct {
			files int
			size  int64
		}{gameFiles, gameSize}

		totalFiles += gameFiles
		totalSize += gameSize
	}

	fmt.Printf("\n%s[OVERALL]%s\n", ColorBold, ColorReset)
	fmt.Printf("  Total Files: %s%d%s\n", ColorYellow, totalFiles, ColorReset)
	fmt.Printf("  Total Size:  %s%s%s\n", ColorYellow, formatSize(totalSize), ColorReset)

	fmt.Printf("\n%s[BY GAME]%s\n", ColorBold, ColorReset)
	for gameName, stats := range gameStats {
		fmt.Printf("  %s%s%s\n", ColorCyan, gameName, ColorReset)
		fmt.Printf("    Files: %d\n", stats.files)
		fmt.Printf("    Size:  %s\n", formatSize(stats.size))
	}

	fmt.Printf("\n%s%s\n", strings.Repeat("=", 100), ColorReset)
}
