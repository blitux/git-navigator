package core

import (
	"os"
	"path/filepath"
	"testing"
)

func TestGitNavigatorError_Error(t *testing.T) {
	err := &GitNavigatorError{
		Code:    ErrCustom,
		Message: "test error message",
	}

	if got := err.Error(); got != "test error message" {
		t.Errorf("GitNavigatorError.Error() = %v, want %v", got, "test error message")
	}
}

func TestNotInGitRepo(t *testing.T) {
	err := NotInGitRepo()

	if err.Code != ErrNotInGitRepo {
		t.Errorf("NotInGitRepo().Code = %v, want %v", err.Code, ErrNotInGitRepo)
	}
	if err.Message != "Not in a git repository" {
		t.Errorf("NotInGitRepo().Message = %v", err.Message)
	}
}

func TestFileNotFound(t *testing.T) {
	err := FileNotFound("/path/to/file.txt")

	if err.Code != ErrFileNotFound {
		t.Errorf("FileNotFound().Code = %v, want %v", err.Code, ErrFileNotFound)
	}
	if err.Message != "File does not exist: /path/to/file.txt" {
		t.Errorf("FileNotFound().Message = %v", err.Message)
	}
}

func TestInvalidUtf8Path(t *testing.T) {
	err := InvalidUtf8Path()

	if err.Code != ErrInvalidUtf8Path {
		t.Errorf("InvalidUtf8Path().Code = %v, want %v", err.Code, ErrInvalidUtf8Path)
	}
}

func TestNoIndicesProvided(t *testing.T) {
	err := NoIndicesProvided()

	if err.Code != ErrNoIndicesProvided {
		t.Errorf("NoIndicesProvided().Code = %v, want %v", err.Code, ErrNoIndicesProvided)
	}
}

func TestInvalidIndexFormat(t *testing.T) {
	err := InvalidIndexFormat("abc")

	if err.Code != ErrInvalidIndexFormat {
		t.Errorf("InvalidIndexFormat().Code = %v, want %v", err.Code, ErrInvalidIndexFormat)
	}
	if err.Message != "Invalid index format: abc. Use format like: 1, 1-3, or 1,3,5" {
		t.Errorf("InvalidIndexFormat().Message = %v", err.Message)
	}
}

func TestNoValidIndices(t *testing.T) {
	err := NoValidIndices()

	if err.Code != ErrNoValidIndices {
		t.Errorf("NoValidIndices().Code = %v, want %v", err.Code, ErrNoValidIndices)
	}
}

func TestInvalidRangeFormat(t *testing.T) {
	err := InvalidRangeFormat("5-")

	if err.Code != ErrInvalidIndexFormat {
		t.Errorf("InvalidRangeFormat().Code = %v, want %v", err.Code, ErrInvalidIndexFormat)
	}
	if err.Message != "Invalid range format: '5-'. Use format like '3-6'" {
		t.Errorf("InvalidRangeFormat().Message = %v", err.Message)
	}
}

func TestIndexOutOfRange(t *testing.T) {
	err := IndexOutOfRange(10, 5)

	if err.Code != ErrIndexOutOfRange {
		t.Errorf("IndexOutOfRange().Code = %v, want %v", err.Code, ErrIndexOutOfRange)
	}
	if err.Message != "Index 10 is out of range (1-5 available)" {
		t.Errorf("IndexOutOfRange().Message = %v", err.Message)
	}
}

func TestNoFilesAvailable(t *testing.T) {
	err := NoFilesAvailable()

	if err.Code != ErrNoFilesAvailable {
		t.Errorf("NoFilesAvailable().Code = %v, want %v", err.Code, ErrNoFilesAvailable)
	}
}

func TestCacheDirectoryNotFound(t *testing.T) {
	err := CacheDirectoryNotFound()

	if err.Code != ErrCacheDirectoryNotFound {
		t.Errorf("CacheDirectoryNotFound().Code = %v, want %v", err.Code, ErrCacheDirectoryNotFound)
	}
}

func TestCacheFileNotFound(t *testing.T) {
	err := CacheFileNotFound("/cache/files.json")

	if err.Code != ErrCacheFileNotFound {
		t.Errorf("CacheFileNotFound().Code = %v, want %v", err.Code, ErrCacheFileNotFound)
	}
}

func TestNoCachedFiles(t *testing.T) {
	err := NoCachedFiles()

	if err.Code != ErrNoCachedFiles {
		t.Errorf("NoCachedFiles().Code = %v, want %v", err.Code, ErrNoCachedFiles)
	}
}

func TestNoAvailableFiles(t *testing.T) {
	err := NoAvailableFiles()

	if err.Code != ErrNoFilesAvailable {
		t.Errorf("NoAvailableFiles().Code = %v, want %v", err.Code, ErrNoFilesAvailable)
	}
}

func TestNoValidFilesSelected(t *testing.T) {
	err := NoValidFilesSelected()

	if err.Code != ErrNoValidFilesSelected {
		t.Errorf("NoValidFilesSelected().Code = %v, want %v", err.Code, ErrNoValidFilesSelected)
	}
}

func TestCustomError(t *testing.T) {
	err := CustomError("custom error message")

	if err.Code != ErrCustom {
		t.Errorf("CustomError().Code = %v, want %v", err.Code, ErrCustom)
	}
	if err.Message != "custom error message" {
		t.Errorf("CustomError().Message = %v", err.Message)
	}
}

