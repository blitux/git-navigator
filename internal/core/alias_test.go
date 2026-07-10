package core

import (
	"os"
	"path/filepath"
	"testing"
)

func TestShellString(t *testing.T) {
	tests := []struct {
		shell   Shell
		want    string
	}{
		{ShellBash, "bash"},
		{ShellZsh, "zsh"},
		{ShellFish, "fish"},
		{ShellSh, "sh"},
		{Shell(99), "unknown"},
	}

	for _, tt := range tests {
		if got := tt.shell.String(); got != tt.want {
			t.Errorf("Shell.String() = %v, want %v", got, tt.want)
		}
	}
}

func TestShellFormatAlias(t *testing.T) {
	tests := []struct {
		shell   Shell
		name    string
		command string
		want    string
	}{
		{ShellBash, "gs", "git-navigator status", `alias gs="git-navigator status"`},
		{ShellZsh, "gs", "git-navigator status", `alias gs="git-navigator status"`},
		{ShellFish, "gs", "git-navigator status", `alias gs "git-navigator status"`},
		{ShellSh, "gs", "git-navigator status", `alias gs="git-navigator status"`},
	}

	for _, tt := range tests {
		got := tt.shell.FormatAlias(tt.name, tt.command)
		if got != tt.want {
			t.Errorf("Shell(%v).FormatAlias() = %v, want %v", tt.shell, got, tt.want)
		}
	}
}

func TestShellParseAlias(t *testing.T) {
	tests := []struct {
		name     string
		shell    Shell
		line     string
		wantName string
		wantCmd  string
		wantOk   bool
	}{
		{
			name:     "bash simple",
			shell:    ShellBash,
			line:     `alias gs="git-navigator status"`,
			wantName: "gs",
			wantCmd:  "git-navigator status",
			wantOk:   true,
		},
		{
			name:     "zsh simple",
			shell:    ShellZsh,
			line:     `alias gs="git-navigator status"`,
			wantName: "gs",
			wantCmd:  "git-navigator status",
			wantOk:   true,
		},
		{
			name:     "fish simple",
			shell:    ShellFish,
			line:     `alias gs "git-navigator status"`,
			wantName: "gs",
			wantCmd:  "git-navigator status",
			wantOk:   true,
		},
		{
			name:     "bash with single quotes",
			shell:    ShellBash,
			line:     `alias gs='git-navigator status'`,
			wantName: "gs",
			wantCmd:  "git-navigator status",
			wantOk:   true,
		},
		{
			name:     "not an alias line",
			shell:    ShellBash,
			line:     `git status`,
			wantName: "",
			wantCmd:  "",
			wantOk:   false,
		},
		{
			name:     "empty line",
			shell:    ShellBash,
			line:     ``,
			wantName: "",
			wantCmd:  "",
			wantOk:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotName, gotCmd, gotOk := tt.shell.ParseAlias(tt.line)
			if gotName != tt.wantName || gotCmd != tt.wantCmd || gotOk != tt.wantOk {
				t.Errorf("Shell(%v).ParseAlias(%q) = (%q, %q, %v), want (%q, %q, %v)",
					tt.shell, tt.line, gotName, gotCmd, gotOk, tt.wantName, tt.wantCmd, tt.wantOk)
			}
		})
	}
}

func TestAliasRegistryDefinitions(t *testing.T) {
	defs := AliasRegistry{}.Definitions()

	if len(defs) == 0 {
		t.Error("AliasRegistry.Definitions() returned empty slice")
	}

	expected := map[string]string{
		"gs":  "git-navigator status",
		"ga":  "git-navigator add",
		"gd":  "git-navigator diff",
		"grs": "git-navigator reset",
		"gco": "git-navigator checkout",
		"gb":  "git-navigator branches",
	}

	for name, cmd := range expected {
		found := false
		for _, d := range defs {
			if d.Name == name {
				found = true
				if d.Command != cmd {
					t.Errorf("Alias %q has command %q, want %q", name, d.Command, cmd)
				}
			}
		}
		if !found {
			t.Errorf("Alias %q not found in registry", name)
		}
	}
}

func TestAliasRegistryDefinitionsMap(t *testing.T) {
	m := AliasRegistry{}.DefinitionsMap()

	expected := []string{"gs", "ga", "gd", "grs", "gco", "gb", "gl"}
	for _, name := range expected {
		if _, ok := m[name]; !ok {
			t.Errorf("Alias %q not found in DefinitionsMap", name)
		}
	}

	if len(m) != len(expected) {
		t.Errorf("DefinitionsMap has %d entries, want %d", len(m), len(expected))
	}
}

func TestAliasManagerCompare(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "testrc")

	manager := NewAliasManagerForTest(ShellBash, configPath)
	comparisons := manager.Compare()

	if len(comparisons) == 0 {
		t.Error("Compare() returned empty comparisons")
	}

	for _, comp := range comparisons {
		if comp.Status.Installed {
			t.Errorf("Expected no aliases installed in empty file, got %q", comp.Alias.Name)
		}
		if comp.Status.Outdated {
			t.Error("Expected no aliases outdated in empty file")
		}
	}
}

