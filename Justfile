#!/usr/bin/env -S just --justfile

_default:
  just --list

tag := `git rev-parse --short HEAD`
image := "ghcr.io/lmaotrigine/heartbeat"

all: clean build

ci-lint: check test

build *args:
  @just _cargo build {{args}}

_cargo *args:
  SQLX_OFFLINE=1 cargo {{args}}

check:
  cargo fmt --all -- --check
  @just _cargo clippy --all-features -- -D warnings

clean:
  cargo clean

test *args:
  RUST_BACKTRACE=1 just _cargo nextest run {{args}}

_docker *args:
  TAG={{tag}} docker buildx {{args}}

bake:
  @just _docker bake

push:
  @just _docker bake --push

run *args:
  docker compose up -d {{args}}

pull:
  docker pull {{image}}:latest
