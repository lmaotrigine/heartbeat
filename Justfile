_default:
  just --list

name := "ghcr.io/lmaotrigine/heartbeat"
tag := `git rev-parse --short HEAD`
img := name + ":" + tag
latest := name + ":latest"

all: clean build

build *args:
  cargo build {{args}}

check:
  cargo fmt --all -- --check
  cargo clippy --all-features -- -D warnings

clean:
  cargo clean

test *args:
  RUST_BACKTRACE=1 cargo nextest run {{args}}

docker:
  #!/bin/sh -eux
  docker build -t {{img}} .
  docker tag {{img}} {{latest}}

push: docker
  #!/bin/sh -eux
  docker push -a {{name}}
  docker rmi {{img}}
