# kit

kit installs things and remembers them in a list.

It keeps one list file of Facile tools, Flathub apps and Nix packages, and makes the machine
match it. It replaces the `facile` installer for everything except sign-in.

## What it does

- Installs Facile tools from their GitHub releases, checksum-verified, into `~/.local/bin`
- Installs Flathub apps with Flatpak at user scope
- Installs Nix packages into one per-user profile, without sudo and without `nix profile`
- Records every `kit add` and `kit del` in `~/.config/kit/kit.yml`
- Rebuilds a machine from that file with `kit sync`, and reports drift with `kit check`
- Updates Facile tools, Flathub apps, the Nix profile and itself with `kit update`
- Rolls the Nix profile back one generation with `kit undo`
- Prints one JSON document with `--json` on every command

## Stack

| Layer | Tech |
|---|---|
| CLI | Rust 2021, clap 4.5, anyhow 1, dialoguer 0.11 |
| Runtime | kit-core library: ureq 2 with rustls, serde_yaml 0.9, fd-lock 4, thiserror 2 |
| Deploy | Static musl binaries for Linux, plus macOS builds, from GitHub Releases and a Homebrew cask |

## Install

```sh
curl -fsSL https://get.facile.studio | sh
```

If that host is down, fetch the same script from GitHub:

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/kit/main/install.sh | sh
```

Installs to `~/.local/bin`. Pass `--bin-dir <dir>` to change that, `--source` to build from
source, `--no-path` to leave your shell files alone. With Homebrew:

```sh
brew install FacileStudio/tap/kit
```

## Usage

```sh
kit add sablier firefox ripgrep
kit del ripgrep
kit sync
kit list
kit check
kit update
```

A bare name is looked up in the Facile catalog, then on Flathub, then in nixpkgs. Prefix it to
pick the source yourself: `facile:sablier`, `flathub:org.mozilla.firefox`, `nix:ripgrep`.
Every command and flag: [docs/usage.md](docs/usage.md).

## The list file

```yaml
version: 1
facile:
  - sablier
  - casier
nix:
  - ripgrep
flathub:
  - org.mozilla.firefox
```

kit writes this file itself, under a lock, and keeps the order. Edit it by hand if you like,
then run `kit sync`. Comments are lost on the next write. Copy the file to a new machine and
`kit sync` installs the same set.

## Where each source works

| Source | Flux | Debian and other Linux | macOS |
|---|---|---|---|
| `facile` | yes | yes | yes |
| `flathub` | yes, desktop profile | when `flatpak` is installed | no |
| `nix` | yes | after `kit setup nix` | after `kit setup nix` |

`kit sync` skips the entries whose backend is missing, says how to get it, and still installs
the rest.

## What kit does not do

- Sign-in. `facile login` and `facile logout` still do that.
- mise, apt or Homebrew backends. kit does not manage system packages.
- Sync the list between machines. Copy the file, or keep it in your dotfiles.
- Run sudo or edit your shell files. It prints the line to add instead.

## Configuration

| Variable | What it does |
|---|---|
| `KIT_FILE` | List file path, when `--file` is not given |
| `KIT_BIN_DIR` | Where Facile tools go, default `~/.local/bin` |
| `KIT_CATALOG` | A local catalog that replaces the published one |

Full reference: [docs/configuration.md](docs/configuration.md).

## Documentation

| Doc | What's in it |
|---|---|
| [Usage](docs/usage.md) | Every command and flag, with an example each |
| [Configuration](docs/configuration.md) | List file, sources file, environment, Nix and Flathub details |
| [Development](docs/development.md) | Workspace layout, the gate, CI and the release flow |
| [Parity](docs/parity.md) | Each test of the Go `facile` installer and its Rust port |

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