func TestAliasManagerCompareWithInstalledAliases(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "testrc")

	content := `# Git Navigator aliases
alias gs="git-navigator status"
alias ga="git-navigator add"
alias gd="WRONG COMMAND"
alias gb="git-navigator branches"
`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	manager := NewAliasManagerForTest(ShellBash, configPath)
	manager.readAliases()
	comparisons := manager.Compare()

	for _, comp := range comparisons {
		switch comp.Alias.Name {
		case "gs":
			if !comp.Status.Installed {
				t.Error("gs should be installed")
			}
		case "ga":
			if !comp.Status.Installed {
				t.Error("ga should be installed")
			}
		case "gd":
			if !comp.Status.Outdated {
				t.Error("gd should be outdated (wrong command)")
			}
			if comp.Status.Current != "WRONG COMMAND" {
				t.Errorf("gd current should be 'WRONG COMMAND', got %q", comp.Status.Current)
			}
		case "grs", "gco":
			if !comp.Status.Missing {
				t.Errorf("%s should be missing", comp.Alias.Name)
			}
		case "gl":
			if !comp.Status.Missing {
				t.Errorf("gl should be missing")
			}
		}
	}
}

func TestAliasManagerUpdateReplacesBlock(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "testrc")

	oldContent := `# Some existing config
export PATH=$HOME/bin:$PATH

# Git Navigator aliases
alias gs="git-navigator status"
alias gcb="git-navigator checkout-branch"
alias gl="git log --oneline"

# More config
export EDITOR=vim
`
	if err := os.WriteFile(configPath, []byte(oldContent), 0644); err != nil {
		t.Fatal(err)
	}

	manager := NewAliasManagerForTest(ShellBash, configPath)
	manager.readAliases()

	if _, err := manager.Update(); err != nil {
		t.Fatal(err)
	}

	result, err := os.ReadFile(configPath)
	if err != nil {
		t.Fatal(err)
	}

	resultStr := string(result)

	if !aliasContainsString(resultStr, "# Git Navigator aliases") {
		t.Error("Result should contain AliasMarker")
	}

	if !containsString(resultStr, `alias gs="git-navigator status"`) {
		t.Error("Result should contain gs alias")
	}

	if !containsString(resultStr, `alias gco="git-navigator checkout"`) {
		t.Error("Result should contain gco alias")
	}

	if !containsString(resultStr, "export PATH=$HOME/bin:$PATH") {
		t.Error("Result should preserve PATH export")
	}

	if !containsString(resultStr, "export EDITOR=vim") {
		t.Error("Result should preserve EDITOR export")
	}

	if containsString(resultStr, "gcb") {
		t.Error("Result should NOT contain old gcb alias")
	}

	if containsString(resultStr, "git log --oneline") {
		t.Error("Result should NOT contain old gl alias")
	}
}

func TestAliasManagerUpdateAppendsBlock(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "testrc")

	oldContent := `# Some config without git-navigator aliases
export PATH=$HOME/bin:$PATH
`
	if err := os.WriteFile(configPath, []byte(oldContent), 0644); err != nil {
		t.Fatal(err)
	}

	manager := NewAliasManagerForTest(ShellBash, configPath)
	manager.readAliases()

	if _, err := manager.Update(); err != nil {
		t.Fatal(err)
	}

	result, err := os.ReadFile(configPath)
	if err != nil {
		t.Fatal(err)
	}

	resultStr := string(result)

	if !aliasContainsString(resultStr, "# Git Navigator aliases") {
		t.Error("Result should contain AliasMarker")
	}

	if !containsString(resultStr, `alias gs="git-navigator status"`) {
		t.Error("Result should contain gs alias")
	}

	if !aliasContainsString(resultStr, "export PATH=$HOME/bin:$PATH") {
		t.Error("Result should preserve PATH export before aliases")
	}
}

func TestAliasManagerUpdateWithFishShell(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.fish")

	oldContent := `# Fish config
set -g PATH $HOME/bin $PATH

# Git Navigator aliases
alias gs "git-navigator status"
alias gcb "git-navigator checkout-branch"
`
	if err := os.WriteFile(configPath, []byte(oldContent), 0644); err != nil {
		t.Fatal(err)
	}

	manager := NewAliasManagerForTest(ShellFish, configPath)
	manager.readAliases()

	if _, err := manager.Update(); err != nil {
		t.Fatal(err)
	}

	result, err := os.ReadFile(configPath)
	if err != nil {
		t.Fatal(err)
	}

	resultStr := string(result)

	if !containsString(resultStr, `alias gs "git-navigator status"`) {
		t.Error("Result should contain fish-format gs alias")
	}

	if !containsString(resultStr, `alias ga "git-navigator add"`) {
		t.Error("Result should contain fish-format ga alias")
	}

	if containsString(resultStr, "gcb") {
		t.Error("Result should NOT contain old gcb alias")
	}
}

func TestAliasManagerUpdateWithZshShell(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, ".zshrc")

	oldContent := `# Zsh config
export PATH=$HOME/bin:$PATH

# Git Navigator aliases
alias gs="git-navigator status"
alias rm="git rm"
`
	if err := os.WriteFile(configPath, []byte(oldContent), 0644); err != nil {
		t.Fatal(err)
	}

	manager := NewAliasManagerForTest(ShellZsh, configPath)
	manager.readAliases()

	if _, err := manager.Update(); err != nil {
		t.Fatal(err)
	}

	result, err := os.ReadFile(configPath)
	if err != nil {
		t.Fatal(err)
	}

	resultStr := string(result)

	if !containsString(resultStr, `alias gs="git-navigator status"`) {
		t.Error("Result should contain gs alias")
	}

	if !containsString(resultStr, `alias grs="git-navigator reset"`) {
		t.Error("Result should contain grs alias")
	}

	if containsString(resultStr, `alias rm=`) {
		t.Error("Result should NOT contain old rm alias")
	}
}

func aliasContainsString(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && aliasContainsSubstring(s, substr))
}

func aliasContainsSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
