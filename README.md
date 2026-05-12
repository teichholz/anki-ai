# anki-ai

A headless Anki manager for AI agents. It exposes a JSON-first CLI that lets you create, search, update, and sync Anki flashcards from scripts or AI assistants without the Anki desktop app.

## Prerequisites

- **Rust 1.75+** — install via [rustup](https://rustup.rs/)
- **protoc** (Protocol Buffers compiler) — required to build the Anki library

  ```bash
  # Arch / Manjaro
  sudo pacman -S protobuf

  # Ubuntu / Debian
  sudo apt install protobuf-compiler

  # macOS
  brew install protobuf
  ```

## Install

```bash
cargo install --path .
```

The `anki-ai` binary is placed in `~/.cargo/bin/`.

## Dev setup

```bash
# Build (debug)
cargo build

# Lint + format check
just check

# Run tests (single-threaded — tests share a temp collection)
just test
# or: cargo test -- --test-threads=1

# Release build
just build
# or: cargo build --release
```

## Environment variable

| Variable | Default | Description |
|---|---|---|
| `ANKI_COLLECTION_PATH` | Auto-detected from `~/.local/share/Anki2/` | Absolute path to `collection.anki2` |

Set this if auto-detection fails (e.g. non-standard profile name):

```bash
export ANKI_COLLECTION_PATH="$HOME/.local/share/Anki2/MyProfile/collection.anki2"
```

## First-time setup

```bash
anki-ai auth login
```

Prompts for your AnkiWeb email and password, stores the auth token at `~/.config/anki-ai/auth.json`.

## Commands

| Command | Description |
|---|---|
| `auth login` | Exchange AnkiWeb credentials for an auth token |
| `sync` | Bidirectional sync with AnkiWeb (`--no-media` to skip media, `--upload` to force upload) |
| `info` | Show collection statistics (note count, card count, path) |
| `snapshot` | Create a timestamped backup of the collection file |
| `snapshots` | List available snapshots |
| `restore SNAPSHOT` | Restore collection from a snapshot (`--yes` to skip prompt) |
| `decks list` | List all decks with due card counts |
| `decks create NAME` | Create a deck (supports `::` for nested decks) |
| `decks delete NAME` | Delete a deck and all its cards (`--yes` to skip prompt) |
| `notes add` | Add a note (`--deck NAME --field Front=X --field Back=Y`) |
| `notes get ID` | Get a note by ID |
| `notes search QUERY` | Search notes using Anki search syntax |
| `notes update ID` | Update note fields (`--field Name=Value`) |
| `notes delete ID` | Delete a note by ID (`--yes` to skip prompt) |
| `cards list QUERY` | Find card IDs matching a search query |
| `cards info ID` | Show scheduling info for a card |
| `cards suspend ID...` | Suspend one or more cards |
| `cards unsuspend ID...` | Unsuspend one or more cards |
| `tags list` | List all tags in the collection |
| `tags add TAG... [-q QUERY]` | Add tags to notes matching a query |
| `tags remove TAG... -q QUERY` | Remove tags from notes matching a query |
| `tags rename OLD NEW` | Rename a tag across all notes |
| `notetypes list` | List all note types with note counts |
| `notetypes fields NAME` | List the fields of a note type |
| `media upload FILE...` | Copy files into the collection media folder |
| `skill` | Print the Claude skill file content (embed in AI context) |

## Typical workflow

```bash
# 1. Authenticate once
anki-ai auth login

# 2. Pull cards from phone
anki-ai sync

# 3. See what decks exist
anki-ai decks list

# 4. Add a new card
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"

# 5. Push the new card back
anki-ai sync

# 6. Verify the card is there
anki-ai notes search "rendir"
```

## Claude skill

The `skill` command prints a Markdown skill file designed to be loaded into an AI agent's context:

```bash
anki-ai skill
```

Pass the output to your AI assistant so it knows how to invoke the CLI correctly.
