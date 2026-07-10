package commands

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/blitux/git-navigator/internal/core"
	"github.com/fatih/color"
)

func ExecuteUpdate(check, showVersion, yes, verbose bool) {
	if showVersion {
		fmt.Printf("\n  git-navigator v%s\n\n", color.BlueString(core.Version))
		return
	}

	fmt.Println()
	fmt.Println(color.BlueString("  Buscando actualizaciones..."))
	fmt.Println()

	release, err := core.CheckForUpdate(core.Version)
	if err != nil {
		if updateErr, ok := err.(*core.UpdateError); ok {
			core.PrintError(updateErr.Message)
		} else {
			core.PrintError("Error al buscar actualizaciones: " + err.Error())
		}
		return
	}

	needsUpdate, err := core.NeedsUpdate(core.Version, release.Version)
	if err != nil {
		core.PrintError("Error al comparar versiones: " + err.Error())
		return
	}

	body := formatVersionInfo(release, needsUpdate)

	core.Template().
		Title("Información de versión").
		Body(body).
		Print()

	if !needsUpdate {
		return
	}

	if !yes {
		fmt.Printf("%s ", color.BlueString("¿Instalar actualización? [y/N]:"))
		reader := bufio.NewReader(os.Stdin)
		input, _ := reader.ReadString('\n')
		input = strings.TrimSpace(strings.ToLower(input))
		if input != "y" && input != "yes" {
			fmt.Println()
			core.PrintInfo("Actualización cancelada")
			return
		}
	}

	fmt.Println()
	fmt.Println(color.BlueString("  Descargando actualización..."))
	fmt.Println()

	target := core.GetTarget()
	asset := core.FindAsset(release, target)
	if asset == nil {
		core.PrintError(fmt.Sprintf("No se encontró binario para %s", target))
		return
	}

	if err := core.BackupCurrent(core.Version); err != nil {
		core.PrintError("No se pudo crear backup: " + err.Error())
		return
	}

	tmpPath, err := core.DownloadAsset(asset, target)
	if err != nil {
		core.PrintError(err.Error())
		return
	}

	fmt.Println()
	fmt.Println(color.BlueString("  Aplicando actualización..."))

	if err := core.ApplyUpdate(tmpPath); err != nil {
		core.PrintError(err.Error())
		return
	}

	config, err := core.LoadOrCreateConfig()
	if err == nil {
		config.UpdateVersion(release.Version)
	}

	core.PrintSuccess(fmt.Sprintf("Actualizado a v%s", release.Version))
}

func formatVersionInfo(release *core.ReleaseInfo, needsUpdate bool) string {
	status := color.GreenString("Actualizado")
	if needsUpdate {
		status = color.YellowString("Actualización disponible")
	}

	info := fmt.Sprintf("   Actual: %s\n   Última: %s\n   Estado: %s",
		color.BlueString("v"+core.Version),
		color.BlueString("v"+release.Version),
		status)

	if needsUpdate && release.Body != "" {
		notes := core.FormatReleaseNotes(release.Body, 5)
		if notes != "" {
			info += "\n\n   Novedades:\n"
			for _, line := range strings.Split(notes, "\n") {
				info += fmt.Sprintf("   • %s\n", color.WhiteString(line))
			}
		}
	}

	return info
}
