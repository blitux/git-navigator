package core

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
)

type GitRepo struct {
	repoPath string
	workDir  string
}

func OpenGitRepo(path string) (*GitRepo, *GitNavigatorError) {
	repoPath, err := findGitRoot(path)
	if err != nil {
		return nil, NotInGitRepo()
	}

	workDir, err := getWorkDir(repoPath)
	if err != nil {
		return nil, CustomError(fmt.Sprintf("Repository has no working directory: %v", err))
	}

	return &GitRepo{repoPath: repoPath, workDir: workDir}, nil
}

func findGitRoot(path string) (string, error) {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return "", err
	}

	for {
		gitPath := filepath.Join(absPath, ".git")
		if _, err := os.Stat(gitPath); err == nil {
			return absPath, nil
		}

		parent := filepath.Dir(absPath)
		if parent == absPath {
			return "", fmt.Errorf("not a git repository")
		}
		absPath = parent
	}
}

func getWorkDir(repoPath string) (string, error) {
	dotGit := filepath.Join(repoPath, ".git")
	info, err := os.Stat(dotGit)
	if err != nil {
		return "", err
	}

	if info.IsDir() {
		return repoPath, nil
	}

	content, err := os.ReadFile(dotGit)
	if err != nil {
		return "", err
	}

	gitDir := ""
	for _, line := range strings.Split(string(content), "\n") {
		if strings.HasPrefix(line, "gitdir:") {
			gitDir = strings.TrimSpace(strings.TrimPrefix(line, "gitdir:"))
			break
		}
	}

	if gitDir == "" {
		return "", fmt.Errorf("could not find workdir from gitlink")
	}

	return filepath.Dir(gitDir), nil
}

func (g *GitRepo) GetStatus() ([]FileEntry, *GitNavigatorError) {
	cmd := exec.Command("git", "status", "--porcelain=v1", "-uall")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return nil, IoError(err)
	}

	files := []FileEntry{}
	lines := strings.Split(out.String(), "\n")
	lineNum := 0

	for _, line := range lines {
		if len(line) < 3 {
			continue
		}

		staged := line[0] != ' ' && line[0] != '?'
		unstaged := line[1] != ' '

		XY := line[:2]
		path := strings.TrimSpace(line[3:])

		status := parseStatus(XY)

		if staged && status != GitStatusUntracked {
			lineNum++
			files = append(files, FileEntry{
				Index:  lineNum,
				Status: status,
				Path:   path,
				Staged: true,
			})
		}

		if unstaged || (!staged && status == GitStatusUntracked) {
			lineNum++
			files = append(files, FileEntry{
				Index:  lineNum,
				Status: status,
				Path:   path,
				Staged: false,
			})
		}
	}

	sort.Slice(files, func(i, j int) bool {
		priI := files[i].Status.SortPriority(files[i].Staged)
		priJ := files[j].Status.SortPriority(files[j].Staged)
		if priI != priJ {
			return priI < priJ
		}
		return files[i].Path < files[j].Path
	})

	for i := range files {
		files[i].Index = i + 1
	}

	return files, nil
}

func parseStatus(xy string) GitStatus {
	statusMap := map[string]GitStatus{
		"M":  GitStatusModified,
		"A":  GitStatusAdded,
		"D":  GitStatusDeleted,
		"R":  GitStatusRenamed,
		"C":  GitStatusCopied,
		"T":  GitStatusTypeChanged,
		"??": GitStatusUntracked,
		"UU": GitStatusUnmerged,
		"AM": GitStatusModified,
		"MM": GitStatusModified,
		"AD": GitStatusDeleted,
		"MD": GitStatusDeleted,
	}

	if status, ok := statusMap[xy]; ok {
		return status
	}

	if strings.Contains(xy, "M") {
		return GitStatusModified
	}
	if strings.Contains(xy, "D") {
		return GitStatusDeleted
	}
	if strings.Contains(xy, "A") {
		return GitStatusAdded
	}
	if strings.Contains(xy, "?") {
		return GitStatusUntracked
	}

	return GitStatusModified
}

func (g *GitRepo) GetCurrentBranch() (string, *GitNavigatorError) {
	cmd := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return "detached", nil
	}

	branch := strings.TrimSpace(out.String())
	if branch == "HEAD" {
		commit := g.GetCurrentCommit()
		if commit != "" {
			return fmt.Sprintf("detached at %s", commit[:7]), nil
		}
		return "detached", nil
	}

	return branch, nil
}

func (g *GitRepo) GetCurrentCommit() string {
	cmd := exec.Command("git", "rev-parse", "--short", "HEAD")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Run()
	return strings.TrimSpace(out.String())
}

func (g *GitRepo) GetParentCommitInfo() (string, string, *GitNavigatorError) {
	cmd := exec.Command("git", "log", "-1", "--pretty=format:%h %s")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return "", "- no commits yet -", nil
	}

	parts := strings.SplitN(strings.TrimSpace(out.String()), " ", 2)
	if len(parts) < 2 {
		return parts[0], "", nil
	}
	return parts[0], parts[1], nil
}

