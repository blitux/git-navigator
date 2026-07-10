package core

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

type Shell int

const (
	ShellBash Shell = iota
	ShellZsh
	ShellFish
	ShellSh
)

func (s Shell) String() string {
	switch s {
	case ShellBash:
		return "bash"
	case ShellZsh:
		return "zsh"
	case ShellFish:
		return "fish"
	case ShellSh:
		return "sh"
	default:
		return "unknown"
	}
}

func DetectShell() (Shell, error) {
	shellPath := os.Getenv("SHELL")
	if shellPath == "" {
		return ShellBash, nil
	}

	if strings.Contains(shellPath, "bash") {
		return ShellBash, nil
	}
	if strings.Contains(shellPath, "zsh") {
		return ShellZsh, nil
	}
	if strings.Contains(shellPath, "fish") {
		return ShellFish, nil
	}
	if strings.Contains(shellPath, "sh") {
		return ShellSh, nil
	}

	return ShellBash, nil
}

func (s Shell) ConfigPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}

	switch s {
	case ShellBash:
		if runtime.GOOS == "darwin" {
			bashProfile := filepath.Join(home, ".bash_profile")
			if _, err := os.Stat(bashProfile); err == nil {
				return bashProfile, nil
			}
		}
		return filepath.Join(home, ".bashrc"), nil
	case ShellZsh:
		return filepath.Join(home, ".zshrc"), nil
	case ShellFish:
		return filepath.Join(home, ".config", "fish", "config.fish"), nil
	case ShellSh:
		return filepath.Join(home, ".profile"), nil
	default:
		return filepath.Join(home, ".bashrc"), nil
	}
}

func (s Shell) FormatAlias(name, command string) string {
	switch s {
	case ShellFish:
		return "alias " + name + " \"" + command + "\""
	default:
		return "alias " + name + "=\"" + command + "\""
	}
}

func (s Shell) ParseAlias(line string) (string, string, bool) {
	line = strings.TrimSpace(line)
	if !strings.HasPrefix(line, "alias ") {
		return "", "", false
	}

	rest := strings.TrimPrefix(line, "alias ")
	rest = strings.TrimSpace(rest)

	switch s {
	case ShellFish:
		parts := strings.SplitN(rest, " ", 2)
		if len(parts) != 2 {
			return "", "", false
		}
		name := parts[0]
		command := strings.Trim(parts[1], "\"")
		return name, command, true
	default:
		parts := strings.SplitN(rest, "=", 2)
		if len(parts) != 2 {
			return "", "", false
		}
		name := parts[0]
		command := strings.Trim(parts[1], "\"")
		return name, command, true
	}
}

const AliasMarker = "# Git Navigator aliases"

type AliasDefinition struct {
	Name        string
	Command     string
	Description string
}

type AliasStatus struct {
	Installed bool
	Outdated  bool
	Current   string
}

type AliasComparison struct {
	Alias  AliasDefinition
	Status AliasStatus
}

type AliasRegistry struct{}

func (AliasRegistry) Definitions() []AliasDefinition {
	return []AliasDefinition{
		{Name: "gs", Command: "git-navigator status", Description: "Show numbered git status"},
		{Name: "ga", Command: "git-navigator add", Description: "Add files by index"},
		{Name: "gd", Command: "git-navigator diff", Description: "Show diff by index"},
		{Name: "grs", Command: "git-navigator reset", Description: "Reset files by index"},
		{Name: "gco", Command: "git-navigator checkout", Description: "Checkout files by index"},
		{Name: "gb", Command: "git-navigator branches", Description: "Show numbered branches"},
	}
}

func (AliasRegistry) DefinitionsMap() map[string]AliasDefinition {
	defs := AliasRegistry{}.Definitions()
	result := make(map[string]AliasDefinition)
	for _, d := range defs {
		result[d.Name] = d
	}
	return result
}

