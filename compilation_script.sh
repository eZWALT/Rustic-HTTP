#!/bin/sh

set -e # Exit early if any commands fail


(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
  cargo build --release --target-dir=/tmp/http-server-target --manifest-path Cargo.toml
)


exec /tmp/http-server-target/release/http-server-starter-rust "$@"
