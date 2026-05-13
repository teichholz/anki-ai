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

release version:
    sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    cargo build --release
    git add Cargo.toml Cargo.lock
    git commit -m "chore: release v{{version}}"
    git tag v{{version}}
    git push && git push --tags
    gh release create v{{version}} ./target/release/anki-ai \
        --title "v{{version}}" \
        --notes "Release v{{version}}"
