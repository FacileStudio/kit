---
name: kit
description: >
  Facile package manager. Use when the user asks to install, remove, update,
  or list a Facile tool, a Flathub app or a Nix package, to set up a machine
  from a list, to check drift, or mentions kit, kit.yml or facile install.
---

# kit: Facile package manager

Binary: `kit`
List file: `~/.config/kit/kit.yml` (or `--file`, or `KIT_FILE`)

## When to apply

Use when the user wants a tool installed or removed and wants the machine to remember it.
Triggers: "install sablier", "add a Facile tool", "install firefox from Flathub", "install
ripgrep with nix", "update my tools", "what is installed", "set up this machine", "kit sync",
"kit.yml", "facile install"

## Commands

```text
kit add <package>...           Add to the list and install
  --all                       Every tool in the Facile catalog
  --version <tag>             Release tag of the Facile tool
  --source                    Build from source
  --no-skill / --no-mcp       Skip agent skill or MCP registration
  --force                     Replace a binary another manager placed
  -y / --yes                  Never prompt; fail on an ambiguous name

kit del <package>...           Remove from the list and uninstall (aliases rm, remove)
kit sync                       Install what the list names and is missing
kit list                       Listed packages and their state
  --check                     Ask GitHub for latest releases now
kit check                      Drift between list and machine; exit 1 on drift
kit dump [--yes]               Add installed but unlisted packages to the list
kit clean [--yes]              Remove installed but unlisted packages
kit update [package]...        Update everything listed, then kit itself
  --check                     Report only
  --force                     Reinstall even when current
kit undo                       Roll the Nix profile back one generation
kit doctor                     Health checks
kit setup nix [--yes]          Run the upstream Nix installer
kit self-update                Replace kit with its latest release

Global: --file <path>  --bin-dir <dir>  --json  --no-color  -q  -v
```

### Package names
- `sablier`: bare name, tried in the Facile catalog, then Flathub, then nixpkgs
- `facile:sablier`, `flathub:org.mozilla.firefox`, `nix:ripgrep`: explicit source
- Several at once: spaces or commas, `kit add sablier,casier`

## Rules
- Use `--json` when parsing output, and `--yes` in non-interactive runs: without a terminal,
  `clean`, `dump` and `setup nix` refuse, and an ambiguous name fails with the candidates
- Prefer an explicit `source:id` when a name could match several sources
- Never edit `kit.yml` while a kit command runs; use `kit add` and `kit del`
- Sign-in is not kit: use `facile login`
- kit never runs sudo and never edits shell files; relay the PATH line it prints to the user
- Run `kit <command> --help` for exact syntax when unsure
