package commands

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteBranches(showRemote bool, index *int) {
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

	if index != nil {
		checkoutBranchByIndex(gitRepo, *index)
		return
	}

	listBranches(gitRepo, showRemote)
}

func listBranches(gitRepo *core.GitRepo, showRemote bool) {
	var branches []core.BranchEntry
	var gerr *core.GitNavigatorError

	if showRemote {
		branches, gerr = gitRepo.GetRemoteBranches()
	} else {
		branches, gerr = gitRepo.GetLocalBranches()
	}

	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	if len(branches) == 0 {
		if showRemote {
			core.PrintInfo("No remote branches found. Run `git fetch` to update remotes.")
		} else {
			core.PrintInfo("No branches found. Make your first commit to create one.")
		}
		return
	}

	title := "Local Branches"
	if showRemote {
		title = "Remote Branches"
	}

	helpText := "Use gb <index> to checkout or gb without --remote for local branches."
	if showRemote {
		helpText = "Use gb --remote to list remote branches."
	}

	body := ""
	for _, b := range branches {
		body += fmt.Sprintf("  %s%s%s  %s\n",
			color.BlackString("["),
			color.WhiteString(b.Index),
			color.BlackString("]"),
			color.BlueString(b.Name))
	}

	core.Template().
		Title(title).
		Body(body).
		Help(helpText).
		Print()

	saveBranchesCache(branches, gitRepo.GetWorkDir())
}

func checkoutBranchByIndex(gitRepo *core.GitRepo, index int) {
	branches, gerr := gitRepo.GetLocalBranches()
	if gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	var targetBranch string
	found := false

	for _, b := range branches {
		if b.IsCurrent {
			continue
		}
		var idx int
		fmt.Sscanf(b.Index, "%d", &idx)
		if idx == index {
			targetBranch = b.Name
			found = true
			break
		}
	}

	if !found {
		core.PrintError(fmt.Sprintf("Branch index %d not found", index))
		return
	}

	if gerr := gitRepo.CheckoutBranch(targetBranch); gerr != nil {
		core.PrintError(gerr.Message)
		return
	}

	fmt.Printf("Switched to branch '%s'\n", targetBranch)
}

func saveBranchesCache(branches []core.BranchEntry, repoPath string) {
}
