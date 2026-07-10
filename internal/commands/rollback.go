package commands

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteRollback(list bool, version string) {
	if list {
		listBackups()
		return
	}

	if version != "" {
		restoreVersion(version)
		return
	}

	interactiveRollback()
}

func listBackups() {
	backups, err := core.ListBackups()
	if err != nil {
		core.PrintError(err.Error())
		return
	}

	if len(backups) == 0 {
		core.PrintInfo("No hay backups disponibles")
		return
	}

	body := core.FormatBackupsList(backups)
	core.Template().
		Title("Backups disponibles").
		Body(body).
		Print()
}

func restoreVersion(version string) {
	fmt.Println()
	fmt.Printf("  Restaurando git-navigator v%s...\n", color.BlueString(version))
	fmt.Println()

	if err := core.RestoreFromBackup(version); err != nil {
		core.PrintError(err.Error())
		return
	}

	core.PrintSuccess(fmt.Sprintf("Restaurado a v%s", version))
}

func interactiveRollback() {
	backups, err := core.ListBackups()
	if err != nil {
		core.PrintError(err.Error())
		return
	}

	if len(backups) == 0 {
		core.PrintError("No hay backups disponibles")
		return
	}

	body := core.FormatBackupsList(backups)
	core.Template().
		Title("Selecciona versión a restaurar").
		Body(body).
		Print()

	fmt.Printf("\n%s ", color.BlueString("Ingresa selección (1-"+strconv.Itoa(len(backups))+"):"))
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	input = strings.TrimSpace(input)

	idx, err := strconv.Atoi(input)
	if err != nil || idx < 1 || idx > len(backups) {
		core.PrintError("Selección inválida")
		return
	}

	selected := backups[idx-1]
	restoreVersion(selected.Version)
}
