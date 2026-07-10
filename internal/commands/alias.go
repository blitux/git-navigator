package commands

import (
	"fmt"
	"runtime"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteAlias(update bool) {
	manager, err := core.NewAliasManager()
	if err != nil {
		core.PrintError("No se pudo inicializar el gestor de aliases: " + err.Error())
		return
	}

	if update {
		updateAliases(manager)
	} else {
		showAliases(manager)
	}
}

func showAliases(manager *core.AliasManager) {
	comparisons := manager.Compare()

	fmt.Println()
	fmt.Printf("  %s (%s)\n", color.BlueString("Git Navigator Aliases"), manager.Shell().String())
	fmt.Println("  " + repeatDash(50))
	fmt.Printf("  %-10s %-10s %-30s %s\n",
		color.BlueString("Status"),
		color.BlueString("Alias"),
		color.BlueString("Command"),
		color.BlueString("Description"))
	fmt.Println("  " + repeatDash(50))

	installed := 0
	missing := 0
	outdated := 0

	for _, comp := range comparisons {
		statusIcon := color.RedString("✗")

		if comp.Status.Installed {
			statusIcon = color.GreenString("✓")
			installed++
		} else if comp.Status.Outdated {
			statusIcon = color.YellowString("⚠")
			outdated++
		} else {
			missing++
		}

		cmdDisplay := comp.Alias.Command
		if len(cmdDisplay) > 28 {
			cmdDisplay = cmdDisplay[:25] + "..."
		}

		fmt.Printf("  %-10s %-10s %-30s %s\n",
			statusIcon,
			color.CyanString(comp.Alias.Name),
			cmdDisplay,
			comp.Alias.Description)

		if comp.Status.Outdated {
			fmt.Printf("           %s  %s\n",
				color.BlackString("current:"),
				color.BlackString(comp.Status.Current))
		}
	}

	fmt.Println("  " + repeatDash(50))
	fmt.Println()
	fmt.Printf("  %s: %s\n",
		color.BlackString("Config file"),
		color.BlackString(manager.ConfigPath()))
	fmt.Printf("  %s: %s installed, %s missing, %s outdated\n",
		color.BlackString("Summary"),
		color.GreenString(fmt.Sprintf("%d", installed)),
		color.RedString(fmt.Sprintf("%d", missing)),
		color.YellowString(fmt.Sprintf("%d", outdated)))

	if missing > 0 || outdated > 0 {
		fmt.Println()
		core.PrintInfo("Ejecuta 'git-navigator alias --update' para instalar/actualizar aliases")
	}
	fmt.Println()
}

func updateAliases(manager *core.AliasManager) {
	fmt.Println()
	fmt.Printf("  Actualizando aliases en %s...\n", color.BlueString(manager.ConfigPath()))
	fmt.Println()

	before := manager.Compare()
	added := 0
	updated := 0
	unchanged := 0

	for _, comp := range before {
		symbol := color.RedString("+")
		text := "added"

		if comp.Status.Installed {
			symbol = color.BlackString("=")
			text = "unchanged"
			unchanged++
		} else if comp.Status.Outdated {
			symbol = color.YellowString("~")
			text = "updated"
			updated++
		} else {
			added++
		}

		fmt.Printf("  %s %-8s %s (%s)\n",
			symbol,
			comp.Alias.Name,
			comp.Alias.Command,
			text)
	}

	fmt.Println()

	if _, err := manager.Update(); err != nil {
		core.PrintError("Error al actualizar aliases: " + err.Error())
		return
	}

	if added > 0 || updated > 0 {
		core.PrintSuccess(fmt.Sprintf("%d aliases agregados/actualizados en %s",
			added+updated, manager.ConfigPath()))

		shell := runtime.GOOS
		var reloadCmd string
		switch shell {
		case "linux", "freebsd", "netbsd", "openbsd":
			if manager.Shell().String() == "zsh" {
				reloadCmd = "source ~/.zshrc"
			} else {
				reloadCmd = "source ~/.bashrc"
			}
		case "darwin":
			reloadCmd = "source ~/.bashrc"
		case "windows":
			reloadCmd = "reinicia tu terminal"
		default:
			reloadCmd = "reinicia tu terminal"
		}

		core.PrintInfo("Por favor reinicia tu terminal o ejecuta: " + color.BlueString(reloadCmd))
	} else {
		core.PrintSuccess("Todos los aliases ya están actualizados")
	}
}

func repeatDash(n int) string {
	result := ""
	for i := 0; i < n; i++ {
		result += "─"
	}
	return result
}
