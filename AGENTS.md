# Agent Guidelines

## Testing requirements

Every new feature or piece of functionality **must** be accompanied by tests with good coverage.

- Add unit tests in the same file as the code under test, inside a `#[cfg(test)]` module.
- Cover the happy path, relevant edge cases, and failure/error paths.
- Use `tempfile` (already a dev-dependency) for any tests that need a temporary filesystem location.
- Run `cargo test` before considering a task done. All tests must pass.
- Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` as well — the `just check` command runs both at once.

Do not submit or consider complete any change that adds functionality without corresponding tests.
