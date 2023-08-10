_default:
  just --list

tag := `git rev-parse --short HEAD`

all: clean build

build *args:
  cargo build {{args}}

check:
  cargo fmt --all -- --check
  SQLX_OFFLINE=1 cargo clippy --all-features -- -D warnings

clean:
  cargo clean

test *args:
  RUST_BACKTRACE=1 SQLX_OFFLINE=1 cargo nextest run {{args}}

docker:
  TAG={{tag}} docker buildx bake

push:
  TAG={{tag}} docker buildx bake --push
