# anki-ai — Claude Skill

Use this file as context when generating `anki-ai` CLI invocations.

## Overview

`anki-ai` is a headless CLI for managing Anki flashcard decks on an Ubuntu server.
It syncs bidirectionally with AnkiWeb so cards created on a mobile device are available server-side.
All data commands write JSON to stdout. Errors go to stderr with a non-zero exit code.

---

## Commands

### `anki-ai auth login`

Exchange AnkiWeb email and password for an auth token (hkey) and store it at
`~/.config/anki-ai/auth.json`. Must be run once before any other commands.

**Prompts interactively:** email, then password (hidden).

```bash
anki-ai auth login
# AnkiWeb email: user@example.com
# AnkiWeb password: ••••••••
# Logged in successfully.
```

---

### `anki-ai sync`

Pull changes from AnkiWeb and push local changes back. Performs a full bidirectional sync.

**Flags:**
- `--media` / `--no-media` — include media files in sync (default: `--no-media`)
- `--upload` / `--download` — direction for a full sync when one is required (default: `--download`)

```bash
anki-ai sync
# Sync complete.

anki-ai sync --media
# Sync complete.

# First-time or post-schema-change:
anki-ai sync --download
# Full sync required — performing full download.
# Full download complete.
```

---

### `anki-ai decks list`

Print all decks as a JSON array.

**Output shape:**
```json
[
  { "id": 1, "name": "Default" },
  { "id": 1715000000000, "name": "Spanish" },
  { "id": 1715000000001, "name": "Spanish::Verbs" }
]
```

```bash
anki-ai decks list
```

Nested deck names use `::` as a separator (Anki convention).

---

### `anki-ai notes add`

Add a Basic (Front/Back) note to a named deck. The deck must already exist.

**Required flags:**
- `--deck TEXT` — deck name (exact match, case-sensitive)
- `--front TEXT` — front field content
- `--back TEXT` — back field content

**Output shape:**
```json
{ "id": 1715001234567 }
```

```bash
anki-ai notes add --deck "Spanish" --front "rendir" --back "to yield"
# {"id": 1715001234567}

anki-ai notes add --deck "Default" --front "What is 2+2?" --back "4"
```

---

### `anki-ai notes search QUERY`

Run an Anki search query and return matching notes as a JSON array.

**Argument:** `QUERY` — Anki search string (same syntax as the desktop app)

**Output shape:**
```json
[
  {
    "id": 1715001234567,
    "fields": {
      "Front": "rendir",
      "Back": "to yield"
    },
    "tags": ["verb", "irregular"]
  }
]
```

```bash
# All notes in a deck
anki-ai notes search "deck:Spanish"

# Notes containing a word
anki-ai notes search "rendir"

# Notes with a tag
anki-ai notes search "tag:irregular"

# Notes modified in the last 7 days
anki-ai notes search "deck:Spanish modified:7"

# All notes
anki-ai notes search ""
```

---

## Typical Workflow

```bash
# 1. Authenticate once
anki-ai auth login

# 2. Pull cards from phone
anki-ai sync

# 3. See what decks exist
anki-ai decks list

# 4. Add a new card
anki-ai notes add --deck "Spanish" --front "rendir" --back "to yield"

# 5. Push the new card back
anki-ai sync

# 6. Verify the card is there
anki-ai notes search "rendir"
```

---

## Error Handling

- Non-zero exit code on any error.
- Error message on stderr.
- If not authenticated: `Not authenticated. Run 'anki-ai auth login' first.`
- If deck not found: `Deck 'Name' not found.`
- If collection not found: set `ANKI_COLLECTION_PATH` env var.

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `ANKI_COLLECTION_PATH` | Auto-detected from `~/.local/share/Anki2/` | Absolute path to `collection.anki2` |
