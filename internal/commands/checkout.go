package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
)

func ExecuteCheckout(args []string, createBranch bool) {
	if createBranch {
		if len(args) == 0 {
			core.PrintErrorWithUsage("Branch name required with -b flag", []string{
				"gco -b <branch-name>",
			})
			return
		}
		if len(args) > 1 {
			core.PrintErrorWithUsage("Only one branch name allowed with -b flag", []string{
				"gco -b <branch-name>",
			})
			return
		}
		createAndCheckoutBranch(args[0])
		return
	}

	if len(args) == 0 {
		core.PrintErrorWithUsage("No file indices or branch name provided", []string{
			"gco <index>...",
			"gco <branch>",
			"gco -b <branch-name>",
		})
		return
	}

	if len(args) == 1 && !core.IsNumericIndex(args[0]) {
		checkoutBranchByName(args[0])
		return
	}

	checkoutFilesByIndices(args)
}

func checkoutFilesByIndices(args []string) {
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
		core.PrintError("There are no changes to checkout")
		ExecuteStatus()
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

	if gerr := gitRepo.CheckoutFiles(paths); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	fileWord := "file"
	if len(paths) > 1 {
		fileWord = "files"
	}
	core.Template().Success(fmt.Sprintf("Successfully checked out %d %s.", len(paths), fileWord)).Print()

	fmt.Println("Updated status:")
	files, _ := gitRepo.GetStatus()
	printGroupedStatus(files)

	newCache := &core.StateCache{
		Files:       files,
		Branches:    []core.BranchEntry{},
		LastUpdated: nowUnix(),
		RepoPath:    gitRepo.GetWorkDir(),
	}
	core.SaveCache(newCache, gitRepo.GetWorkDir())
}

func checkoutBranchByName(name string) {
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

	if gerr := gitRepo.CheckoutBranch(name); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	core.Template().Success(fmt.Sprintf("Successfully switched to branch '%s'", name)).Print()
}

func createAndCheckoutBranch(name string) {
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

	if gerr := gitRepo.CreateBranch(name); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	core.Template().Success(fmt.Sprintf("Successfully created and switched to branch '%s'", name)).Print()
}
