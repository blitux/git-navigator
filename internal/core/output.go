package core

import (
	"fmt"

	"github.com/fatih/color"
)

func PrintError(msg string) {
	fmt.Printf("\n%s %s\n\n", color.RedString("✕ Error:"), msg)
}

func PrintSuccess(msg string) {
	fmt.Printf("\n%s %s\n\n", color.GreenString("✓"), msg)
}

func PrintInfo(msg string) {
	fmt.Printf("\n%s\n\n", msg)
}

func PrintSectionHeader(header string) {
	fmt.Printf("\n%s:\n\n", header)
}

type CommandTemplate struct {
	TitleValue   string
	BodyValue    string
	HelpValue    string
	WarningValue string
	SuccessValue string
}

func (t CommandTemplate) String() string {
	result := "\n"
	if t.TitleValue != "" {
		result += color.BlueString(t.TitleValue) + "\n\n"
	}
	if t.BodyValue != "" {
		result += t.BodyValue + "\n"
	}
	if t.WarningValue != "" {
		result += color.YellowString(t.WarningValue) + "\n"
	}
	if t.SuccessValue != "" {
		result += color.GreenString(t.SuccessValue) + "\n"
	}
	if t.HelpValue != "" {
		result += color.BlackString(t.HelpValue) + "\n"
	}
	result += "\n"
	return result
}

func (t CommandTemplate) Print() {
	fmt.Print(t.String())
}

func Template() *CommandTemplate {
	return &CommandTemplate{}
}

func (t *CommandTemplate) Title(s string) *CommandTemplate {
	t.TitleValue = s
	return t
}

func (t *CommandTemplate) Body(s string) *CommandTemplate {
	t.BodyValue = s
	return t
}

func (t *CommandTemplate) Help(s string) *CommandTemplate {
	t.HelpValue = s
	return t
}

func (t *CommandTemplate) Warning(s string) *CommandTemplate {
	t.WarningValue = s
	return t
}

func (t *CommandTemplate) Success(s string) *CommandTemplate {
	t.SuccessValue = s
	return t
}

func PrintErrorWithUsage(msg string, usage []string) {
	fmt.Printf("\n%s %s.\n\n", color.RedString("✕ Error:"), msg)
	fmt.Printf("%s\n", color.BlueString("Usage:"))
	for _, u := range usage {
		fmt.Printf("  %s\n", u)
	}
	fmt.Println()
}
