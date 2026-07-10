package core

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"runtime"
	"strings"

	"github.com/Masterminds/semver/v3"
	"github.com/fatih/color"
	selfupdate "github.com/minio/selfupdate"
)

type Asset struct {
	Name        string `json:"name"`
	DownloadURL string `json:"browser_download_url"`
}

type ReleaseInfo struct {
	Version string `json:"version"`
	Body    string `json:"body"`
	Assets  []Asset `json:"assets"`
}

type UpdateError struct {
	Code    string
	Message string
}

func (e *UpdateError) Error() string {
	return e.Message
}

func NewUpdateError(code, message string) *UpdateError {
	return &UpdateError{Code: code, Message: message}
}

var (
	ErrNetworkFailure    = NewUpdateError("NETWORK", "No se pudo conectar a GitHub. Verifica tu conexión.")
	ErrParseError       = NewUpdateError("PARSE", "Error al procesar respuesta de GitHub.")
	ErrAssetNotFound    = NewUpdateError("ASSET_NOT_FOUND", "No se encontró binario para esta plataforma.")
	ErrDownloadFailed   = NewUpdateError("DOWNLOAD", "Error al descargar actualización.")
	ErrChecksumMismatch = NewUpdateError("CHECKSUM", "El archivo descargado está corrupto.")
	ErrApplyFailed      = NewUpdateError("APPLY", "No se pudo reemplazar el binario. Es posible que necesites reiniciar.")
)

func GetTarget() string {
	goos := runtime.GOOS
	goarch := runtime.GOARCH
	env := runtime.GOOS

	switch {
	case goos == "windows" && goarch == "amd64":
		return "windows-x64.exe"
	case goos == "linux" && goarch == "amd64" && env == "gnu":
		return "linux-x64"
	case goos == "linux" && goarch == "arm64":
		return "linux-arm64"
	case goos == "linux" && goarch == "amd64" && env == "musl":
		return "linux-musl-x64"
	case goos == "darwin" && goarch == "amd64":
		return "darwin-x64"
	case goos == "darwin" && goarch == "arm64":
		return "darwin-arm64"
	default:
		return goos + "-" + goarch
	}
}

func CheckForUpdate(currentVersion string) (*ReleaseInfo, error) {
	repo := DefaultRepositoryConfig()
	url := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/latest", repo.Owner, repo.Name)

	resp, err := http.Get(url)
	if err != nil {
		return nil, ErrNetworkFailure
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, ErrNetworkFailure
	}

	var release struct {
		TagName string `json:"tag_name"`
		Body    string `json:"body"`
		Assets  []struct {
			Name               string `json:"name"`
			BrowserDownloadURL string `json:"browser_download_url"`
		} `json:"assets"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&release); err != nil {
		return nil, ErrParseError
	}

	version := strings.TrimPrefix(release.TagName, "v")

	assets := make([]Asset, len(release.Assets))
	for i, a := range release.Assets {
		assets[i] = Asset{
			Name:        a.Name,
			DownloadURL: a.BrowserDownloadURL,
		}
	}

	return &ReleaseInfo{
		Version: version,
		Body:    release.Body,
		Assets:  assets,
	}, nil
}

func NeedsUpdate(currentVersion, latestVersion string) (bool, error) {
	c, err := semver.NewVersion(currentVersion)
	if err != nil {
		c, err = semver.NewVersion("v" + currentVersion)
		if err != nil {
			return false, fmt.Errorf("invalid current version: %w", err)
		}
	}
	l, err := semver.NewVersion(latestVersion)
	if err != nil {
		l, err = semver.NewVersion("v" + latestVersion)
		if err != nil {
			return false, fmt.Errorf("invalid latest version: %w", err)
		}
	}
	return l.GreaterThan(c), nil
}

func FindAsset(release *ReleaseInfo, target string) *Asset {
	for i := range release.Assets {
		if strings.Contains(release.Assets[i].Name, target) {
			return &release.Assets[i]
		}
	}
	return nil
}

func DownloadAsset(asset *Asset, target string) (string, error) {
	fmt.Printf("  Descargando %s...\n", color.BlueString(asset.Name))

	resp, err := http.Get(asset.DownloadURL)
	if err != nil {
		return "", ErrDownloadFailed
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", ErrDownloadFailed
	}

	tmpFile, err := os.CreateTemp("", "git-navigator-*")
	if err != nil {
		return "", ErrDownloadFailed
	}
	defer tmpFile.Close()

	if _, err := io.Copy(tmpFile, resp.Body); err != nil {
		os.Remove(tmpFile.Name())
		return "", ErrDownloadFailed
	}

	return tmpFile.Name(), nil
}

func ApplyUpdate(archivePath string) error {
	file, err := os.Open(archivePath)
	if err != nil {
		return ErrApplyFailed
	}
	defer file.Close()

	hash := sha256.New()
	reader := io.TeeReader(file, hash)

	err = selfupdate.Apply(reader, selfupdate.Options{})
	if err != nil {
		if rerr := selfupdate.RollbackError(err); rerr != nil {
			return fmt.Errorf("%w: rollback failed: %v", ErrApplyFailed, rerr)
		}
		return fmt.Errorf("%w: %v", ErrApplyFailed, err)
	}

	os.Remove(archivePath)

	return nil
}

func BackupCurrent(version string) error {
	backupDir, err := GetBackupDir()
	if err != nil {
		return err
	}

	currentExe, err := os.Executable()
	if err != nil {
		return fmt.Errorf("cannot determine current executable: %w", err)
	}

	data, err := os.ReadFile(currentExe)
	if err != nil {
		return fmt.Errorf("cannot read current executable: %w", err)
	}

	backupPath := fmt.Sprintf("%s/git-navigator-v%s", backupDir, version)
	if err := os.WriteFile(backupPath, data, 0755); err != nil {
		return fmt.Errorf("cannot create backup: %w", err)
	}

	fmt.Printf("  Backup creado: %s\n", color.BlueString(backupPath))
	return nil
}

func FormatReleaseNotes(body string, maxLines int) string {
	var lines []string
	for _, line := range strings.Split(body, "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		line = strings.TrimPrefix(line, "- ")
		line = strings.TrimPrefix(line, "* ")
		if line != "" && !strings.HasPrefix(line, "#") && !strings.HasPrefix(line, "##") {
			lines = append(lines, line)
			if len(lines) >= maxLines {
				break
			}
		}
	}
	return strings.Join(lines, "\n")
}
