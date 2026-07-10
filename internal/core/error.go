package core

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type GitNavigatorError struct {
	Code    ErrorCode
	Message string
}

type ErrorCode int

const (
	ErrNotInGitRepo ErrorCode = iota
	ErrFileNotFound
	ErrInvalidUtf8Path
	ErrNoIndicesProvided
	ErrInvalidIndexFormat
	ErrNoValidIndices
	ErrIndexOutOfRange
	ErrNoFilesAvailable
	ErrCacheDirectoryNotFound
	ErrCacheLoadError
	ErrCacheFileNotFound
	ErrNoCachedFiles
	ErrNoValidFilesSelected
	ErrGitAddFailed
	ErrUpdateFailed
	ErrAlreadyUpToDate
	ErrUpdateCanceled
	ErrConfigError
	ErrRollbackFailed
	ErrVersionNotFound
	ErrIo
	ErrCustom
)

type Result[T any] struct {
	Value T
	Err   *GitNavigatorError
}

func (r Result[T]) Ok() (T, error) {
	if r.Err != nil {
		return r.Value, r.Err
	}
	return r.Value, nil
}

func (r Result[T]) IsOk() bool {
	return r.Err == nil
}

func NotInGitRepo() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNotInGitRepo, Message: "Not in a git repository"}
}

func FileNotFound(path string) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrFileNotFound, Message: fmt.Sprintf("File does not exist: %s", path)}
}

func InvalidUtf8Path() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrInvalidUtf8Path, Message: "Invalid UTF-8 path in repository"}
}

func NoIndicesProvided() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoIndicesProvided, Message: "No file indices provided. Usage: <command> <indices>"}
}

func InvalidIndexFormat(input string) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrInvalidIndexFormat, Message: fmt.Sprintf("Invalid index format: %s. Use format like: 1, 1-3, or 1,3,5", input)}
}

func NoValidIndices() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoValidIndices, Message: "No valid indices provided. Use format like: 1, 1-3, or 1,3,5"}
}

func InvalidRangeFormat(rangeStr string) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrInvalidIndexFormat, Message: fmt.Sprintf("Invalid range format: '%s'. Use format like '3-6'", rangeStr)}
}

func IndexOutOfRange(index, max int) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrIndexOutOfRange, Message: fmt.Sprintf("Index %d is out of range (1-%d available)", index, max)}
}

func NoFilesAvailable() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoFilesAvailable, Message: "No files available to operate on"}
}

func CacheDirectoryNotFound() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrCacheDirectoryNotFound, Message: "Could not find cache directory"}
}

func CacheFileNotFound(path string) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrCacheFileNotFound, Message: fmt.Sprintf("Cache file does not exist at '%s'. Run 'gs' first to generate file list.", path)}
}

func NoCachedFiles() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoCachedFiles, Message: "No cached files found. Run 'gs' first to generate file list."}
}

func NoAvailableFiles() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoFilesAvailable, Message: "No files available. Run 'gs' first to see available files."}
}

func NoValidFilesSelected() *GitNavigatorError {
	return &GitNavigatorError{Code: ErrNoValidFilesSelected, Message: "No valid files found for the specified indices."}
}

func CustomError(msg string) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrCustom, Message: msg}
}

func IoError(err error) *GitNavigatorError {
	return &GitNavigatorError{Code: ErrIo, Message: fmt.Sprintf("IO error: %v", err)}
}

func (e *GitNavigatorError) Error() string {
	return e.Message
}

func NewResult[T any](value T, err error) Result[T] {
	if err == nil {
		return Result[T]{Value: value, Err: nil}
	}
	if gerr, ok := err.(*GitNavigatorError); ok {
		return Result[T]{Value: value, Err: gerr}
	}
	return Result[T]{Value: value, Err: &GitNavigatorError{Code: ErrCustom, Message: err.Error()}}
}

func NewOkResult[T any](value T) Result[T] {
	return Result[T]{Value: value, Err: nil}
}

func NewErrorResult[T any](err *GitNavigatorError) Result[T] {
	return Result[T]{Value: *new(T), Err: err}
}

type CacheInfo struct {
	Path string
	Err  error
}

func LoadCache(repoPath string) (*StateCache, *GitNavigatorError) {
	cacheDir, err := GetCacheDir(repoPath)
	if err != nil {
		return nil, CacheDirectoryNotFound()
	}

	cacheFile := filepath.Join(cacheDir, "files.json")
	data, err := os.ReadFile(cacheFile)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, CacheFileNotFound(cacheFile)
		}
		return nil, IoError(err)
	}

	var cache StateCache
	if err := json.Unmarshal(data, &cache); err != nil {
		return nil, CustomError(fmt.Sprintf("Failed to parse cache file: %v", err))
	}

	if len(cache.Files) == 0 {
		return nil, NoCachedFiles()
	}

	return &cache, nil
}

func SaveCache(cache *StateCache, repoPath string) *GitNavigatorError {
	cacheDir, err := GetCacheDir(repoPath)
	if err != nil {
		return CacheDirectoryNotFound()
	}

	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		return IoError(err)
	}

	cacheFile := filepath.Join(cacheDir, "files.json")
	data, err := json.MarshalIndent(cache, "", "  ")
	if err != nil {
		return CustomError(fmt.Sprintf("Failed to serialize cache data: %v", err))
	}

	if err := os.WriteFile(cacheFile, data, 0644); err != nil {
		return IoError(err)
	}

	return nil
}
