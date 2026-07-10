package main

import (
	"fmt"
	"os"

	"github.com/blitux/git-navigator/internal/commands"
	"github.com/spf13/cobra"
)

var debug bool

func main() {
	rootCmd := &cobra.Command{
		Use:   "git-navigator",
		Short: "A lightweight and efficient Git navigation tool",
		Long: `Git Navigator - A modern reimagining of SCM Breeze's core workflow

Commands:
  gs    Show numbered git status
  ga    Add files by index
  gd    Show diff for indexed files
  grs   Reset files by index
  gco   Checkout files or branches
  gb    List numbered branches`,
	}

	rootCmd.PersistentFlags().BoolVar(&debug, "debug", false, "Enable debug logging")

	statusCmd := &cobra.Command{
		Use:   "status",
		Short: "Show numbered git status (gs alias)",
		Run: func(cmd *cobra.Command, args []string) {
			commands.ExecuteStatus()
		},
	}

	addCmd := &cobra.Command{
		Use:   "add [indices]",
		Short: "Add files by index (ga alias)",
		Args:  cobra.MinimumNArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			commands.ExecuteAdd(args)
		},
	}

	diffCmd := &cobra.Command{
		Use:   "diff [indices]",
		Short: "Show diff for files by index (gd alias)",
		Args:  cobra.MinimumNArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			commands.ExecuteDiff(args)
		},
	}

	resetCmd := &cobra.Command{
		Use:   "reset [indices]",
		Short: "Reset files by index (grs alias)",
		Args:  cobra.MinimumNArgs(0),
		Run: func(cmd *cobra.Command, args []string) {
			commands.ExecuteReset(args)
		},
	}

	checkoutCmd := &cobra.Command{
		Use:   "checkout [indices]",
		Short: "Checkout files by index or switch to branch (gco alias)",
		Run: func(cmd *cobra.Command, args []string) {
			createBranch, _ := cmd.Flags().GetBool("create")
			commands.ExecuteCheckout(args, createBranch)
		},
	}
	checkoutCmd.Flags().BoolP("create", "b", false, "Create and switch to a new branch")

	branchesCmd := &cobra.Command{
		Use:   "branches",
		Short: "Show numbered branches or switch to a branch (gb alias)",
		Run: func(cmd *cobra.Command, args []string) {
			showRemote, _ := cmd.Flags().GetBool("remote")
			var index *int
			if len(args) > 0 {
				var idx int
				fmt.Sscanf(args[0], "%d", &idx)
				index = &idx
			}
			commands.ExecuteBranches(showRemote, index)
		},
	}
	branchesCmd.Flags().BoolP("remote", "r", false, "Show remote branches instead of local")

	updateCmd := &cobra.Command{
		Use:   "update",
		Short: "Check for updates or update to the latest version",
		Run: func(cmd *cobra.Command, args []string) {
			check, _ := cmd.Flags().GetBool("check")
			version, _ := cmd.Flags().GetBool("version")
			yes, _ := cmd.Flags().GetBool("yes")
			verbose, _ := cmd.Flags().GetBool("verbose")
			commands.ExecuteUpdate(check, version, yes, verbose)
		},
	}
	updateCmd.Flags().Bool("check", false, "Check for updates without installing")
	updateCmd.Flags().Bool("version", false, "Show current version")
	updateCmd.Flags().Bool("yes", false, "Skip confirmation prompt")
	updateCmd.Flags().Bool("verbose", false, "Show verbose output")

	rootCmd.AddCommand(statusCmd)
	rootCmd.AddCommand(addCmd)
	rootCmd.AddCommand(diffCmd)
	rootCmd.AddCommand(resetCmd)
	rootCmd.AddCommand(checkoutCmd)
	rootCmd.AddCommand(branchesCmd)
	rootCmd.AddCommand(updateCmd)

	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
