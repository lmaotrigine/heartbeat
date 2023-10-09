#!/usr/bin/env -S just --justfile

_default:
  just --list

tag := `git rev-parse --short HEAD`
image := "ghcr.io/lmaotrigine/heartbeat"

all: clean build

ci-lint: check check-forbidden
  just test --all-features

build *args:
  @just _cargo build {{args}}

_cargo *args:
  SQLX_OFFLINE=1 cargo {{args}}

check:
  cargo fmt --all -- --check
  @just _cargo clippy --all --all-features -- -D warnings

check-forbidden:
  @bin/forbid

clean:
  cargo clean

test *args:
  RUST_BACKTRACE=1 just _cargo nextest run {{args}}

_docker *args:
  TAG={{tag}} IMAGE={{image}} docker buildx {{args}}

bake:
  @just _docker bake

push:
  @just _docker bake --push

run *args:
  docker compose up -d {{args}}

pull:
  docker pull {{image}}:latest

migrate:
  cargo run --bin migrate_db

gensecret:
  #!/usr/bin/env python3
  from base64 import b64encode
  from secrets import token_bytes
  print(b64encode(token_bytes(48)).decode('ascii'))

_lint-static X:
  {{X}} tsc
  {{X}} stylelint static/*.css
  {{X}} prettier --check static/*.{mjs,css}

lint-static:
  just _lint-static bunx

lint-static-ci:
  just _lint-static npx