type AliasManager struct {
	shell       Shell
	configPath  string
	allAliases  map[string]string
	currentDefs map[string]AliasDefinition
}

func NewAliasManager() (*AliasManager, error) {
	shell, err := DetectShell()
	if err != nil {
		return nil, err
	}
	configPath, err := shell.ConfigPath()
	if err != nil {
		return nil, err
	}

	manager := &AliasManager{
		shell:       shell,
		configPath:   configPath,
		allAliases:  make(map[string]string),
		currentDefs: AliasRegistry{}.DefinitionsMap(),
	}

	if _, err := os.Stat(configPath); err == nil {
		manager.readAliases()
	}

	return manager, nil
}

func (m *AliasManager) readAliases() {
	content, err := os.ReadFile(m.configPath)
	if err != nil {
		return
	}

	inBlock := false
	for _, line := range strings.Split(string(content), "\n") {
		line = strings.TrimSpace(line)

		if strings.HasPrefix(line, AliasMarker) {
			inBlock = true
			continue
		}

		if inBlock {
			if line == "" || (!strings.HasPrefix(line, "alias ") && !strings.HasPrefix(line, "#")) {
				break
			}
			if strings.HasPrefix(line, "#") {
				continue
			}

			if name, cmd, ok := m.shell.ParseAlias(line); ok {
				m.allAliases[name] = cmd
			}
		}
	}
}

func (m *AliasManager) Compare() []AliasComparison {
	var comparisons []AliasComparison

	for _, def := range m.currentDefs {
		status := AliasStatus{Installed: false, Outdated: false}
		if cmd, exists := m.allAliases[def.Name]; exists {
			if cmd == def.Command {
				status.Installed = true
			} else {
				status.Outdated = true
				status.Current = cmd
			}
		}
		comparisons = append(comparisons, AliasComparison{
			Alias:  def,
			Status: status,
		})
	}

	return comparisons
}

func (m *AliasManager) Update() ([]AliasComparison, error) {
	comparisons := m.Compare()

	var aliasLines []string
	aliasLines = append(aliasLines, AliasMarker)
	for _, def := range m.currentDefs {
		aliasLines = append(aliasLines, m.shell.FormatAlias(def.Name, def.Command))
	}

	content := ""
	if data, err := os.ReadFile(m.configPath); err == nil {
		content = string(data)
	}

	newContent := m.replaceOrAppendBlock(content, aliasLines)

	if err := os.WriteFile(m.configPath, []byte(newContent), 0644); err != nil {
		return nil, err
	}

	return comparisons, nil
}

func (m *AliasManager) replaceOrAppendBlock(content string, newLines []string) string {
	if strings.Contains(content, AliasMarker) {
		return m.replaceBlock(content, newLines)
	}
	return m.appendBlock(content, newLines)
}

func (m *AliasManager) replaceBlock(content string, newLines []string) string {
	var result []string
	inBlock := false
	blockReplaced := false

	for _, line := range strings.Split(content, "\n") {
		if strings.HasPrefix(strings.TrimSpace(line), AliasMarker) {
			if !blockReplaced {
				result = append(result, newLines...)
				blockReplaced = true
			}
			inBlock = true
			continue
		}

		if inBlock {
			if strings.TrimSpace(line) == "" {
				inBlock = false
				result = append(result, line)
				continue
			}
			if strings.HasPrefix(strings.TrimSpace(line), "alias ") {
				continue
			}
			inBlock = false
		}

		result = append(result, line)
	}

	return strings.Join(result, "\n") + "\n"
}

func (m *AliasManager) appendBlock(content string, newLines []string) string {
	if content != "" && !strings.HasSuffix(content, "\n") {
		content += "\n"
	}
	if content != "" {
		content += "\n"
	}
	for _, line := range newLines {
		content += line + "\n"
	}
	return content
}

func (m *AliasManager) Shell() Shell {
	return m.shell
}

func (m *AliasManager) ConfigPath() string {
	return m.configPath
}
