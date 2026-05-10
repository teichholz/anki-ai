# anki-ai

Headless CLI for managing Anki flashcards on an Ubuntu server. Uses the `anki` Python package directly (no AnkiConnect, no display required). Syncs bidirectionally with AnkiWeb.

**Pinned:** `anki==25.9.4`

## Install

```bash
uv sync
```

## Quick Start

```bash
# Authenticate once
uv run anki-ai auth login

# Pull cards from AnkiWeb (e.g. cards created on your phone)
uv run anki-ai sync

# List decks
uv run anki-ai decks list

# Add a card
uv run anki-ai notes add --deck "Spanish" --front "rendir" --back "to yield"

# Push new card back
uv run anki-ai sync

# Search notes
uv run anki-ai notes search "deck:Spanish"
```

## Collection Path

Auto-detected from `~/.local/share/Anki2/`. Override with:

```bash
export ANKI_COLLECTION_PATH=~/.local/share/Anki2/YourProfile/collection.anki2
```

## Auth Token

Stored at `~/.config/anki-ai/auth.json` (mode `0600`). AnkiWeb credentials are never persisted — only the session token.

## All Commands

| Command | Description |
|---|---|
| `anki-ai auth login` | Authenticate with AnkiWeb |
| `anki-ai sync` | Bidirectional sync with AnkiWeb |
| `anki-ai decks list` | List all decks as JSON |
| `anki-ai notes add --deck D --front F --back B` | Add a Basic note |
| `anki-ai notes search QUERY` | Search notes, return JSON |

See `skill/anki.md` for the full command reference used by AI agents.
