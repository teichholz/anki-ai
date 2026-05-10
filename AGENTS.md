# Anki AI Manager — Agent Instructions

## Goal

Build a headless CLI tool that lets an AI agent manage Anki flashcard decks on an Ubuntu server,
syncing bidirectionally with AnkiWeb so cards created on a mobile device are available server-side.

---

## Hard Requirements

- **No GUI, no display.** Runs on a headless Ubuntu server. AnkiConnect is explicitly ruled out.
- **AnkiWeb sync** must work in both directions. Cards created on a phone must be pullable.
- **Data safety first.** The Anki collection is a SQLite file. Corruption from a missed `col.close()`
  or concurrent access is a real risk. Every code path must close the collection cleanly.
- **Pinned `anki` package version.** The collection schema is version-coupled to the library.
  Pick one version, pin it in `pyproject.toml`, and document it. Do not let it float.
- **uv native** Must use uv and later be installable via uv

---

## Architecture Decisions (already made — do not revisit)

- Use the `anki` Python package directly, not AnkiConnect.
- CLI wrapper built with `typer`.
- Auth token (`hkey`) stored in `~/.config/anki-ai/auth.json` with `0o600` permissions,
  never in env vars or config files committed to the repo.
- Sync pattern for all mutating operations: **sync → mutate → sync**.
  Treat the server as a pass-through, not an independent source of truth.
- Media sync is a separate optional flag (`--media / --no-media`), defaulting to `--no-media` for the POC.
- Collection path is configurable via `ANKI_COLLECTION_PATH` env var, with a sensible default
  for a standard Anki Linux install (`~/.local/share/Anki2/<profile>/collection.anki2`).

---

## Project Structure

```
anki-ai/
├── anki_ai/
│   ├── __init__.py
│   ├── collection.py      # Context manager wrapping Collection open/close lifecycle
│   ├── sync.py            # AnkiWeb auth: login (exchange credentials → hkey), load/save token, run sync
│   ├── notes.py           # Add note, search notes (returns structured dicts), delete note by id
│   ├── decks.py           # List decks (id + name), create deck
│   └── cli.py             # Click group wiring all commands
├── skill/
│   └── anki.md            # Claude skill file: describes every CLI command for AI consumption
├── tests/
│   ├── test_collection.py
│   ├── test_notes.py
│   └── test_decks.py
├── pyproject.toml
├── .env.example
└── README.md
```

---

## POC Scope

Implement exactly these CLI commands, nothing more:

| Command | Description |
|---|---|
| `anki-ai auth login` | Exchange AnkiWeb email + password for an hkey, store it |
| `anki-ai sync` | Pull from AnkiWeb (full sync cycle) |
| `anki-ai decks list` | Print all deck names and IDs as JSON |
| `anki-ai notes add` | Add a Basic note to a named deck (front + back as CLI args) |
| `anki-ai notes search <query>` | Run an Anki search query, return matching notes as JSON |

Output format for all data commands: JSON to stdout. Errors to stderr with non-zero exit code.

---

## Key Implementation Notes

### Collection lifecycle

Always use a context manager — never open and forget:

```python
from contextlib import contextmanager
from anki.collection import Collection

@contextmanager
def open_collection(path: str):
    col = Collection(path)
    try:
        yield col
    finally:
        col.close()
```

### Sync

The sync API changed significantly around Anki 2.1.50. Target the current stable release.
`sync_collection` returns a `SyncOutput` with a `required` field that may indicate
a full sync is needed on first run or after schema changes — handle this explicitly,
do not silently ignore it.

```python
from anki.sync import SyncAuth

auth = SyncAuth(hkey=load_hkey(), endpoint=None)  # endpoint=None uses AnkiWeb
output = col.sync_collection(auth, sync_media=False)
# check output.required and handle SyncCollectionResponse.ChangesRequired.FULL_SYNC
```

### Auth login flow

AnkiWeb credentials should never be stored. Exchange them once:

```python
# Use col.sync_login(username, password) to get the hkey
# Then persist only the hkey to ~/.config/anki-ai/auth.json
```

### Note model

For the POC, target the built-in `Basic` note type only (front/back).
Retrieve the model with `col.models.by_name("Basic")`.

---

## pyproject.toml Requirements

- Python >= 3.10
- Dependencies: `anki` (pin to a specific version after checking PyPI current stable),
  `click>=8.0`
- Dev dependencies: `pytest`, `pytest-mock`
- Entry point: `anki-ai = "anki_ai.cli:cli"`

---

## Skill File (`skill/anki.md`)

Generate this file as a Claude skill describing the CLI interface — not the Python internals.
It should document every command with its flags, example invocations, and expected JSON output shape.
This file is what an AI agent will be given as context when generating `anki-ai` invocations.

---

## What Success Looks Like for the POC

Running this sequence on a fresh Ubuntu machine (with Anki previously used on a phone) works end-to-end:

```bash
anki-ai auth login                          # stores hkey
anki-ai sync                                # pulls phone cards
anki-ai decks list                          # shows decks including phone-created ones
anki-ai notes add --deck "Spanish" \
  --front "rendir" --back "to yield"        # adds a card
anki-ai sync                                # pushes new card back to AnkiWeb
```
