# kit: Development

How the workspace is laid out, how to run the gate, what CI checks, and how a release is cut.

## Prerequisites

- Rust stable with `rustfmt` and `clippy`. `mise install` sets it up from `mise.toml`.
- [filet](https://github.com/FacileStudio/filet), which the gate runs last. CI installs v0.21.0.
- lefthook, installed by `mise install` through the `postinstall` hook.
- Nix, only for the ignored tests that drive a real `nix`.

## Workspace layout

```text
Cargo.toml                 workspace, the single version number for both crates
crates/kit-core/           the library; the future Flux store links it too
  catalog/tools.yml        the Facile tool catalog, embedded with include_str!
  src/
crates/kit/                the kit binary
  src/
  tests/                   end-to-end tests that run the built binary
install.sh                 bootstrap installer for kit itself
scripts/check.sh           the gate
scripts/cask.sh            writes the Homebrew cask from checksums.txt
docs/                      this documentation
integrations/SKILL.md      the agent skill for kit
```

### kit-core modules

| Module | What it owns |
|---|---|
| `catalog` | The Facile catalog: the embedded copy, the daily refresh, the cache, the user sources file, `facile.toml` and `mise.toml` discovery, merging |
| `facile` | Facile tools: release download with retries and the API fallback, checksum, extract, atomic install, source build, agent skills, MCP registration, latest-tag lookup and its daily cache, update checks, self-update, doctor checks |
| `flathub` | The `flatpak` CLI at user scope: the remote, install, uninstall, update, list, search, ID validation |
| `nix` | The per-user `buildEnv` profile: build, prune, rollback, the `nix.json` record, attribute validation |
| `listfile` | The list file: path resolution, the seed, parsing, dedupe, the lock and the atomic write |
| `resolve` | Turning `sablier` or `nix:ripgrep` into one backend and ID, or a list of candidates |
| `paths` | Every directory kit uses on disk |
| `proc` | Spawning child processes, private to the crate |

`kit-core` returns data and typed errors (`thiserror`). It never prints and never prompts.

### The kit binary

| Path | What it owns |
|---|---|
| `src/cli/` | The clap command tree and the global flags |
| `src/cmd/` | One file per command, plus `apply.rs`, `drift.rs` and `machine.rs` that they share |
| `src/ui/` | All output: the five levels of standards/cli section 7, colors, prompts, the failure type |

The binary uses `anyhow` for errors. Output goes through `ui`, never `println!`.

## The gate

```sh
sh scripts/check.sh
```

It runs, in order, and stops at the first failure:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --locked`
4. `sh -n install.sh`
5. `filet check .`

`mise run check` runs the same script. lefthook runs the shared FacileStudio hooks on commit.
Never raise a limit in `filet.yml`: split the file instead. The limits are 250 lines and 8
functions per file, nesting 4, lines of 120 characters.

Tests that need the network or a real Nix are `#[ignore]`. Run them with:

```sh
cargo test --workspace --locked -- --ignored
```

## CI

`.github/workflows/ci.yml` runs on every push and pull request:

| Job | What it does |
|---|---|
| `check` | Installs Rust stable and filet v0.21.0, then runs `sh scripts/check.sh` |
| `nix` | Installs Nix, then runs the ignored tests: the real `nix` sync and the live catalog fetch |

## Release

1. Bump `version` under `[workspace.package]` in `Cargo.toml`. Both crates inherit it.
2. Move the `## [Unreleased]` items in `CHANGELOG.md` under `## [X.Y.Z] - YYYY-MM-DD`.
3. Run the gate, merge to `main`, then tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.

`.github/workflows/release.yml` runs on the tag:

- builds 4 targets: `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` with
  `cargo zigbuild`, `x86_64-apple-darwin` and `aarch64-apple-darwin` with `cargo build`
- packs each into `kit_<version>_<linux|darwin>_<amd64|arm64>.tar.gz` with `kit`, `README.md`
  and `LICENSE` at the top level
- writes `checksums.txt` and publishes the GitHub release, with the notes taken from the
  matching `CHANGELOG.md` section
- writes `Casks/kit.rb` with `scripts/cask.sh` and pushes it to `FacileStudio/homebrew-tap`,
  using the `HOMEBREW_TAP_GITHUB_TOKEN` secret

A tag with a hyphen, such as `v0.2.0-rc.1`, is published as a prerelease and skips the cask.

Check the release has 4 archives plus `checksums.txt`:

```sh
gh release view vX.Y.Z --repo FacileStudio/kit
```

## Static musl builds locally

kit uses rustls and no OpenSSL, so the Linux binary links statically. A local musl build looks
for `musl-gcc`. When it is missing, point the C compiler at the system `gcc`:

```sh
rustup target add x86_64-unknown-linux-musl
CC_x86_64_unknown_linux_musl=gcc cargo build --release --target x86_64-unknown-linux-musl
file target/x86_64-unknown-linux-musl/release/kit
```

`file` should say "static-pie linked". CI does not need this: zig provides the C toolchain.

Back to the [README](../README.md).
