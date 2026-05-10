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
- `--media` / `--no-media` — include media files in sync (default: `--media`)
- `--upload` / `--download` — direction for a full sync when one is required (default: `--download`)

```bash
anki-ai sync
# Sync complete. (media included by default)

anki-ai sync --no-media
# Sync complete. (skips media)

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

### `anki-ai decks create NAME`

Create a deck. Returns the deck object (existing deck returned unchanged if the name is already taken).

Anki supports nested decks using `::` as a separator. Any missing parent decks are created automatically.

**Output shape:**
```json
{ "id": 1715000000002, "name": "Languages::Spanish::Verbs" }
```

```bash
anki-ai decks create "Spanish"
anki-ai decks create "Languages::Spanish::Verbs"
```

---

### `anki-ai decks delete NAME`

Delete a deck and all its cards. Prompts for confirmation unless `--yes` / `-y` is passed.

```bash
anki-ai decks delete "Spanish"
anki-ai decks delete "Spanish" --yes
```

---

### `anki-ai notes add`

Add a note to a named deck. The deck must already exist.

**Required flags:**
- `--deck TEXT` — deck name (exact match, case-sensitive)
- `--field Name=Value` — field content; repeat for each field (at least one required)

**Optional flags:**
- `--type TEXT` — note type name (default: `Basic`)

**Output shape:**
```json
{ "id": 1715001234567 }
```

```bash
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"
# {"id": 1715001234567}

anki-ai notes add --deck "Default" --type "Basic" --field Front="What is 2+2?" --field Back="4"

# With media — upload first, then embed the returned filename:
anki-ai media upload bark.mp3
# [{"filename": "bark.mp3"}]
anki-ai notes add --deck "Animals" --field Front="What sound does a dog make?" --field Back="[sound:bark.mp3]"
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

### `anki-ai media upload FILE [FILE ...]`

Copy one or more local files into the collection's media folder (`collection.media/`).
Returns the stored filename for each file. Use the filename to embed media in note fields.

- **Images:** `<img src="filename.jpg">`
- **Audio:** `[sound:filename.mp3]`

Media is included in `anki-ai sync` by default (no extra flags needed).

**Output shape:**
```json
[{ "filename": "bark.mp3" }]
```

```bash
# Upload a single audio file
anki-ai media upload /path/to/bark.mp3
# [{"filename": "bark.mp3"}]

# Upload multiple files at once
anki-ai media upload photo.jpg audio.mp3

# Full workflow: upload then add note referencing the media
anki-ai media upload /tmp/bark.mp3
anki-ai notes add --deck "Animals" \
  --field Front="What sound does a dog make?" \
  --field Back="[sound:bark.mp3]"
anki-ai sync
```

---

## Typical Workflow

```bash
# 1. Authenticate once
anki-ai auth login

# 2. Pull cards from phone (media included by default)
anki-ai sync

# 3. See what decks exist
anki-ai decks list

# 4. Add a new card
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"

# 5. Push the new card back (media included by default)
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
