package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
)

func getRepo() *core.GitRepo {
	cwd, err := os.Getwd()
	if err != nil {
		core.PrintError("Could not determine current directory")
		return nil
	}

	gitRepo, gerr := core.OpenGitRepo(cwd)
	if gerr != nil {
		core.PrintError(gerr.Message)
		return nil
	}

	return gitRepo
}

func getRepoAndCache() (*core.GitRepo, *core.StateCache) {
	cwd, err := os.Getwd()
	if err != nil {
		core.PrintError("Could not determine current directory")
		return nil, nil
	}

	gitRepo, gerr := core.OpenGitRepo(cwd)
	if gerr != nil {
		core.PrintError(gerr.Message)
		return nil, nil
	}

	cache, gerr := core.LoadCache(cwd)
	if gerr != nil {
		core.PrintError(fmt.Sprintf("Cannot load file cache: %s. Run 'gs' first.", gerr.Message))
		return nil, nil
	}

	return gitRepo, cache
}
