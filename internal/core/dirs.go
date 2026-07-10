package core

import (
	"os"
	"path/filepath"
	"runtime"
)

func GetConfigDir() (string, error) {
	base := getConfigBase()
	dir := filepath.Join(base, "git-navigator")
	if err := os.MkdirAll(dir, 0755); err != nil {
		return "", err
	}
	return dir, nil
}

func GetBackupDir() (string, error) {
	configDir, err := GetConfigDir()
	if err != nil {
		return "", err
	}
	backupDir := filepath.Join(configDir, "backups")
	if err := os.MkdirAll(backupDir, 0755); err != nil {
		return "", err
	}
	return backupDir, nil
}

func getConfigBase() string {
	goos := runtime.GOOS
	switch goos {
	case "linux", "freebsd", "netbsd", "openbsd", "darwin":
		if xdg := os.Getenv("XDG_CONFIG_HOME"); xdg != "" {
			return xdg
		}
		home, _ := os.UserHomeDir()
		if home != "" {
			if goos == "darwin" {
				return filepath.Join(home, "Library", "Application Support")
			}
			return filepath.Join(home, ".config")
		}
		return "~/.config"
	case "windows":
		if appdata := os.Getenv("APPDATA"); appdata != "" {
			return appdata
		}
		return "."
	default:
		if xdg := os.Getenv("XDG_CONFIG_HOME"); xdg != "" {
			return xdg
		}
		home, _ := os.UserHomeDir()
		if home != "" {
			return filepath.Join(home, ".config")
		}
		return "~/.config"
	}
}
