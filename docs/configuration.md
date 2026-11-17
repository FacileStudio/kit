# kit — Configuration

Where kit reads and writes its files, every environment variable it reads, and how its Nix
profile and Flathub remote are set up.

## The list file

kit picks the list file in this order and uses the first that is set:

1. `--file <path>`
2. `KIT_FILE`, when it is not empty
3. `$XDG_CONFIG_HOME/kit/kit.yml`, which is `~/.config/kit/kit.yml` by default

When that file does not exist yet, kit reads `/etc/kit/kit.yml` instead. A distribution such as
Flux ships that file as the seed for new users. The first `kit add` or `kit del` writes your own
copy. When neither file exists, the list starts empty.

```yaml
version: 1
facile:
  - sablier
nix:
  - ripgrep
flathub:
  - org.mozilla.firefox
```

| Key | Type | What it holds |
|---|---|---|
| `version` | number | Always `1`. Any other value is an error. Missing means `1`. |
| `facile` | list of strings | Facile tools, by catalog name |
| `nix` | list of strings | nixpkgs attributes |
| `flathub` | list of strings | Flathub application IDs, in reverse DNS form |

Rules kit enforces:

- Any other top-level key is an error that names the key.
- Each section is deduplicated on read and on write, keeping the first occurrence.
- Writes keep the order you gave and always put the keys in the order above.
- Every write takes an exclusive lock on `kit.yml.lock` beside the file, so two kit runs
  never lose each other's change. The new file replaces the old one in one rename.
- Comments are lost on the next write, because the YAML library does not keep them.

## The sources file

The sources file adds your own tools to the Facile catalog. kit reads the first of these that
exists:

1. `$XDG_CONFIG_HOME/kit/config.yml`
2. `$XDG_CONFIG_HOME/kit/config.yaml`
3. `~/.facile.yml`
4. `~/.facile.yaml`
5. `$XDG_CONFIG_HOME/facile/config.yml`
6. `$XDG_CONFIG_HOME/facile/config.yaml`
7. `$XDG_CONFIG_HOME/facile/sources.yml`
8. `$XDG_CONFIG_HOME/facile/sources.yaml`

Items 3 to 8 are the files the Go `facile` read, so an existing setup keeps working. No file
means no extra tools.

```yaml
palette: saravenpi/palette
pres: saravenpi/pres@dev

my-tool:
  summary: A short description
  repo: owner/my-tool
  branch: main
  bin: my-tool
  build: rust
  versionCmd: my-tool --version
  versionPattern: "v(\\d+\\.\\d+\\.\\d+)"

lists:
  - https://example.com/facile-repos.txt
```

- `name: owner/repo` adds a tool. `@branch` pins a branch.
- A full object takes `summary`, `repo`, `branch`, `bin`, `bins`, `build`, `srcSubdir`,
  `asset`, `skill`, `goVersionVar`, `requires`, `versionCmd` and `versionPattern`.
- An entry with only a repo reads the repo's `facile.toml`, or the `[facile]` block of its
  `mise.toml`, for the rest.
- `lists:` takes URLs, or `{name, url}` objects. Each URL returns one `owner/repo` per line, or
  a PEP 503 simple index.
- `tools:`, `single:` and `sources:` blocks, and a bare list at the top, are accepted too.
- A tool with the same name as a catalog tool replaces it. `kit doctor` warns about that.

## The catalog

kit embeds the catalog (`crates/kit-core/catalog/tools.yml`) and refreshes it once a day from
`https://raw.githubusercontent.com/FacileStudio/kit/main/crates/kit-core/catalog/tools.yml`.
The copy lives in `$XDG_CACHE_HOME/kit/tools.yml`. When the network fails, kit uses that copy,
then the embedded one. A cache older than the running kit binary counts as stale, so a new kit
never hides its newer catalog behind an old cache.

## Environment variables

| Variable | Default | What it does |
|---|---|---|
| `KIT_FILE` | `~/.config/kit/kit.yml` | List file path when `--file` is not given |
| `KIT_BIN_DIR` | `~/.local/bin` | Where Facile tools go when `--bin-dir` is not given |
| `FACILE_BIN_DIR` | none | Read when `KIT_BIN_DIR` is unset, for old setups |
| `KIT_CATALOG` | none | Path to a local catalog that replaces the published one and the sources file |
| `FACILE_CATALOG` | none | Read when `KIT_CATALOG` is unset or empty |
| `XDG_CONFIG_HOME` | `~/.config` | Parent of `kit/kit.yml` and `kit/config.yml` |
| `XDG_CACHE_HOME` | `~/.cache` | Parent of `kit/`, which holds the catalog and release caches |
| `XDG_STATE_HOME` | `~/.local/state` | Parent of `kit/`, which holds the Nix profile and its record |
| `XDG_DATA_HOME` | `~/.local/share` | Where kit looks for `mycelium/skills` when it registers skills |
| `GH_TOKEN` | none | GitHub token for release downloads through the API |
| `GITHUB_TOKEN` | none | Read when `GH_TOKEN` is unset |
| `NO_COLOR` | none | Any non-empty value turns colors off |

Without either variable kit tries `gh auth token`, then goes on without a token. kit sends the
token to the GitHub API host only, never to a download host. It uses the API when a release
download keeps failing with a server error, and when a latest-tag lookup needs it.

A source build sets `MISE_IGNORED_CONFIG_PATHS` for the build, with the checkout appended, so
mise never runs a freshly cloned repo's hooks.

`install.sh` also reads `KIT_NO_PATH` (and `FACILE_NO_PATH` as a fallback) and `CI`: any of
them stops it from editing your shell files.

## The Nix profile

kit keeps one Nix profile per user at `$XDG_STATE_HOME/kit/nix-profile`, which is
`~/.local/state/kit/nix-profile` by default. Put its `bin` on your `PATH`:

```sh
export PATH="$HOME/.local/state/kit/nix-profile/bin:$PATH"
```

- Each sync builds one `buildEnv` of the whole `nix:` list with `nix build --impure --profile`,
  from the `nixpkgs` your flake registry names.
- kit records the nixpkgs revision and the attribute list in `$XDG_STATE_HOME/kit/nix.json`.
  `kit sync` rebuilds only when the list or the registry revision changed.
- After each build kit keeps the last 10 generations with
  `nix-env -p <profile> --delete-generations +10`. Garbage collection run by root does not
  prune profiles in a user's home, so kit prunes its own.
- `kit undo` runs `nix-env -p <profile> --rollback`.
- kit checks an attribute exists with `nix eval` before it writes it to the list.
- kit never calls `nix profile`, which is still experimental.

## The Flathub remote

kit installs Flatpak apps at user scope only. Before an install it runs:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

Install, uninstall and update run with `--user -y --noninteractive`. kit reads what is
installed with `flatpak list --user --app --columns=application`, so system-wide apps are
neither listed nor removed. Application IDs must be reverse DNS, such as
`org.mozilla.firefox`.

## Agent skills

When a Facile tool ships a skill, `kit add` registers it for Claude Code
(`~/.claude/skills/<skill>/SKILL.md`), Codex (a marked block in `~/.codex/AGENTS.md`) and the
mycelium skills directory when it already exists. Pass `--no-skill` to skip that. Tools the
catalog lists under `mcp:` also run `<bin> install` to register their MCP server, unless you
pass `--no-mcp`.

Back to the [README](../README.md).
