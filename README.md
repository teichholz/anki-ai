# anki-ai

Headless CLI for managing Anki flashcards on an Ubuntu server. Uses the `anki` Python package directly (no AnkiConnect, no display required). Syncs bidirectionally with AnkiWeb.

**Pinned:** `anki==25.9.4`

## Install

```bash
uv tool install .
```

For development:

```bash
uv sync
```

## Quick Start

```bash
# Authenticate once
anki-ai auth login

# Pull cards from AnkiWeb (e.g. cards created on your phone)
anki-ai sync

# List decks
anki-ai decks list

# Add a card
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"

# Push new card back
anki-ai sync

# Search notes
anki-ai notes search "deck:Spanish"
```

## Collection Path

Auto-detected from `~/.local/share/Anki2/`. Override with:

```bash
export ANKI_COLLECTION_PATH=~/.local/share/Anki2/YourProfile/collection.anki2
```

## Auth Token

Stored at `~/.config/anki-ai/auth.json` (mode `0600`). AnkiWeb credentials are never persisted — only the session token.

## Claude Code Skill

The package ships a skill file that gives Claude full context about every command. After installing, register it as a global slash command:

```bash
cp $(anki-ai skill) ~/.claude/commands/anki.md
```

This makes `/anki` available in any Claude Code session. Re-run after upgrading to keep the skill in sync with the CLI version.

## All Commands

| Command | Description |
|---|---|
| `anki-ai auth login` | Authenticate with AnkiWeb |
| `anki-ai sync` | Bidirectional sync with AnkiWeb (media included by default) |
| `anki-ai decks list` | List all decks as JSON |
| `anki-ai decks create NAME` | Create a deck (`::` for nested, e.g. `Spanish::Verbs`) |
| `anki-ai decks delete NAME` | Delete a deck and all its cards |
| `anki-ai notes add --deck D --field K=V` | Add a note |
| `anki-ai notes get ID` | Get a note by ID |
| `anki-ai notes update ID --field K=V` | Update note fields |
| `anki-ai notes delete ID` | Delete a note |
| `anki-ai notes search QUERY` | Search notes, return JSON |
| `anki-ai cards list QUERY` | Find cards matching a query |
| `anki-ai cards info ID` | Show card scheduling info |
| `anki-ai cards suspend ID...` | Suspend cards |
| `anki-ai cards unsuspend ID...` | Unsuspend cards |
| `anki-ai tags list` | List all tags |
| `anki-ai tags add TAGS --query Q` | Add tags to matching notes |
| `anki-ai tags remove TAGS --query Q` | Remove tags from matching notes |
| `anki-ai tags rename OLD NEW` | Rename a tag across all notes |
| `anki-ai notetypes list` | List all note types |
| `anki-ai notetypes fields NAME` | List fields of a note type |
| `anki-ai media upload FILE...` | Copy files into the media folder |
| `anki-ai skill` | Print path to the installed Claude skill file |
