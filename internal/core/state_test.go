package core

import (
	"os"
	"testing"
)

func TestGitStatus_String(t *testing.T) {
	tests := []struct {
		status GitStatus
		want   string
	}{
		{GitStatusModified, "M"},
		{GitStatusAdded, "A"},
		{GitStatusDeleted, "D"},
		{GitStatusRenamed, "R"},
		{GitStatusCopied, "C"},
		{GitStatusTypeChanged, "T"},
		{GitStatusUntracked, "??"},
		{GitStatusUnmerged, "UU"},
		{GitStatus(100), "?"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.status.String(); got != tt.want {
				t.Errorf("GitStatus.String() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestGitStatus_Description(t *testing.T) {
	tests := []struct {
		status GitStatus
		want   string
	}{
		{GitStatusModified, "modified"},
		{GitStatusAdded, "new"},
		{GitStatusDeleted, "deleted"},
		{GitStatusRenamed, "renamed"},
		{GitStatusCopied, "copied"},
		{GitStatusTypeChanged, "type changed"},
		{GitStatusUntracked, "untracked"},
		{GitStatusUnmerged, "both modified"},
		{GitStatus(100), "unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.status.Description(); got != tt.want {
				t.Errorf("GitStatus.Description() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestGitStatus_SortPriority(t *testing.T) {
	tests := []struct {
		name   string
		status GitStatus
		staged bool
		want   int
	}{
		{"unmerged staged", GitStatusUnmerged, true, 0},
		{"unmerged unstaged", GitStatusUnmerged, false, 0},
		{"added staged", GitStatusAdded, true, 1},
		{"added unstaged", GitStatusAdded, false, 7},
		{"modified staged", GitStatusModified, true, 2},
		{"modified unstaged", GitStatusModified, false, 8},
		{"deleted staged", GitStatusDeleted, true, 3},
		{"deleted unstaged", GitStatusDeleted, false, 9},
		{"renamed staged", GitStatusRenamed, true, 4},
		{"renamed unstaged", GitStatusRenamed, false, 10},
		{"copied staged", GitStatusCopied, true, 5},
		{"copied unstaged", GitStatusCopied, false, 11},
		{"type changed staged", GitStatusTypeChanged, true, 6},
		{"type changed unstaged", GitStatusTypeChanged, false, 12},
		{"untracked staged", GitStatusUntracked, true, 12},
		{"untracked unstaged", GitStatusUntracked, false, 12},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.status.SortPriority(tt.staged); got != tt.want {
				t.Errorf("GitStatus.SortPriority(%v) = %v, want %v", tt.staged, got, tt.want)
			}
		})
	}
}

func TestHashRepoPath(t *testing.T) {
	tests := []struct {
		name     string
		repoPath string
		wantLen  int
	}{
		{"simple path", "/home/user/project", 16},
		{"empty path", "", 16},
		{"long path", "/home/user/very/long/path/to/some/really/deep/directory/project", 16},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := hashRepoPath(tt.repoPath)
			if len(got) != tt.wantLen {
				t.Errorf("hashRepoPath() returned string of length %v, want %v", len(got), tt.wantLen)
			}
		})
	}

	h1 := hashRepoPath("/path/one")
	h2 := hashRepoPath("/path/two")
	if h1 == h2 {
		t.Error("hashRepoPath() returned same hash for different inputs")
	}

	h3 := hashRepoPath("/path/one")
	if h1 != h3 {
		t.Error("hashRepoPath() returned different hash for same input")
	}
}

func TestGetCacheDir(t *testing.T) {
	os.Setenv("XDG_CACHE_HOME", "")
	os.Unsetenv("XDG_CACHE_HOME")

	t.Run("returns consistent hash for same path", func(t *testing.T) {
		os.Unsetenv("XDG_CACHE_HOME")

		dir1, _ := GetCacheDir("/some/path")
		dir2, _ := GetCacheDir("/some/path")

		if dir1 != dir2 {
			t.Errorf("GetCacheDir() returned different dirs for same path: %v vs %v", dir1, dir2)
		}
	})

	t.Run("returns different hash for different paths", func(t *testing.T) {
		os.Unsetenv("XDG_CACHE_HOME")

		dir1, _ := GetCacheDir("/path/one")
		dir2, _ := GetCacheDir("/path/two")

		if dir1 == dir2 {
			t.Error("GetCacheDir() returned same dir for different paths")
		}
	})
}

func TestFileEntry(t *testing.T) {
	entry := FileEntry{
		Index:  1,
		Status: GitStatusModified,
		Path:   "test.txt",
		Staged: false,
	}

	if entry.Index != 1 {
		t.Errorf("FileEntry.Index = %v, want 1", entry.Index)
	}
	if entry.Status != GitStatusModified {
		t.Errorf("FileEntry.Status = %v, want GitStatusModified", entry.Status)
	}
	if entry.Path != "test.txt" {
		t.Errorf("FileEntry.Path = %v, want test.txt", entry.Path)
	}
	if entry.Staged != false {
		t.Errorf("FileEntry.Staged = %v, want false", entry.Staged)
	}
}

func TestBranchEntry(t *testing.T) {
	entry := BranchEntry{
		Index:        "1",
		Name:         "main",
		IsCurrent:    true,
		TrackingInfo: "origin/main",
		IsRemote:     false,
	}

	if entry.Index != "1" {
		t.Errorf("BranchEntry.Index = %v, want 1", entry.Index)
	}
	if entry.Name != "main" {
		t.Errorf("BranchEntry.Name = %v, want main", entry.Name)
	}
	if !entry.IsCurrent {
		t.Error("BranchEntry.IsCurrent = false, want true")
	}
	if entry.TrackingInfo != "origin/main" {
		t.Errorf("BranchEntry.TrackingInfo = %v, want origin/main", entry.TrackingInfo)
	}
	if entry.IsRemote {
		t.Error("BranchEntry.IsRemote = true, want false")
	}
}

func TestStateCache(t *testing.T) {
	cache := StateCache{
		Files: []FileEntry{
			{Index: 1, Status: GitStatusModified, Path: "a.txt", Staged: true},
			{Index: 2, Status: GitStatusAdded, Path: "b.txt", Staged: false},
		},
		LastUpdated: 1234567890,
		RepoPath:    "/test/repo",
	}

	if len(cache.Files) != 2 {
		t.Errorf("StateCache.Files length = %v, want 2", len(cache.Files))
	}
	if cache.LastUpdated != 1234567890 {
		t.Errorf("StateCache.LastUpdated = %v, want 1234567890", cache.LastUpdated)
	}
	if cache.RepoPath != "/test/repo" {
		t.Errorf("StateCache.RepoPath = %v, want /test/repo", cache.RepoPath)
	}
}