func TestIoError(t *testing.T) {
	originalErr := os.ErrPermission
	err := IoError(originalErr)

	if err.Code != ErrIo {
		t.Errorf("IoError().Code = %v, want %v", err.Code, ErrIo)
	}
	if err.Message != "IO error: permission denied" {
		t.Errorf("IoError().Message = %v", err.Message)
	}
}

func TestLoadCache_Success(t *testing.T) {
	repoPath := "/test/repo"

	cacheDir, err := GetCacheDir(repoPath)
	if err != nil {
		t.Fatalf("GetCacheDir() error: %v", err)
	}
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		t.Fatalf("MkdirAll() error: %v", err)
	}

	cacheFile := filepath.Join(cacheDir, "files.json")
	os.WriteFile(cacheFile, []byte(`{"files":[{"Index":1,"Status":0,"Path":"test.txt","Staged":true}],"last_updated":1234567890,"repo_path":"/test"}`), 0644)

	cache, gnErr := LoadCache(repoPath)

	if gnErr != nil {
		t.Errorf("LoadCache() unexpected error: %v", gnErr)
	}
	if cache == nil {
		t.Error("LoadCache() expected non-nil cache")
	}
	if len(cache.Files) != 1 {
		t.Errorf("LoadCache().Files length = %v, want 1", len(cache.Files))
	}
}

func TestLoadCache_EmptyCache(t *testing.T) {
	repoPath := "/test/empty"

	cacheDir, err := GetCacheDir(repoPath)
	if err != nil {
		t.Fatalf("GetCacheDir() error: %v", err)
	}
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		t.Fatalf("MkdirAll() error: %v", err)
	}

	cacheFile := filepath.Join(cacheDir, "files.json")
	os.WriteFile(cacheFile, []byte(`{"files":[],"last_updated":0,"repo_path":"/test"}`), 0644)

	cache, gnErr := LoadCache(repoPath)

	if gnErr == nil {
		t.Error("LoadCache() expected error for empty cache")
	}
	if gnErr.Code != ErrNoCachedFiles {
		t.Errorf("LoadCache() error.Code = %v, want ErrNoCachedFiles", gnErr.Code)
	}
	if cache != nil {
		t.Error("LoadCache() expected nil cache on error")
	}
}

func TestLoadCache_InvalidJSON(t *testing.T) {
	repoPath := "/test/invalid"

	cacheDir, err := GetCacheDir(repoPath)
	if err != nil {
		t.Fatalf("GetCacheDir() error: %v", err)
	}
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		t.Fatalf("MkdirAll() error: %v", err)
	}

	cacheFile := filepath.Join(cacheDir, "files.json")
	os.WriteFile(cacheFile, []byte(`{invalid json}`), 0644)

	cache, gnErr := LoadCache(repoPath)

	if gnErr == nil {
		t.Error("LoadCache() expected error for invalid JSON")
	}
	if gnErr.Code != ErrCustom {
		t.Errorf("LoadCache() error.Code = %v, want ErrCustom", gnErr.Code)
	}
	if cache != nil {
		t.Error("LoadCache() expected nil cache on error")
	}
}

func TestSaveCache(t *testing.T) {
	repoPath := "/test/save"

	cache := &StateCache{
		Files: []FileEntry{
			{Index: 1, Status: GitStatusModified, Path: "test.txt", Staged: true},
		},
		LastUpdated: 1234567890,
		RepoPath:    "/test/repo",
	}

	err := SaveCache(cache, repoPath)

	if err != nil {
		t.Errorf("SaveCache() unexpected error: %v", err)
	}

	cacheDir, _ := GetCacheDir(repoPath)
	cachePath := filepath.Join(cacheDir, "files.json")
	if _, statErr := os.Stat(cachePath); os.IsNotExist(statErr) {
		t.Error("SaveCache() did not create cache file")
	}

	data, readErr := os.ReadFile(cachePath)
	if readErr != nil {
		t.Errorf("SaveCache() failed to read back cache: %v", readErr)
	}
	if len(data) == 0 {
		t.Error("SaveCache() wrote empty file")
	}
}

func TestLoadSaveCache_RoundTrip(t *testing.T) {
	repoPath := "/test/roundtrip"

	original := &StateCache{
		Files: []FileEntry{
			{Index: 1, Status: GitStatusModified, Path: "a.txt", Staged: true},
			{Index: 2, Status: GitStatusAdded, Path: "b.txt", Staged: false},
		},
		LastUpdated: 1234567890,
		RepoPath:    "/test/repo",
	}

	err := SaveCache(original, repoPath)
	if err != nil {
		t.Fatalf("SaveCache() error: %v", err)
	}

	loaded, err := LoadCache(repoPath)
	if err != nil {
		t.Fatalf("LoadCache() error: %v", err)
	}

	if len(loaded.Files) != len(original.Files) {
		t.Errorf("LoadSaveCache_RoundTrip: Files length = %v, want %v", len(loaded.Files), len(original.Files))
	}
	if loaded.LastUpdated != original.LastUpdated {
		t.Errorf("LoadSaveCache_RoundTrip: LastUpdated = %v, want %v", loaded.LastUpdated, original.LastUpdated)
	}
	if loaded.RepoPath != original.RepoPath {
		t.Errorf("LoadSaveCache_RoundTrip: RepoPath = %v, want %v", loaded.RepoPath, original.RepoPath)
	}
}
