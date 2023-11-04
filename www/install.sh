#!/usr/bin/env bash

set -euo pipefail

if [ -n "${GITHUB_ACTIONS-}" ]; then
  set -x
fi

help() {
  cat <<EOF
Install a binary release of Heartbeat hosted on GitHub

USAGE:
    install [options]

FLAGS:
    -h, --help      Display this message
    -f, --force     Force overwriting an existing binary

OPTIONS:
    --tag TAG       Tag (version) of Heartbeat to install, defaults to the latest release
    --to LOCATION   Where to install the binary [default: ~/.local/bin]
    --target TARGET
EOF
}

crate=heartbeat
url=https://github.com/lmaotrigine/heartbeat
releases=$url/releases

say() {
  echo "install: $*"
}

say_err() {
  say "$@" >&2
}

err() {
  if [ -n "${td-}" ]; then
    rm -rf "$td"
  fi
  say_err "error: $*"
  exit 1
}

need() {
  if ! command -v "$1" > /dev/null 2>&1; then
    err "need $1 (command not found)"
  fi
}

force=0
while test $# -gt 0; do
  case $1 in
    --force | -f)
      force=1
      ;;
    --help | -h)
      help
      exit 0
      ;;
    --tag)
      tag=$2
      shift
      ;;
    --target)
      target=$2
      shift
      ;;
    --to)
      dest=$2
      shift
      ;;
    *)
      ;;
  esac
  shift
done

need curl
need install
need mkdir
need mktemp
need tar

if [ -z "${tag-}" ]; then
  need grep
  need cut
fi

if [ -z "${target-}" ]; then
  need cut
fi

if [ -z "${dest-}" ]; then
  dest=$HOME/.local/bin
fi

if [ -z "${tag-}" ]; then
  tag=$(curl --proto =https --tlsv1.2 -fsS https://api.github.com/repos/lmaotrigine/heartbeat/releases/latest | grep tag_name | cut -d'"' -f4)
fi

if [ -z "${target-}" ]; then
  # bash in MinGW (e.g. git-bash, used in GitHub Actions Windows runners) includes
  # a version suffix in `uname -s` output, e.g. MINGW64_NT-10-0.19044
  kernel=$(uname -s | cut -d- -f1)
  uname_target="$(uname -m)-$kernel"
  case $uname_target in
    aarch64-Linux)     target=aarch64-unknown-linux-musl;;
    arm64-Darwin)      target=aarch64-apple-darwin;;
    x86_64-Darwin)     target=x86_64-apple-darwin;;
    x86_64-Linux)      target=x86_64-unknown-linux-musl;;
    x86_64-Windows_NT) target=x86_64-pc-windows-msvc;;
    x86_64-MINGW64_NT) target=x86_64-pc-windows-msvc;;
    *)
      # shellcheck disable=SC2016  # we *don't* want to expand backticks.
      err 'Could not determine target from output of `uname -m`-`uname -s`, please use `--target`:' "$uname_target"
    ;;
  esac
fi

case $target in
  x86_64-pc-windows-msvc) extension=zip; need unzip;;
  *)                      extension=tar.xz;;
esac

archive="$releases/download/$tag/$crate-$tag-$target.$extension"

say_err "Repository:  $url"
say_err "Crate:       $crate"
say_err "Tag:         $tag"
say_err "Target:      $target"
say_err "Destination: $dest"
say_err "Archive:     $archive"

td=$(mktemp -d || mktemp -d -t tmp)

if [ "$extension" = "zip" ]; then
  # unzip sometimes dies on stdin, so download first
  curl --proto =https --tlsv1.2 -fsSL "$archive" > "$td"/heartbeat.zip
  unzip -d "$td" "$td"/heartbeat.zip
else
  curl --proto =https --tlsv1.2 -fsSL "$archive" | tar -C "$td" -xJ
fi

for f in "$td"/*; do
  name=$(basename "$f")
  test -x "$f" || continue
  if [ -e "$dest/$name" ] && [ $force = 0 ]; then
    err "$name already exists in $dest"
  else
    mkdir -p "$dest"
    install -m 755 "$f" "$dest"
  fi
done

rm -rf "$td"
