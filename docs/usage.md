# kit — Usage

Every command and flag of `kit` 0.1.0, taken from its `--help` output, with one example each.

## Packages

A package is a bare name or `source:id`:

| Spec | Meaning |
|---|---|
| `sablier` | Look in the Facile catalog, then on Flathub, then in nixpkgs |
| `facile:sablier` | A tool from the Facile catalog, by catalog name |
| `flathub:org.mozilla.firefox` | A Flathub app, by application ID |
| `nix:ripgrep` | A nixpkgs attribute |

A bare name that matches several packages opens a picker on a terminal. Without a terminal, or
with `--yes`, kit fails and prints the candidates. Separate several packages with spaces or
commas: `kit add sablier,casier`.

## Global flags

Every command accepts these:

```text
      --file <path>    List file to read and write (default ~/.config/kit/kit.yml)
      --bin-dir <dir>  Directory Facile tools install into (default ~/.local/bin)
      --json           Print one JSON document to stdout
      --no-color       Disable colored output
  -q, --quiet          Print errors and warnings only
  -v, --verbose...     Print more detail
  -h, --help           Print help
  -V, --version        Print version
```

`--json` sends the result to stdout as one JSON document and keeps progress on stderr.
`NO_COLOR` set to any non-empty value has the same effect as `--no-color`.

## Commands

```text
add          Add packages to the list and install them
del          Remove packages from the list and uninstall them [aliases: rm, remove, uninstall]
sync         Install what the list names and is missing
list         List what the list file names and what is installed
check        Report drift between the list file and this machine
dump         Add what is installed but not listed to the list file
clean        Remove what is installed but not listed
update       Update listed packages and kit itself
undo         Roll the Nix profile back one generation
doctor       Check the health of the kit installation
setup        Set up a package backend
self-update  Replace kit with its latest release
version      Print the kit version
```

### `kit add`

Adds packages to the list file, then installs exactly what was added. With no package it opens
a picker over the Facile catalog, with listed and installed tools already checked.

```text
Usage: kit add [OPTIONS] [package]...

      --all            Add every tool in the Facile catalog
      --source         Build Facile tools from source, ignore published releases
      --version <tag>  Release tag of the Facile tool to install (default latest)
      --no-skill       Skip AI agent skill registration
      --no-mcp         Skip MCP server registration in agent harnesses
      --force          Replace a binary another package manager placed in the bin directory
  -y, --yes            Never prompt: fail when a name matches several packages
```

```sh
kit add sablier flathub:org.mozilla.firefox nix:ripgrep
```

A bare name that only Nix could answer fails on a machine without Nix and tells you to run
`kit setup nix`.

### `kit del`

Removes packages from the list file, then uninstalls them. Aliases: `rm`, `remove`,
`uninstall`. A Facile tool is removed from the bin directory only. A Nix package goes away when
kit rebuilds its profile without it. A Facile tool that an older `facile` installed and the list
does not name can still be removed this way.

```text
Usage: kit del [OPTIONS] <package>...
```

```sh
kit del nix:ripgrep
```

### `kit sync`

Installs what the list names and the machine lacks. It never removes anything. Entries whose
backend is missing are skipped with one hint per backend.

```text
Usage: kit sync [OPTIONS]
```

```sh
kit sync --quiet
```

### `kit list`

Lists what the list file names and what is installed, one section per source, with kit first.
Latest Facile releases come from a daily cache.

```text
Usage: kit list [OPTIONS]

      --check          Ask GitHub for the latest Facile releases now instead of using the daily cache
```

```sh
kit list --check
```

### `kit check`

Reports drift between the list file and the machine: listed but not installed, installed but
not listed, and outdated. It exits 1 when there is any drift and 0 when the machine matches.

```text
Usage: kit check [OPTIONS]
```

```sh
kit check --json
```

### `kit dump`

Adds Facile tools and Flathub apps that are installed but not listed to the list file. Nix needs
no dump: kit's Nix profile is built from the list. It asks first.

```text
Usage: kit dump [OPTIONS]

  -y, --yes            Do not ask for confirmation
```

```sh
kit dump --yes
```

### `kit clean`

Removes what is installed but not listed. It removes a Facile tool only from the bin directory
and only when kit placed it, and touches Nix only inside kit's own profile. It asks first, and
fails without a terminal unless you pass `--yes`.

```text
Usage: kit clean [OPTIONS]

  -y, --yes            Do not ask for confirmation
```

```sh
kit clean
```

### `kit update`

With no package, updates every listed Facile tool, every Flathub app, the Nix profile, then
kit itself last. A Facile tool that already reports its latest release is left alone. The Nix
profile is rebuilt when the registry `nixpkgs` moved.

```text
Usage: kit update [OPTIONS] [package]...

      --force          Reinstall even when already at the latest release
      --check          Report what is out of date without installing anything
      --source         Build Facile tools from source, ignore published releases
      --no-skill       Skip AI agent skill registration
```

```sh
kit update sablier casier
```

### `kit undo`

Rolls kit's Nix profile back one generation. Only Nix keeps generations, so `nix` is the only
source it accepts.

```text
Usage: kit undo [OPTIONS] [source]

  [source]  Source to undo: only nix keeps generations [default: nix]
```

```sh
kit undo
```

### `kit doctor`

Checks the install directory, every installed Facile tool, the catalog, the backends the list
names, and your sources file. It exits 1 when it finds a problem.

```text
Usage: kit doctor [OPTIONS]
```

```sh
kit doctor
```

### `kit setup nix`

Prints, then runs, the upstream multi-user Nix installer after you confirm:
`curl -sSfL https://nixos.org/nix/install | sh -s -- --daemon`. The installer asks for sudo
itself; kit never does. Afterwards kit prints the `PATH` line for its Nix profile instead of
editing your shell files.

```text
Usage: kit setup nix [OPTIONS]

  -y, --yes            Do not ask for confirmation
```

```sh
kit setup nix
```

### `kit self-update`

Replaces the running kit with its latest release. When Homebrew or another package manager
owns the binary, kit names that manager's update command instead.

```text
Usage: kit self-update [OPTIONS]

      --force          Replace kit even when it is current or a source build
      --source         Build kit from source, ignore published releases
```

```sh
kit self-update
```

### `kit version`

Prints the kit version on one line. `kit --version` prints the same.

```text
Usage: kit version [OPTIONS]
```

```sh
kit version
```

## Exit codes

| Code | When |
|---|---|
| `0` | Success |
| `1` | A package failed, `kit check` found drift, or `kit doctor` found a problem |
| `2` | The command line did not parse |

## The bootstrap script

`install.sh` installs kit itself. It takes these flags:

```text
  --bin-dir <dir>   Directory to install into (default: ~/.local/bin)
  --version <tag>   Release tag to install (default: latest)
  --source          Build from source with cargo, ignore published releases
  --no-path         Do not touch shell config files to put the bin dir on PATH
  -h, --help        Show this help
```

It adds the bin directory to `PATH` in your zsh, bash and fish files once, unless you pass
`--no-path`, set `KIT_NO_PATH`, or run it under `CI`.

```sh
curl -fsSL https://get.facile.studio | sh -s -- --bin-dir ~/bin
```

Back to the [README](../README.md).
