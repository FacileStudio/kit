# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [Unreleased]

## [0.1.0] - 2026-11-17

### Added

- The list file `~/.config/kit/kit.yml` with `facile`, `nix` and `flathub` sections. kit edits
  it under an exclusive lock and replaces it atomically. `--file` and `KIT_FILE` pick another
  path, and `/etc/kit/kit.yml` seeds it when it is missing.
- `kit add`, `kit del` (aliases `rm`, `remove`, `uninstall`), `kit sync`, `kit list`,
  `kit check`, `kit dump`, `kit clean`, `kit update`, `kit undo`, `kit doctor`,
  `kit setup nix`, `kit self-update` and `kit version`.
- `--json` on every command, printing one JSON document to stdout.
- Name resolution: a bare name tries the Facile catalog, then Flathub, then nixpkgs.
  `facile:`, `flathub:` and `nix:` prefixes pick the source. Several matches open a picker on a
  terminal and fail with the candidates elsewhere.
- The Facile backend, ported from the Go `facile` installer: release download with retries and
  the GitHub API fallback, checksum verification that aborts on a mismatch, atomic install,
  source builds, agent skill registration and MCP registration.
- The Facile catalog, moved here from `FacileStudio/facile` unchanged, refreshed daily from
  `main`. The user sources files that `facile` read still work.
- The Flathub backend: Flatpak apps at user scope, with the Flathub remote added on first use.
- The Nix backend: one per-user `buildEnv` profile at `~/.local/state/kit/nix-profile`, built
  with `nix build --profile`, pruned to 10 generations and rolled back with `kit undo`.
- `install.sh` bootstrap with `--bin-dir`, `--version`, `--source` and `--no-path`.
- Release archives for Linux (static musl) and macOS on amd64 and arm64, `checksums.txt`, and a
  Homebrew cask in `FacileStudio/tap`.
