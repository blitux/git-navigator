package core

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"
)

const (
	RepoOwner = "blitux"
	RepoName  = "git-navigator"
	BinName   = "git-navigator"
)

type RepositoryConfig struct {
	Owner   string `json:"owner"`
	Name    string `json:"name"`
	BinName string `json:"bin_name"`
}

func DefaultRepositoryConfig() RepositoryConfig {
	return RepositoryConfig{
		Owner:   RepoOwner,
		Name:    RepoName,
		BinName: BinName,
	}
}

type UpdateConfig struct {
	LastCheck        *time.Time `json:"last_check,omitempty"`
	AutoCheckEnabled bool       `json:"auto_check_enabled"`
	SkipVersion      string     `json:"skip_version,omitempty"`
}

type InstallConfig struct {
	InstalledVersion string             `json:"installed_version"`
	InstallDate     time.Time          `json:"install_date"`
	BinaryPath      string             `json:"binary_path"`
	Repository      RepositoryConfig  `json:"repository"`
	UpdateConfig    UpdateConfig       `json:"update_config"`
}

func (c *InstallConfig) Save() error {
	configDir, err := GetConfigDir()
	if err != nil {
		return err
	}
	configFile := filepath.Join(configDir, "config.json")
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(configFile, data, 0644)
}

func LoadOrCreateConfig() (*InstallConfig, error) {
	configDir, err := GetConfigDir()
	if err != nil {
		return nil, err
	}
	configFile := filepath.Join(configDir, "config.json")

	if data, err := os.ReadFile(configFile); err == nil {
		var config InstallConfig
		if err := json.Unmarshal(data, &config); err == nil {
			return &config, nil
		}
	}

	currentExe, err := os.Executable()
	if err != nil {
		currentExe = ""
	}

	config := &InstallConfig{
		InstalledVersion: "",
		InstallDate:     time.Now().UTC(),
		BinaryPath:      currentExe,
		Repository:      DefaultRepositoryConfig(),
		UpdateConfig: UpdateConfig{
			AutoCheckEnabled: false,
		},
	}
	if err := config.Save(); err != nil {
		return nil, err
	}
	return config, nil
}

func (c *InstallConfig) UpdateVersion(newVersion string) error {
	now := time.Now().UTC()
	c.InstalledVersion = newVersion
	c.UpdateConfig.LastCheck = &now
	return c.Save()
}
