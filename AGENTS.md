# kit

The Facile package manager. Rust, one binary over one library, no server.

## What it is

`kit` installs Facile tools, Flathub apps and Nix packages, and remembers them in one list file
(`~/.config/kit/kit.yml`). `kit sync` makes a machine match the list. It replaces the install
half of `facile`: each product repo's `install.sh` shim bootstraps kit and runs
`kit add <tool>`. Sign-in is not here: `facile login` and `facile logout` stay in
FacileStudio/facile.

The normative rules live in `~/.mycelium/memory/standards/cli.md`, synced to every machine by
mycelium. When this repo and that document disagree, the document wins.

## Layout

```text
Cargo.toml                     workspace; the one version number
crates/kit-core/               the library; the Flux store will link it
  catalog/tools.yml            the Facile tool catalog, embedded with include_str!
  src/catalog/                 catalog load, daily refresh, cache, user sources, merge
  src/facile/                  Facile tools: download, verify, install, build, skills, update
  src/flathub/                 the flatpak CLI at user scope
  src/nix/                     the per-user buildEnv profile through nix and nix-env
  src/listfile.rs, listfile/   the list file: path, seed, parse, lock, atomic write
  src/resolve.rs, resolve/     a typed name to one backend and ID, or candidates
  src/paths.rs                 every directory kit touches
crates/kit/                    the binary
  src/cli/                     clap: global flags and the command tree
  src/cmd/                     one file per command, plus shared apply, drift and machine
  src/ui/                      all output and prompts
  tests/                       end-to-end tests against the built binary
install.sh                     bootstrap for kit itself
scripts/check.sh               the gate
scripts/cask.sh                the Homebrew cask, written on release
```

## The catalog is the product

`crates/kit-core/catalog/tools.yml` is canonical. kit embeds it and refreshes it daily from its
raw URL on `main`, so adding a tool needs no kit release. Keep one copy: never add a second at
the repo root. The `auth:` block is parsed and written back but unused here; it stays correct
for `facile login`.

## Rules that are load-bearing

- **A checksum mismatch aborts.** It never falls back to a source build.
- **Atomic install.** Stage beside the destination, then rename. Writing over a running binary
  corrupts it.
- **No state file for Facile tools.** What is installed is discovered by running the binaries.
  Never report a version the installed binary did not print.
- **One writer for the list file.** Every change goes through `ListStore::edit`: exclusive lock,
  read, change, atomic replace. Never write `kit.yml` any other way.
- **Never sudo.** `kit setup nix` runs the upstream installer, which asks for sudo itself.
- **Never edit shell rc files.** The binary prints the PATH line and stops. `install.sh` is the
  one exception, because it runs before there is a kit.
- **Never call `nix profile`.** It is experimental. Use `nix build --profile` and `nix-env -p`.
- **Flatpak at user scope only.** Every call passes `--user`.
- **`kit-core` never prints or prompts.** It returns data and typed errors; `crates/kit` decides.

## Style

- No inline comments and no comments inside function bodies. Every public item gets one doc
  comment above it, saying why when that is not obvious.
- No `unwrap` or `expect` outside tests.
- Errors: `thiserror` in kit-core, `anyhow` in the binary. They are lowercase, name what failed,
  and say what to do.
- Blocking I/O only (`ureq`), no tokio. rustls, never OpenSSL, so musl builds stay static.
- Output goes through `crates/kit/src/ui`, never `println!`.
- filet limits: 250 lines and 8 functions per file, nesting 4, 120 columns. Split files; never
  raise a limit.

## Commands

```sh
sh scripts/check.sh
cargo test --workspace -- --ignored
cargo run -q -p kit -- --help
sh -n install.sh
```
