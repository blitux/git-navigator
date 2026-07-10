package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
)

func ExecuteReset(args []string) {
	if len(args) == 1 && args[0] == "." {
		executeResetAll()
		return
	}

	cwd, err := os.Getwd()
	if err != nil {
		core.PrintError("Could not determine current directory")
		return
	}

	gitRepo, gerr := core.OpenGitRepo(cwd)
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	cache, gerr := core.LoadCache(cwd)
	if gerr != nil {
		core.PrintError(fmt.Sprintf("Cannot load file cache: %s. Run 'gs' first.", gerr.Message))
		return
	}

	if len(cache.Files) == 0 {
		core.PrintError("No files available to reset")
		ExecuteStatus()
		return
	}

	if len(args) == 0 {
		core.PrintErrorWithUsage("No file indices provided", []string{
			"grs <index>...",
			"grs .",
		})
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

	paths := make([]string, len(indices))
	for i, idx := range indices {
		paths[i] = cache.Files[idx-1].Path
	}

	if gerr := gitRepo.ResetFiles(paths); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	fileWord := "file"
	if len(paths) > 1 {
		fileWord = "files"
	}
	core.Template().Success(fmt.Sprintf("Successfully reset %d %s from git index.", len(paths), fileWord)).Print()

	fmt.Println("Updated status:")
	ExecuteStatus()
}

func executeResetAll() {
	cwd, err := os.Getwd()
	if err != nil {
		core.PrintError("Could not determine current directory")
		return
	}

	_, gerr := core.OpenGitRepo(cwd)
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	if gerr := core.RunGitReset(); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	core.Template().Success("Successfully reset all staged files from git index.").Print()
	fmt.Println("Updated status:")
	ExecuteStatus()
}
