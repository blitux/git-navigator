package commands

import (
	"fmt"
	"os/exec"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteDiff(args []string) {
	if len(args) == 0 {
		core.PrintErrorWithUsage("No file indices provided", []string{
			"gd <index>...",
		})
		return
	}

	gitRepo, cache := getRepoAndCache()
	if gitRepo == nil {
		return
	}

	if len(cache.Files) == 0 {
		core.PrintError("No files found in cache")
		return
	}

	indices, gerr := core.ParseIndices(args[0])
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	if gerr := core.ValidateIndices(indices, len(cache.Files)); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	files := make([]core.FileEntry, len(indices))
	for i, idx := range indices {
		files[i] = cache.Files[idx-1]
	}

	allUntracked := true
	for _, f := range files {
		if f.Status != core.GitStatusUntracked {
			allUntracked = false
			break
		}
	}

	if !allUntracked {
		fmt.Println(color.BlueString("Showing diff for %d file(s):", len(files)))
	}

	for i, file := range files {
		if len(files) > 1 && i > 0 {
			fmt.Println()
		}
		showFileDiff(gitRepo.GetWorkDir(), &file)
	}
}

func showFileDiff(workDir string, file *core.FileEntry) {
	if file.Status == core.GitStatusUntracked {
		core.PrintError(fmt.Sprintf("File is untracked: %s. No diff to show.", file.Path))
		return
	}

	var cmd *exec.Cmd

	switch file.Status {
	case core.GitStatusDeleted:
		cmd = exec.Command("git", "diff", "--color", "HEAD", "--", file.Path)
	default:
		if file.Staged {
			cmd = exec.Command("git", "diff", "--cached", "--color", "HEAD", "--", file.Path)
		} else {
			cmd = exec.Command("git", "diff", "--color", "--", file.Path)
		}
	}

	cmd.Dir = workDir
	output, err := cmd.Output()
	if err != nil {
		if len(output) > 0 {
			fmt.Printf("No changes to show for %s\n", file.Path)
		} else {
			core.PrintError(fmt.Sprintf("git diff failed: %v", err))
		}
		return
	}

	fmt.Print(string(output))
}
