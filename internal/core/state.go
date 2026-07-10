package core

import (
	"crypto/sha256"
	"fmt"
	"os"
	"path/filepath"
)

type GitStatus int

const (
	GitStatusModified GitStatus = iota
	GitStatusAdded
	GitStatusDeleted
	GitStatusRenamed
	GitStatusCopied
	GitStatusTypeChanged
	GitStatusUntracked
	GitStatusUnmerged
)

func (s GitStatus) String() string {
	switch s {
	case GitStatusModified:
		return "M"
	case GitStatusAdded:
		return "A"
	case GitStatusDeleted:
		return "D"
	case GitStatusRenamed:
		return "R"
	case GitStatusCopied:
		return "C"
	case GitStatusTypeChanged:
		return "T"
	case GitStatusUntracked:
		return "??"
	case GitStatusUnmerged:
		return "UU"
	default:
		return "?"
	}
}

func (s GitStatus) Description() string {
	switch s {
	case GitStatusModified:
		return "modified"
	case GitStatusAdded:
		return "new"
	case GitStatusDeleted:
		return "deleted"
	case GitStatusRenamed:
		return "renamed"
	case GitStatusCopied:
		return "copied"
	case GitStatusTypeChanged:
		return "type changed"
	case GitStatusUntracked:
		return "untracked"
	case GitStatusUnmerged:
		return "both modified"
	default:
		return "unknown"
	}
}

func (s GitStatus) SortPriority(staged bool) int {
	priority := map[GitStatus]int{
		GitStatusUnmerged:    0,
		GitStatusAdded:       1,
		GitStatusModified:    2,
		GitStatusDeleted:     3,
		GitStatusRenamed:     4,
		GitStatusCopied:      5,
		GitStatusTypeChanged: 6,
		GitStatusUntracked:   12,
	}

	base := priority[s]
	if s != GitStatusUnmerged && s != GitStatusUntracked && staged {
		return base
	}
	if s != GitStatusUnmerged && s != GitStatusUntracked && !staged {
		return base + 6
	}
	return base
}

type FileEntry struct {
	Index  int
	Status GitStatus
	Path   string
	Staged bool
}

type BranchEntry struct {
	Index        string
	Name         string
	IsCurrent    bool
	TrackingInfo string
	IsRemote     bool
}

type StateCache struct {
	Files       []FileEntry `json:"files"`
	LastUpdated int64       `json:"last_updated"`
	RepoPath    string      `json:"repo_path"`
}

func GetCacheDir(repoPath string) (string, error) {
	cacheHome := os.Getenv("XDG_CACHE_HOME")
	if cacheHome == "" {
		home, err := os.UserCacheDir()
		if err != nil {
			return "", fmt.Errorf("failed to get cache directory: %w", err)
		}
		cacheHome = filepath.Join(home, "git-navigator")
	}

	hash := hashRepoPath(repoPath)
	return filepath.Join(cacheHome, hash), nil
}

func hashRepoPath(repoPath string) string {
	h := sha256.New()
	h.Write([]byte(repoPath))
	return fmt.Sprintf("%x", h.Sum(nil))[:16]
}
