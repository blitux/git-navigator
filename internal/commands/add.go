package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
)

func ExecuteAdd(args []string) {
	if len(args) == 1 && args[0] == "." {
		executeAddAll()
		return
	}

	gitRepo, cache := getRepoAndCache()
	if gitRepo == nil {
		return
	}

	if len(cache.Files) == 0 {
		core.PrintError("No files available to add")
		ExecuteStatus()
		return
	}

	if len(args) == 0 {
		core.PrintErrorWithUsage("No file indices provided", []string{
			"ga <index>...",
			"ga .",
			"ga <folder>",
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

	if gerr := gitRepo.AddFiles(paths); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	fileWord := "file"
	if len(paths) > 1 {
		fileWord = "files"
	}
	core.Template().Success(fmt.Sprintf("Successfully added %d %s to git index.", len(paths), fileWord)).Print()

	fmt.Println("Updated status:")
	files, _ := gitRepo.GetStatus()
	printGroupedStatus(files)

	newCache := &core.StateCache{
		Files:       files,
		LastUpdated: nowUnix(),
		RepoPath:    gitRepo.GetWorkDir(),
	}
	core.SaveCache(newCache, gitRepo.GetWorkDir())
}

func executeAddAll() {
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

	if gerr := core.RunGitAddAll(); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	core.Template().Success("Successfully added all files to git index.").Print()
	fmt.Println("Updated status:")
	ExecuteStatus()
}
