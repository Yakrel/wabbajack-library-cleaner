package main

import (
	"bufio"
	"fmt"
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

var config Config

var archiveExtensions = []string{".7z", ".zip", ".rar", ".tar", ".gz", ".exe"}

func main() {
	// Enable ANSI colors on Windows
	enableWindowsColors()

	// Initialize logging
	initLogging()
	defer config.LogFile.Close()

	baseDir, err := os.Getwd()
	if err != nil {
		fmt.Printf("%s[ERROR]%s Failed to get working directory: %v\n", ColorRed, ColorReset, err)
		logError("Failed to get working directory: %v", err)
		return
	}

	logInfo("Program started in directory: %s", baseDir)

	fmt.Printf("%s%sWorking directory: %s%s\n", ColorBold, ColorCyan, baseDir, ColorReset)

	gameFolders, err := getGameFolders(baseDir)
	if err != nil {
		fmt.Printf("%s[ERROR]%s %v\n", ColorRed, ColorReset, err)
		return
	}

	// Check if we're inside a mod folder (no subfolders, but many archives)
	if len(gameFolders) == 0 {
		archiveCount := countArchivesInDir(baseDir)
		if archiveCount > 10 {
			fmt.Printf("\n%s[WARNING]%s It looks like you're inside a game folder!\n", ColorYellow, ColorReset)
			fmt.Printf("Found %d archive files in current directory.\n\n", archiveCount)
			fmt.Printf("This tool should be placed in the parent directory.\n")
			fmt.Printf("Example: %sF:\\Wabbajack\\%s (not F:\\Wabbajack\\Skyrim\\)\n\n", ColorCyan, ColorReset)

			fmt.Printf("Would you like to clean THIS folder anyway? (yes/no): ")
			scanner := bufio.NewScanner(os.Stdin)
			if scanner.Scan() && confirmInput(scanner.Text()) {
				// Add current directory as the only folder with a clear name
				gameFolders = append(gameFolders, baseDir)
				fmt.Printf("\n%s[OK]%s Processing current directory...\n", ColorGreen, ColorReset)
				logInfo("User chose to clean current directory: %s", baseDir)
			} else {
				fmt.Printf("\n%s[INFO]%s Please move the tool to the correct directory and run again.\n", ColorCyan, ColorReset)
				return
			}
		} else {
			fmt.Printf("%s[ERROR]%s No game folders found!\n", ColorRed, ColorReset)
			fmt.Printf("Make sure you're in the Wabbajack downloads directory.\n")
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
		fmt.Print("\nSelect option (1-3): ")

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
	fmt.Printf("%s1.%s Scan folder (Dry-run - preview only)\n", ColorBold, ColorReset)
	fmt.Printf("%s2.%s Clean folder (Delete old versions)\n", ColorBold, ColorReset)
	fmt.Printf("%s3.%s Exit\n", ColorBold, ColorReset)
	fmt.Printf("\n%s[!] Always run Dry-run first!%s\n", ColorYellow, ColorReset)
}

func enableWindowsColors() {
	// Enable ANSI color support on Windows 10+
	if runtime := os.Getenv("OS"); strings.Contains(runtime, "Windows") {
		// This is handled automatically in Go 1.15+
	}
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
	}
}

func isNumeric(s string) bool {
	_, err := strconv.Atoi(s)
	return err == nil
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

		modKey := modFile.ModName + "-" + modFile.ModID

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

		// Sort by timestamp, then version
		sort.Slice(group.Files, func(i, j int) bool {
			if group.Files[i].Timestamp != group.Files[j].Timestamp {
				return group.Files[i].Timestamp < group.Files[j].Timestamp
			}
			return group.Files[i].Version < group.Files[j].Version
		})

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
	baseModName := group.Files[0].ModName

	// Ensure all files have same ModID and ModName
	for i, file := range group.Files {
		if file.ModID != baseModID {
			return fmt.Errorf("file %d has different ModID: %s vs %s", i, file.ModID, baseModID)
		}
		if file.ModName != baseModName {
			return fmt.Errorf("file %d has different ModName: %s vs %s", i, file.ModName, baseModName)
		}
	}

	return nil
}
