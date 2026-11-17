#!/bin/sh
#
# Bootstrap installer for kit, the Facile package manager.
#
# Every statement sits inside a function and main is the last line, so a
# download truncated mid-flight executes nothing at all.
#
# Once kit is on your PATH, it installs everything else:
#   kit add <tool>

set -eu
# shellcheck disable=SC3040
set -o pipefail 2>/dev/null || true

REPO="FacileStudio/kit"
BIN="kit"
PATH_MARKER='# kit installer'

# setup_colors runs before argument parsing, because die prints with the colors and set -u rejects them unset.
setup_colors() {
  if [ -t 1 ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    C_INFO=$(printf '\033[36m')
    C_OK=$(printf '\033[32m')
    C_WARN=$(printf '\033[33m')
    C_ERR=$(printf '\033[31m')
    C_DIM=$(printf '\033[2m')
    C_OFF=$(printf '\033[0m')
  else
    C_INFO="" C_OK="" C_WARN="" C_ERR="" C_DIM="" C_OFF=""
  fi
}

info() { printf '%s▸%s %s\n' "$C_INFO" "$C_OFF" "$*"; }
ok()   { printf '%s✓%s %s\n' "$C_OK" "$C_OFF" "$*"; }
warn() { printf '%s!%s %s\n' "$C_WARN" "$C_OFF" "$*" >&2; }
hint() { printf '  %s%s%s\n' "$C_DIM" "$*" "$C_OFF"; }
die()  { printf '%s✗%s %s\n' "$C_ERR" "$C_OFF" "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "$1 not found: $2"; }

tilde() {
  case "$1" in
    "$HOME"/*) printf '%s/%s' '~' "${1#"$HOME"/}" ;;
    "$HOME") printf '~' ;;
    *) printf '%s' "$1" ;;
  esac
}

usage() {
  cat <<EOF
Install kit, the Facile package manager.

Usage:
  install.sh [options]

Options:
  --bin-dir <dir>   Directory to install into (default: ~/.local/bin)
  --version <tag>   Release tag to install (default: latest)
  --source          Build from source with cargo, ignore published releases
  --no-path         Do not touch shell config files to put the bin dir on PATH
  -h, --help        Show this help

Environment:
  KIT_BIN_DIR       Same as --bin-dir (FACILE_BIN_DIR is read as a fallback)
  KIT_NO_PATH       Same as --no-path (FACILE_NO_PATH is read as a fallback)
  NO_COLOR          Disable colored output
EOF
}

# parse_args sets the defaults, then the flags. PATH setup is on by default and off in CI.
parse_args() {
  BIN_DIR="${KIT_BIN_DIR:-${FACILE_BIN_DIR:-$HOME/.local/bin}}"
  VERSION=""
  FROM_SOURCE=0
  SETUP_PATH=1
  [ -z "${CI:-}" ] || SETUP_PATH=0
  [ -z "${KIT_NO_PATH:-${FACILE_NO_PATH:-}}" ] || SETUP_PATH=0
  while [ $# -gt 0 ]; do
    case "$1" in
      --bin-dir) BIN_DIR="${2:?--bin-dir needs a value}"; shift 2 ;;
      --bin-dir=*) BIN_DIR="${1#*=}"; shift ;;
      --version) VERSION="${2:?--version needs a value}"; shift 2 ;;
      --version=*) VERSION="${1#*=}"; shift ;;
      --source) FROM_SOURCE=1; shift ;;
      --no-path) SETUP_PATH=0; shift ;;
      -h|--help) usage; exit 0 ;;
      *) die "unknown option: $1 (this script installs kit itself; run 'kit add <tool>' for tools)" ;;
    esac
  done
  BIN_DIR="${BIN_DIR%/}"
  PATH_CHANGED=0
  PATH_ALREADY=0
}

detect_platform() {
  case "$(uname -s)" in
    Linux) OS=linux ;;
    Darwin) OS=darwin ;;
    *) die "unsupported operating system: $(uname -s)" ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64) ARCH=amd64 ;;
    arm64|aarch64) ARCH=arm64 ;;
    *) die "unsupported architecture: $(uname -m)" ;;
  esac
}

make_workdir() {
  WORK="$(mktemp -d)"
  trap 'rm -rf "$WORK"' EXIT
  mkdir -p "$WORK/out"
}

prepare_bin_dir() {
  mkdir -p "$BIN_DIR" 2>/dev/null || die "cannot create $BIN_DIR"
  [ -w "$BIN_DIR" ] || die "$BIN_DIR is not writable"
}

# latest_tag follows the /releases/latest redirect, so it needs no GitHub API token and hits no rate limit.
latest_tag() {
  curl -fsSLI -o /dev/null -w '%{url_effective}' \
    "https://github.com/$REPO/releases/latest" 2>/dev/null |
    sed -n 's#.*/releases/tag/##p'
}

# release_base is the download URL for a tag. KIT_RELEASE_BASE_URL exists only for the installer's own tests.
release_base() {
  if [ -n "${KIT_RELEASE_BASE_URL:-}" ]; then
    printf '%s/%s' "${KIT_RELEASE_BASE_URL%/}" "$1"
  else
    printf 'https://github.com/%s/releases/download/%s' "$REPO" "$1"
  fi
}

install_from_release() {
  [ "$FROM_SOURCE" -eq 0 ] || return 1
  command -v curl >/dev/null 2>&1 || return 1
  command -v tar >/dev/null 2>&1 || return 1

  tag="$VERSION"
  [ -n "$tag" ] || tag="$(latest_tag)"
  [ -n "$tag" ] || return 1

  ver="${tag#v}"
  archive="kit_${ver}_${OS}_${ARCH}.tar.gz"
  base="$(release_base "$tag")"

  info "Downloading kit $ver for $OS/$ARCH"
  curl -fsSL -o "$WORK/$archive" "$base/$archive" 2>/dev/null || return 1
  curl -fsSL -o "$WORK/checksums.txt" "$base/checksums.txt" 2>/dev/null || return 1
  verify_checksum "$WORK" "$archive" || die "checksum mismatch for $archive"

  tar -xzf "$WORK/$archive" -C "$WORK/out" || return 1
  [ -f "$WORK/out/$BIN" ] || return 1
  atomic_install "$WORK/out/$BIN" "$BIN_DIR/$BIN"
}

# verify_checksum fails on a mismatch and never falls back: a wrong hash means a bad artifact, not a bad network.
verify_checksum() {
  dir="$1"
  file="$2"
  if command -v sha256sum >/dev/null 2>&1; then
    sum="$(cd "$dir" && sha256sum "$file")"
  elif command -v shasum >/dev/null 2>&1; then
    sum="$(cd "$dir" && shasum -a 256 "$file")"
  else
    warn "no sha256 tool available, skipping checksum verification"
    return 0
  fi
  grep -qF "${sum%% *}  $file" "$dir/checksums.txt"
}

# install_from_source builds with cargo into a scratch root, then installs the binary like a release would.
install_from_source() {
  need cargo "install Rust from https://rustup.rs"
  set -- --git "https://github.com/$REPO" --root "$WORK/root" --locked "$BIN"
  [ -z "$VERSION" ] || set -- --tag "$VERSION" "$@"

  info "Building kit from source"
  cargo install --quiet "$@" || die "cargo install failed"
  atomic_install "$WORK/root/bin/$BIN" "$BIN_DIR/$BIN"
}

# atomic_install stages beside the destination and renames, so replacing a running kit cannot corrupt it.
atomic_install() {
  src="$1"
  dest="$2"
  staged="$(dirname "$dest")/.$(basename "$dest").new.$$"
  install -m 755 "$src" "$staged" || die "cannot write $staged"
  mv -f "$staged" "$dest" || { rm -f "$staged"; die "cannot write $dest"; }
}

# path_literal writes the bin dir as $HOME/..., so the line survives a moved home and a shared dotfile repo.
# shellcheck disable=SC2016
path_literal() {
  case "$BIN_DIR" in
    "$HOME"/*) printf '$HOME/%s' "${BIN_DIR#"$HOME"/}" ;;
    *) printf '%s' "$BIN_DIR" ;;
  esac
}

# path_snippet guards the prepend at runtime, so a config sourced twice does not duplicate PATH entries.
# shellcheck disable=SC2016
path_snippet() {
  kind="$1"
  dir="$2"
  case "$kind" in
    fish)
      printf '\n%s\nif not contains %s $PATH\n    set -gx PATH %s $PATH\nend\n' \
        "$PATH_MARKER" "$dir" "$dir"
      ;;
    *)
      printf '\n%s\ncase ":$PATH:" in\n  *":%s:"*) ;;\n  *) export PATH="%s:$PATH" ;;\nesac\n' \
        "$PATH_MARKER" "$dir" "$dir"
      ;;
  esac
}

# uses_shell is true when the shell is installed, is the login shell, or has left a config file behind.
uses_shell() {
  shell_name="$1"
  shift
  command -v "$shell_name" >/dev/null 2>&1 && return 0
  [ "$(basename "${SHELL:-}")" = "$shell_name" ] && return 0
  for shell_f in "$@"; do
    [ -e "$shell_f" ] && return 0
  done
  return 1
}

# path_targets prints "kind:file" for zsh, bash and fish. Bash gets .bashrc and .bash_profile when they exist.
path_targets() {
  zdot="${ZDOTDIR:-$HOME}"
  fishrc="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"

  if uses_shell zsh "$zdot/.zshrc"; then
    printf 'posix:%s\n' "$zdot/.zshrc"
  fi

  if uses_shell bash "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; then
    wrote=0
    for f in "$HOME/.bashrc" "$HOME/.bash_profile"; do
      if [ -f "$f" ]; then
        printf 'posix:%s\n' "$f"
        wrote=1
      fi
    done
    [ "$wrote" -eq 1 ] || printf 'posix:%s\n' "$HOME/.bashrc"
  fi

  if uses_shell fish "$fishrc"; then
    printf 'fish:%s\n' "$fishrc"
  fi

  return 0
}

# persist_path appends the snippet to one rc file unless it already mentions the marker or the bin dir.
persist_path() {
  kind="$1"
  file="$2"
  literal="$3"
  if [ -f "$file" ] &&
     grep -qF -e "$PATH_MARKER" -e "$literal" -e "$BIN_DIR" "$file" 2>/dev/null; then
    PATH_ALREADY=$((PATH_ALREADY + 1))
    return 0
  fi
  dir="$(dirname "$file")"
  mkdir -p "$dir" 2>/dev/null || { warn "cannot create $(tilde "$dir")"; return 0; }
  if path_snippet "$kind" "$literal" >> "$file" 2>/dev/null; then
    ok "PATH set in $(tilde "$file")"
    PATH_CHANGED=$((PATH_CHANGED + 1))
  else
    warn "cannot write $(tilde "$file")"
  fi
}

# configure_path never fails the install: the binary is on disk, and an unwritable dotfile is only a warning.
configure_path() {
  [ "$SETUP_PATH" -eq 1 ] || return 0
  case ":$PATH:" in
    *":$BIN_DIR:"*) return 0 ;;
  esac

  literal="$(path_literal)"
  targets="$(path_targets)"
  [ -n "$targets" ] || return 0

  while IFS= read -r target; do
    [ -n "$target" ] || continue
    persist_path "${target%%:*}" "${target#*:}" "$literal"
  done <<EOF
$targets
EOF
}

# report runs the installed binary, persists PATH, and tells the user the one manual step left, if any.
report() {
  version="$("$BIN_DIR/$BIN" --version 2>/dev/null | head -n1)" ||
    die "$BIN installed to $BIN_DIR/$BIN but does not run"
  ok "${version:-$BIN} installed to $(tilde "$BIN_DIR")/$BIN"

  configure_path

  case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
      if [ "$PATH_CHANGED" -gt 0 ]; then
        info "Open a new terminal, or run this once in the current one:"
      elif [ "$PATH_ALREADY" -gt 0 ]; then
        info "Already in your shell config. Open a new terminal, or run:"
      else
        warn "$(tilde "$BIN_DIR") is not on your PATH"
      fi
      hint "export PATH=\"$BIN_DIR:\$PATH\""
      ;;
  esac

  shadow="$(command -v "$BIN" 2>/dev/null || true)"
  if [ -n "$shadow" ] && [ "$shadow" != "$BIN_DIR/$BIN" ]; then
    warn "another $BIN comes first on your PATH: $shadow"
  fi

  info "Run \`kit add <tool>\` to install your tools"
}

main() {
  setup_colors
  parse_args "$@"
  info "Installing kit"
  detect_platform
  make_workdir
  prepare_bin_dir
  install_from_release || install_from_source
  report
}

main "$@"
