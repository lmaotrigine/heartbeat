#!/usr/bin/env -S just --justfile

_default:
  just --list

alias t := test
alias c := check

log := "info"
toolchain := env_var_or_default("RUST_TOOLCHAIN", "stable")
rustfmt_toml := if toolchain == "nightly" { "nightly-rustfmt.toml" } else { "rustfmt.toml" }

export RUST_LOG := log
export SQLX_OFFLINE := "1"

tag := `git rev-parse --short HEAD`
image := "ghcr.io/lmaotrigine/heartbeat"
dsn := "postgres://heartbeat@db/heartbeat"
release := `git describe --tags --exact-match 2>/dev/null || echo`

all: clean (build "--release") (test "--all-features")

update *args:
  cargo +{{toolchain}} update --all {{args}}

refresh *args="--all-features":
  cargo +{{toolchain}} generate-lockfile
  cargo +{{toolchain}} sqlx prepare --database-url {{dsn}} -- {{args}}

ci: build-book lint-static-ci (test "--all-features") (fmt "--check") (clippy "-D" "warnings")
  ./bin/forbid
  cargo +{{toolchain}} update --locked --package heartbeat

build *args:
  cargo +{{toolchain}} build {{args}}

fmt *args:
  cargo +{{toolchain}} fmt --all -- {{args}} --config-path {{rustfmt_toml}}

check: (fmt "--check") clippy (test "--all-features") forbid lint-static
  git diff --no-ext-diff --quiet --exit-code

publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone git@github.com:lmaotrigine/heartbeat.git tmp/release
  cd tmp/release
  VERSION=$(sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1)
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cd ../..
  rm -rf tmp/release

push: check
  ! git branch | grep '* main'
  git push origin

pr: push
  gh pr create --web

# Clean up feature branch BRANCH
done branch=`git rev-parse --abbrev-ref HEAD`:
  git checkout main
  git diff --no-ext-diff --quiet --exit-code
  git pull --rebase origin main
  git diff --no-ext-diff --quiet --exit-code {{branch}}
  git branch -D {{branch}}

clippy *args:
  cargo +{{toolchain}} clippy --all-targets --all-features -- {{args}}

forbid:
  ./bin/forbid

sloc:
  @cat src/*.rs | sed '/^\s*$/d' | wc -l

ws:
  ! rg '\s+$' . .github

clean:
  cargo +{{toolchain}} clean

test *args:
  RUST_BACKTRACE=1 cargo +{{toolchain}} nextest run {{args}}

bake *args:
  TAG={{tag}} IMAGE_NAME={{image}} RELEASE={{release}} docker buildx bake {{args}}

docker-push:
  @just bake --push

run *args:
  docker compose up -d {{args}}

docker-pull:
  docker pull {{image}}:latest

migrate dsn=dsn:
  cargo +{{toolchain}} run --features migrate -- migrate --database-dsn {{dsn}}

new-migration name:
  cargo +{{toolchain}} sqlx migrate add {{name}}

install-dev-deps:
  rustup install nightly
  rustup update nightly
  cargo install ripgrep
  cargo install cargo-sqlx
  cargo install cargo-nextest
  cargo install mdbook mdbook-linkcheck

gensecret:
  #!/usr/bin/env python3
  from base64 import b64encode
  from secrets import token_bytes
  print(b64encode(token_bytes(48)).decode('ascii'))

_lint-static X:
  {{X}} tsc
  {{X}} stylelint {static,www}/*.css
  {{X}} eslint static/*.mjs

lint-static:
  just _lint-static bunx

lint-static-ci:
  #!/usr/bin/env bash
  if git diff --no-ext-diff --quiet --exit-code -- static/ www/; then
    echo "Skipping static asset linting because no changes were made"
  else
    just _lint-static npx
  fi

build-book:
  mdbook build book
