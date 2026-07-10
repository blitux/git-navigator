# CI/CD Migration: Rust → Go

## Estado Actual

### GitHub Actions (release.yml)

Workflow existente en `.github/workflows/release.yml` está diseñado para Rust con `cross` toolchain.

```yaml
# Estructura actual (Rust)
- Usa: cargo install cross
- Build: cross build --release --target ${{ matrix.target }}
- Targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-gnu
```

## Cambios Requeridos para Go

### 1. release.yml - Migrar a Go

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    name: Build for ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: linux-x64
            artifact: git-navigator-linux-x64
          - os: macos-latest
            target: darwin-x64
            artifact: git-navigator-darwin-x64
          - os: windows-latest
            target: windows-x64
            artifact: git-navigator-windows-x64.exe
          - os: ubuntu-latest
            target: linux-arm64
            artifact: git-navigator-linux-arm64

    steps:
      - uses: actions/checkout@v4

      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: '1.21'

      - name: Build
        run: |
          CGO_ENABLED=0 go build -ldflags="-s -w" -o ${{ matrix.artifact }} ./cmd/git-navigator

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')

    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create release
        uses: softprops/action-gh-release@v2
        with:
          files: artifacts/*/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 2. Cambios Clave

| Aspecto | Antes (Rust) | Después (Go) |
|---------|--------------|--------------|
| Toolchain | `cargo install cross` | `actions/setup-go@v5` |
| Build command | `cross build --release` | `go build -ldflags="-s -w"` |
| CGO | N/A | `CGO_ENABLED=0` para static binary |
| Targets | cross toolchain matrix | Native os matrix |
| Binary size | ~3MB (Rust) | ~4-5MB (Go) |

### 3. Optimizaciones Opcionales

```bash
# Strip symbols y compress
-ldflags="-s -w"  # Elimina symbols, reduce ~30%

# Para even más pequeño (UPX)
# brew install upx
upx -9 git-navigator
```

## README.md

README actual es para proyecto Rust. Necesita actualizarse para Go:

- Cambiar "built in Rust" → "built in Go"
- Cambiar install instructions de Rust a Go
- Actualizar screenshots si hay
- Remover "Disclaimer: this tool has been put together by using LLM tools"

## Checklist de Migración

- [ ] Actualizar `.github/workflows/release.yml` para Go
- [ ] Actualizar `README.md` (Rust → Go)
- [ ] Agregar `go.mod`/`go.sum` a repo (ya existe)
- [ ] Testear build local: `go build -ldflags="-s -w" ./cmd/git-navigator`
- [ ] Verificar binary size post-build
- [ ] Hacer tag y push `v0.1.0` para primer release
- [ ] Verificar release se crea automáticamente

## Build Local (Test)

```bash
# Debug build
go build ./cmd/git-navigator

# Release build (stripped)
CGO_ENABLED=0 go build -ldflags="-s -w" -o git-navigator ./cmd/git-navigator

# Verificar tamaño
ls -lh git-navigator
du -sh git-navigator
```
