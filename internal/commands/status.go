package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteStatus() {
	cwd, err := getCurrentDir()
	if err != nil {
		core.PrintError("Not in a git repository")
		return
	}

	gitRepo, gerr := core.OpenGitRepo(cwd)
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	branch, _ := gitRepo.GetCurrentBranch()
	hash, message, _ := gitRepo.GetParentCommitInfo()

	fmt.Println()
	branchColor := color.BlueString(branch)
	fmt.Printf("Branch: %s", branchColor)

	ahead, behind, hasUpstream := gitRepo.GetAheadBehind()
	if hasUpstream && (ahead > 0 || behind > 0) {
		fmt.Print(" (")
		if ahead > 0 {
			fmt.Printf("%s+", color.WhiteString("%d", ahead))
		}
		if behind > 0 {
			fmt.Printf("%s-", color.WhiteString("%d", behind))
		}
		fmt.Print(")")
	}
	fmt.Println()

	if hash == "" {
		fmt.Printf("Parent: %s\n", color.WhiteString(message))
	} else {
		fmt.Printf("Parent: %s %s\n", color.BlueString(hash), color.BlackString(message))
	}
	fmt.Println()

	files, gerr := gitRepo.GetStatus()
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	if len(files) == 0 {
		return
	}

	printGroupedStatus(files)

	cache := &core.StateCache{
		Files:       files,
		Branches:    []core.BranchEntry{},
		LastUpdated: nowUnix(),
		RepoPath:    gitRepo.GetWorkDir(),
	}
	if gerr := core.SaveCache(cache, gitRepo.GetWorkDir()); gerr != nil {
	}
}

func printGroupedStatus(files []core.FileEntry) {
	var staged, unstaged, untracked, unmerged []core.FileEntry

	for _, f := range files {
		switch {
		case f.Status == core.GitStatusUnmerged:
			unmerged = append(unmerged, f)
		case f.Status == core.GitStatusUntracked:
			untracked = append(untracked, f)
		case f.Staged:
			staged = append(staged, f)
		default:
			unstaged = append(unstaged, f)
		}
	}

	printSection("Unmerged:", color.RedString, unmerged, "both modified")
	printSection("Staged:", color.GreenString, staged, "")
	printSection("Not staged:", color.YellowString, unstaged, "")
	printSection("Untracked:", color.CyanString, untracked, "untracked")
}

func printSection(title string, titleColor func(string, ...interface{}) string, files []core.FileEntry, desc string) {
	if len(files) == 0 {
		return
	}

	fmt.Printf("%s %s\n", titleColor("➤"), titleColor(title))
	for _, f := range files {
		statusDesc := desc
		if statusDesc == "" {
			statusDesc = f.Status.Description()
		}
		padding := 12 - len(statusDesc)
		if padding < 1 {
			padding = 1
		}
		fmt.Printf("   %s%s [%s] %s\n",
			color.BlackString("("+statusDesc+")"),
			repeatSpaces(padding),
			color.WhiteString(fmt.Sprintf("%d", f.Index)),
			getColoredPath(f.Status, f.Path))
	}
	fmt.Println()
}

func getColoredPath(status core.GitStatus, path string) string {
	switch status {
	case core.GitStatusModified:
		return color.YellowString(path)
	case core.GitStatusUntracked:
		return color.CyanString(path)
	case core.GitStatusDeleted:
		return color.RedString(path)
	case core.GitStatusAdded:
		return color.GreenString(path)
	case core.GitStatusRenamed, core.GitStatusCopied:
		return color.BlueString(path)
	case core.GitStatusTypeChanged:
		return color.MagentaString(path)
	case core.GitStatusUnmerged:
		return color.RedString(path)
	default:
		return path
	}
}

func repeatSpaces(n int) string {
	result := ""
	for i := 0; i < n; i++ {
		result += " "
	}
	return result
}

func nowUnix() int64 {
	return 0
}

func getCurrentDir() (string, error) {
	return os.Getwd()
}