func (g *GitRepo) GetAheadBehind() (int, int, bool) {
	cmd := exec.Command("git", "rev-list", "--left-right", "--count", "@{upstream}...HEAD")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return 0, 0, false
	}

	var ahead, behind int
	fmt.Sscanf(out.String(), "%d %d", &ahead, &behind)
	return ahead, behind, true
}

func (g *GitRepo) AddFiles(paths []string) *GitNavigatorError {
	if len(paths) == 0 {
		return nil
	}

	args := append([]string{"add", "--"}, paths...)
	cmd := exec.Command("git", args...)
	cmd.Dir = g.workDir
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git add failed: %v", err))
	}
	return nil
}

func (g *GitRepo) ResetFiles(paths []string) *GitNavigatorError {
	if len(paths) == 0 {
		return nil
	}

	args := append([]string{"reset", "HEAD", "--"}, paths...)
	cmd := exec.Command("git", args...)
	cmd.Dir = g.workDir
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git reset failed: %v", err))
	}
	return nil
}

func (g *GitRepo) CheckoutFiles(paths []string) *GitNavigatorError {
	if len(paths) == 0 {
		return nil
	}

	args := append([]string{"checkout", "--"}, paths...)
	cmd := exec.Command("git", args...)
	cmd.Dir = g.workDir
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git checkout failed: %v", err))
	}
	return nil
}

func (g *GitRepo) CreateBranch(name string) *GitNavigatorError {
	cmd := exec.Command("git", "checkout", "-b", name)
	cmd.Dir = g.workDir
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git checkout -b failed: %v", err))
	}
	return nil
}

func (g *GitRepo) CheckoutBranch(name string) *GitNavigatorError {
	cmd := exec.Command("git", "checkout", name)
	cmd.Dir = g.workDir
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git checkout failed: %v", err))
	}
	return nil
}

func (g *GitRepo) GetLocalBranches() ([]BranchEntry, *GitNavigatorError) {
	cmd := exec.Command("git", "branch", "-v")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return nil, IoError(err)
	}

	currentBranch, _ := g.GetCurrentBranch()
	lines := strings.Split(out.String(), "\n")
	branches := []BranchEntry{}

	branchRe := regexp.MustCompile(`^\*?\s*(\S+)\s+([a-f0-9]+)?\s*(.*)$`)

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		matches := branchRe.FindStringSubmatch(line)
		if len(matches) < 2 {
			continue
		}

		name := matches[1]
		isCurrent := strings.HasPrefix(line, "*")

		tracking := ""
		if len(matches) >= 4 {
			tracking = matches[3]
		}
		if tracking == "" {
			tracking = "(no upstream)"
		}

		if isCurrent {
			branches = append([]BranchEntry{{
				Index:        "*",
				Name:         currentBranch,
				IsCurrent:    true,
				TrackingInfo: tracking,
				IsRemote:     false,
			}}, branches...)
		} else {
			branches = append(branches, BranchEntry{
				Index:        fmt.Sprintf("%d", len(branches)),
				Name:         name,
				IsCurrent:    false,
				TrackingInfo: tracking,
				IsRemote:     false,
			})
		}
	}

	if len(branches) > 0 && branches[0].IsCurrent {
		for i := 1; i < len(branches); i++ {
			branches[i].Index = fmt.Sprintf("%d", i)
		}
	}

	return branches, nil
}

func (g *GitRepo) GetRemoteBranches() ([]BranchEntry, *GitNavigatorError) {
	cmd := exec.Command("git", "branch", "-r", "-v")
	cmd.Dir = g.workDir
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return nil, IoError(err)
	}

	lines := strings.Split(out.String(), "\n")
	branches := []BranchEntry{}

	branchRe := regexp.MustCompile(`^\s*(\S+)/(\S+)\s+([a-f0-9]+)?\s*(.*)$`)

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "HEAD ->") {
			continue
		}

		matches := branchRe.FindStringSubmatch(line)
		if len(matches) < 3 {
			continue
		}

		remote := matches[1]
		name := matches[1] + "/" + matches[2]

		branches = append(branches, BranchEntry{
			Index:        fmt.Sprintf("%d", len(branches)+1),
			Name:         name,
			IsCurrent:    false,
			TrackingInfo: remote,
			IsRemote:     true,
		})
	}

	return branches, nil
}

func (g *GitRepo) GetWorkDir() string {
	return g.workDir
}

func RunGitAddAll() *GitNavigatorError {
	cmd := exec.Command("git", "add", ".")
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git add . failed: %v", err))
	}
	return nil
}

func RunGitReset() *GitNavigatorError {
	cmd := exec.Command("git", "reset")
	if err := cmd.Run(); err != nil {
		return CustomError(fmt.Sprintf("git reset failed: %v", err))
	}
	return nil
}
