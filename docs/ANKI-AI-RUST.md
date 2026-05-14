## What the Rust API actually is

The Rust API documentation lives in `rslib/` and is generated via `cargo doc`. The main crates are `anki` (the core library in `rslib/`) and `anki_io` (I/O utilities).

The key insight from the architecture docs: the majority of backend logic lives in a Rust library (`rslib/`). Calls to `pylib` proxy requests to `rslib` and return the results. The Python package wraps the Rust code via a private module called `rsbridge`.

So the Python `anki` package you're using is literally just a thin wrapper around a Rust library. **The real implementation is already in Rust.**

## Can you use `rslib` directly in a Rust project?

**Technically yes, practically it's painful.** Here's the problem:

`rslib` is not published to crates.io. It's an internal crate inside the Anki monorepo, designed to be built as part of Anki's own Bazel-based build system. To use it you'd need to either:

1. **Git dependency** — add `ankitects/anki` as a git dependency in your `Cargo.toml`. This pulls in the entire Anki monorepo as a dependency, which is massive and has complex build requirements.
2. **Build and link the `.so` manually** — build `rslib` yourself, then link against it. Fragile and version-sensitive.
3. **Fork/vendor `rslib`** — copy the relevant crates into your own repo. Significant maintenance burden.

Also, the protobuf definitions used for the Python/Rust bridge are not considered public API. That means the interfaces can change between Anki versions without notice.

## The realistic path for a Rust rewrite

Given that `rslib` is essentially Anki's core, a Rust CLI that depends on it is genuinely possible — but the build setup is non-trivial. Here's the rough approach:

```toml
# Cargo.toml
[dependencies]
anki = { git = "https://github.com/ankitects/anki", tag = "25.9.4" }
```

Then your CLI would call into the same collection-opening, syncing, and card management APIs that Python currently delegates to. You'd use `clap` instead of `typer`, and the logic layer would be nearly a 1:1 port since your Python is already just thin glue over `rslib`.

## Bottom line

It **is** possible to rewrite in Rust using `rslib` directly — your Python code is genuinely just orchestrating calls that Rust already implements. The blocker isn't conceptual, it's build complexity: `rslib` isn't packaged for external consumption, so you'd be fighting the monorepo build system rather than writing application code.

If you want to try it, the most pragmatic approach is: pin to a specific Anki git tag (matching your current `anki==25.9.4`), add it as a path/git dep, and expect to spend some time getting the build working before you can write any CLI code. Want me to sketch out what the project structure and `Cargo.toml` would look like?
