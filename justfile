default: check

check:
    cargo clippy --all-targets -- -D warnings
    cargo fmt --check

fmt:
    cargo fmt

test:
    cargo test

build:
    cargo build --release

install:
    cargo install --path .
