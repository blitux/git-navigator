package core

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/Masterminds/semver/v3"
	"github.com/fatih/color"
)

type BackupInfo struct {
	Version string
	Path    string
	Size    int64
	Created time.Time
}

func ListBackups() ([]BackupInfo, error) {
	backupDir, err := GetBackupDir()
	if err != nil {
		return nil, err
	}

	if !backupDirExists(backupDir) {
		return []BackupInfo{}, nil
	}

	entries, err := os.ReadDir(backupDir)
	if err != nil {
		return nil, err
	}

	var backups []BackupInfo
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		name := entry.Name()
		if !strings.HasPrefix(name, "git-navigator-v") {
			continue
		}
		version := strings.TrimPrefix(name, "git-navigator-v")
		info, err := entry.Info()
		if err != nil {
			continue
		}
		backups = append(backups, BackupInfo{
			Version: version,
			Path:    filepath.Join(backupDir, name),
			Size:    info.Size(),
			Created: info.ModTime(),
		})
	}

	sort.Slice(backups, func(i, j int) bool {
		vi, _ := semver.NewVersion(backups[i].Version)
		vj, _ := semver.NewVersion(backups[j].Version)
		if vi == nil || vj == nil {
			return backups[i].Version > backups[j].Version
		}
		return vi.GreaterThan(vj)
	})

	return backups, nil
}

func backupDirExists(backupDir string) bool {
	entries, err := os.ReadDir(backupDir)
	if err != nil {
		return false
	}
	for _, entry := range entries {
		if !entry.IsDir() && strings.HasPrefix(entry.Name(), "git-navigator-v") {
			return true
		}
	}
	return false
}

func RestoreFromBackup(version string) error {
	backupDir, err := GetBackupDir()
	if err != nil {
		return err
	}
	backupPath := filepath.Join(backupDir, "git-navigator-v"+version)

	if _, err := os.Stat(backupPath); os.IsNotExist(err) {
		return fmt.Errorf("backup v%s not found", version)
	}

	currentExe, err := os.Executable()
	if err != nil {
		return fmt.Errorf("cannot determine current executable: %w", err)
	}

	backupData, err := os.ReadFile(backupPath)
	if err != nil {
		return fmt.Errorf("cannot read backup: %w", err)
	}

	if err := os.WriteFile(currentExe, backupData, 0755); err != nil {
		return fmt.Errorf("cannot restore binary: %w", err)
	}

	return nil
}

func FormatBackupsList(backups []BackupInfo) string {
	if len(backups) == 0 {
		return "  No hay backups disponibles"
	}

	var lines []string
	for i, b := range backups {
		sizeKB := b.Size / 1024
		date := b.Created.Format("2006-01-02 15:04")
		idx := color.BlackString("["+strconv.Itoa(i+1)+"]")
		ver := color.BlueString("v" + b.Version)
		size := color.BlackString(strconv.FormatInt(sizeKB, 10) + " KB")
		dateStr := color.BlackString(date)
		lines = append(lines, fmt.Sprintf("  %s  %s  (%s, %s)", idx, ver, size, dateStr))
	}
	return strings.Join(lines, "\n")
}
