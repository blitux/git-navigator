# Git Navigator - Migración Rust → Go

## Estado de la Migración

### Implementado ✅

#### Comandos
- [x] `status` (gs) - Muestra status numerado agrupado por staged/unstaged/untracked
- [x] `add` (ga) - Añade archivos por índice
- [x] `diff` (gd) - Muestra diff de archivos indexados
- [x] `reset` (grs) - Resetea archivos por índice
- [x] `checkout` (gco) - Checkout files y branches (`-b` para crear)
- [x] `branches` (gb) - Lista branches numeradas

#### Core
- [x] GitStatus enum (Modified, Added, Deleted, Renamed, Copied, TypeChanged, Untracked, Unmerged)
- [x] FileEntry / BranchEntry structs
- [x] Index parsing flexible (`1`, `1-3`, `1,3,5`, `1 3-5,8`)
- [x] State cache en JSON (`$XDG_CACHE_HOME/git-navigator/<hash>/files.json`)
- [x] Domain-specific errors
- [x] Output con colores (fatih/color)
- [x] Template system básico

---

### Faltante ❌

#### Comandos
- [ ] `copy` (cp) - Wrapper de `cp` con resolución de índices
- [ ] `remove` (rm) - Wrapper de `rm` con resolución de índices
- [ ] `alias` - Mostrar/actualizar aliases shell (bash/zsh/fish)
- [ ] `update` - Auto-update desde GitHub releases
- [ ] `rollback` - Restaurar versiones anteriores

#### Features
- [ ] AliasManager - Detección de shell, lectura/escritura de config de aliases
- [ ] Branch cache (`branches.json`) para `gb <index>` checkout
- [ ] Install script (`install.sh`)
- [ ] Auto-detección de plataforma para releases
- [ ] Tests unitarios

---

## Estructura Actual (Go)

```
cmd/git-navigator/main.go      # Entry point con cobra
internal/
├── core/
│   ├── state.go          # GitStatus, FileEntry, BranchEntry, StateCache
│   ├── error.go          # Errores domain-specific
│   ├── index_parser.go   # Parsing de índices flexible
│   ├── output.go         # print_error, print_success, Template
│   └── git.go           # Operaciones git via exec
└── commands/
    ├── status.go
    ├── add.go
    ├── diff.go
    ├── reset.go
    ├── checkout.go
    └── branches.go
```

---

## Dependencias Go

```
github.com/fatih/color v1.16.0
github.com/spf13/cobra v1.8.0
```

---

## Shell Aliases a Implementar

```bash
alias gs='git-navigator status'
alias ga='git-navigator add'
alias gd='git-navigator diff'
alias grs='git-navigator reset'
alias gco='git-navigator checkout'
alias gb='git-navigator branches'
alias gcb='git-navigator checkout-branch'
alias gl="git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
```
