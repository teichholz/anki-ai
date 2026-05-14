# anki-ai — Claude Skill

Use this file as context when generating `anki-ai` CLI invocations.

## Overview

`anki-ai` is a headless CLI for managing Anki flashcard decks, designed for use by AI agents.
It syncs bidirectionally with AnkiWeb so cards created on a mobile device are available server-side.
All data commands write JSON to stdout. Errors go to stderr with a non-zero exit code.

---

## AI Agent Guidelines

- **Never run interactive commands.** Use `--email`/`--password` flags for auth; use `--yes` for all destructive commands.
- **Always sync before reading** if freshness matters; **always sync after writing** to push changes to AnkiWeb.
- **Snapshot before bulk operations.** Run `anki-ai snapshot` before any bulk add/update/delete/move/find-replace session.
- **Use `restore --last` to undo.** There is no undo command — `anki-ai restore --last --yes` restores from the most recent snapshot.
- **Auth is persistent.** After one successful `auth login`, the hkey is stored at `~/.config/anki-ai/auth.json` (mode 0600). No need to re-auth between commands.
- **Full sync exits cleanly.** When a full sync is required, the binary completes the upload or download and exits with code 0. The next command re-opens the collection normally — no special handling needed.

---

## Auth Flow

Authentication is a one-time setup. The hkey (host key) is stored at `~/.config/anki-ai/auth.json`
(permissions 0600) and loaded automatically by every command that needs it.

### `anki-ai auth login`

Exchange AnkiWeb credentials for an hkey and store it.

**Resolution order for email and password (first wins):**
1. `--email` / `--password` CLI flags
2. `ANKI_EMAIL` / `ANKI_PASSWORD` environment variables
3. Interactive prompt (blocks — not usable by AI agents)

**AI agents must use flags or env vars** — interactive prompts block indefinitely.

```bash
# Via CLI flags:
anki-ai auth login --email user@example.com --password s3cr3t

# Via environment variables (preferred for CI / agent containers):
export ANKI_EMAIL=user@example.com
export ANKI_PASSWORD=s3cr3t
anki-ai auth login

# Interactive (human use only):
anki-ai auth login
# AnkiWeb email: user@example.com
# AnkiWeb password: ••••••••
# Logged in successfully.
```

---

## Sync

### `anki-ai sync`

Pull changes from AnkiWeb and push local changes back. Performs a bidirectional delta sync by default.
When server and client schemas diverge, a full sync is triggered automatically.

**Flags:**
- `--media=false` — skip media sync (default: media is synced)
- `--upload` — on full sync, upload local collection to server (default: download from server)

**Output (one of the following, then optionally "Media sync complete."):**

| Message | Meaning |
|---|---|
| `Already up to date.` | No changes on either side |
| `Sync complete.` | Delta sync succeeded |
| `Full download complete.` | Full sync — server replaced local collection |
| `Full upload complete.` | Full sync — local replaced server collection |
| `Media sync complete.` | Media phase done (printed after collection sync when `--media` is active) |

```bash
anki-ai sync               # standard sync (collection + media)
anki-ai sync --media=false # skip media
anki-ai sync --upload      # force full upload
```

---

## Collection Info

### `anki-ai info`

Show collection statistics.

**Output shape:**
```json
{
  "path": "/home/user/.local/share/Anki2/User 1/collection.anki2",
  "notes": 1234,
  "cards": 1456
}
```

---

## Snapshots & Undo

### `anki-ai snapshot`

Create a timestamped snapshot of the collection file. Always run before any bulk or destructive operation.

**Output shape:**
```json
{ "snapshot": "/home/user/.local/share/Anki2/User 1/snapshots/snapshot-2026-05-14-10.30.00.anki2" }
```

---

### `anki-ai snapshots`

List all available snapshots (newest first).

**Output shape:**
```json
[
  {
    "name": "snapshot-2026-05-14-10.30.00.anki2",
    "path": "/home/user/.local/share/Anki2/User 1/snapshots/snapshot-2026-05-14-10.30.00.anki2",
    "bytes": 10117120
  }
]
```

---

### `anki-ai restore [SNAPSHOT] [--last]`

Restore the collection from a snapshot, overwriting the current state.

**Arguments / flags:**
- `SNAPSHOT` — bare filename or full path (mutually exclusive with `--last`)
- `--last` — automatically select the most recent snapshot (agent undo shorthand)
- `--yes` / `-y` — skip confirmation prompt (required for AI agents)

```bash
# Undo: restore the most recent snapshot (preferred agent pattern)
anki-ai restore --last --yes

# Restore a specific snapshot by name
anki-ai restore snapshot-2026-05-14-10.30.00.anki2 --yes
```

**Standard rollback workflow:**
```bash
anki-ai snapshot          # checkpoint before session
# ... make changes ...
anki-ai restore --last --yes   # something went wrong — revert
anki-ai sync              # push rollback to AnkiWeb
```

---

## Decks

### `anki-ai decks list`

Print all decks as a JSON array with today's due card counts.

**Output shape:**
```json
[
  { "id": 1, "name": "Default", "new": 0, "learning": 0, "review": 0 },
  { "id": 1715000000000, "name": "Spanish", "new": 5, "learning": 2, "review": 10 },
  { "id": 1715000000001, "name": "Spanish::Verbs", "new": 3, "learning": 1, "review": 4 }
]
```

---

### `anki-ai decks create NAME`

Create a deck. Returns the deck object (unchanged if name already exists).
Missing parent decks are created automatically. Use `::` for nesting.

**Output shape:**
```json
{ "id": 1715000000002, "name": "Languages::Spanish::Verbs", "new": 0, "learning": 0, "review": 0 }
```

```bash
anki-ai decks create "Spanish"
anki-ai decks create "Languages::Spanish::Verbs"
```

---

### `anki-ai decks delete NAME`

Delete a deck and all its notes and cards. Child decks are also deleted.

**Flags:**
- `--yes` / `-y` — skip confirmation (required for AI agents)

```bash
anki-ai decks delete "Spanish" --yes
```

---

### `anki-ai decks rename OLD NEW`

Rename a deck. Child decks follow automatically (`Old::Child` → `New::Child`).

**Output shape:** `DeckInfo` (see `decks list`)

```bash
anki-ai decks rename "Spanish" "Español"
anki-ai decks rename "Languages::Spanish" "Languages::Español"
```

---

### `anki-ai decks reparent DECK [--parent PARENT | --root]`

Move a deck to a different parent without changing its leaf name. Child decks follow automatically.

**Flags:**
- `--parent NAME` — new parent deck (created if absent; mutually exclusive with `--root`)
- `--root` — promote to top level (mutually exclusive with `--parent`)

**Output shape:** `DeckInfo` with updated name

```bash
# Move "N5" under "Japanese"  →  "Japanese::N5"
anki-ai decks reparent "N5" --parent "Japanese"

# Promote "Japanese::N5" to top level  →  "N5"
anki-ai decks reparent "Japanese::N5" --root
```

---

### `anki-ai decks config get NAME`

Show study limits for a deck.

**Output shape:**
```json
{ "config_id": 1, "config_name": "Default", "new_per_day": 20, "reviews_per_day": 200 }
```

> **Note:** Deck configs are shared. Multiple decks can share the same `config_id`. Changing one config changes it for all decks that use it — the same as the Anki GUI behaviour.

```bash
anki-ai decks config get "Japanese"
```

---

### `anki-ai decks config set NAME [--new-per-day N] [--reviews-per-day N]`

Update study limits for a deck. Omit a flag to leave that value unchanged.

**Output shape:** same as `decks config get`

```bash
anki-ai decks config set "Japanese" --new-per-day 10
anki-ai decks config set "Japanese" --new-per-day 10 --reviews-per-day 100
```

---

## Notes

### `anki-ai notes add`

Add a note to a named deck.

**Required flags:**
- `--deck TEXT` — deck name (exact, case-sensitive)
- `--field Name=Value` — field content; repeat for each field (at least one required)

**Optional flags:**
- `--type TEXT` — note type name (default: `Basic`)

**Output shape:**
```json
{ "id": 1715001234567 }
```

```bash
# Basic — explicit Q/A pair:
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"

# Cloze — blanks via {{cN::answer}}; each group generates one card:
anki-ai notes add --deck "Biology" --type "Cloze" \
  --field Text="The {{c1::mitochondria}} produces {{c2::ATP}} via oxidative phosphorylation."

# With media — upload first, embed the returned filename:
anki-ai media upload /tmp/bark.mp3
anki-ai notes add --deck "Animals" \
  --field Front="What sound does a dog make?" \
  --field Back="[sound:bark.mp3]"
```

---

### `anki-ai notes get NOTE_ID`

Get a single note by ID.

**Output shape:**
```json
{
  "id": 1715001234567,
  "type": "Basic",
  "fields": { "Front": "rendir", "Back": "to yield" },
  "tags": ["verb", "irregular"]
}
```

---

### `anki-ai notes search QUERY`

Run an Anki search query and return matching notes as a JSON array. Returns `[]` when nothing matches.

**Output shape:** array of note objects (same shape as `notes get`)

```bash
anki-ai notes search "deck:Spanish"
anki-ai notes search "tag:irregular"
anki-ai notes search "deck:Spanish modified:7"
anki-ai notes search ""                            # all notes
```

**Common search operators:**

| Syntax | Matches |
|---|---|
| `deck:Name` | Notes in deck (exact); `deck:Name*` includes sub-decks |
| `tag:name` | Notes with tag |
| `note:TypeName` | Notes of a note type |
| `Front:word` | Notes where the Front field contains "word" |
| `is:due` | Cards due for review |
| `is:new` | Unseen cards |
| `is:suspended` | Suspended cards |
| `is:buried` | Buried cards |
| `modified:N` | Modified within last N days |
| `added:N` | Added within last N days |
| `nid:ID` | Specific note ID |
| `cid:ID` | Specific card ID |
| `rated:N` | Reviewed within last N days |
| `-tag:name` | Notes NOT tagged "name" (negate with `-`) |
| `deck:A OR deck:B` | Notes in A or B |

---

### `anki-ai notes update NOTE_ID`

Update one or more fields of an existing note. Only specified fields change.

**Required flags:**
- `--field Name=Value` — field to update; repeat for each field

**Output shape:** full `NoteInfo` (same as `notes get`)

```bash
anki-ai notes update 1715001234567 --field Back="to surrender / to yield"
anki-ai notes update 1715001234567 --field Front="rendir (v.)" --field Back="to yield"
```

---

### `anki-ai notes delete NOTE_ID`

Delete a note by ID (also deletes all its cards).

**Flags:**
- `--yes` / `-y` — skip confirmation (required for AI agents)

```bash
anki-ai notes delete 1715001234567 --yes
```

---

### `anki-ai notes move --deck NAME NOTE_ID [NOTE_ID ...]`

Move one or more notes to a different deck. Moves all cards of each note.

**Output shape:**
```json
{ "moved": 3 }
```
The count is cards moved, not notes (a note with 2 templates contributes 2 cards).

```bash
anki-ai notes move --deck "Japanese::N5" 1715001234567 1715001234568
```

---

### `anki-ai notes find-replace PATTERN REPLACEMENT [--field NAME] [-q QUERY]`

Bulk regex find/replace across note fields. `PATTERN` is a Rust regex.

**Flags:**
- `--field NAME` — restrict to one field; omit for all fields
- `-q QUERY` — Anki search query to scope notes (default: all notes)

**Output shape:**
```json
{ "updated": 12 }
```

```bash
# Replace across all notes, all fields
anki-ai notes find-replace "colour" "color"

# Case-insensitive, scoped to one field and one deck
anki-ai notes find-replace "(?i)grey" "gray" --field Back -q "deck:English"

# Fix a typo only in the Front field
anki-ai notes find-replace "teh" "the" --field Front
```

---

## Cards

### `anki-ai cards list QUERY`

Find cards matching a search query. Returns card IDs and basic info.

**Output shape:**
```json
[{ "id": 1715001234568, "note_id": 1715001234567, "deck_id": 1715000000000 }]
```

---

### `anki-ai cards info CARD_ID`

Show scheduling info for a card.

**Output shape:**
```json
{
  "id": 1715001234568,
  "note_id": 1715001234567,
  "deck_id": 1715000000000,
  "due": 100,
  "interval": 10,
  "ease_factor": 2500,
  "reps": 5,
  "lapses": 0,
  "suspended": false
}
```

---

### `anki-ai cards suspend CARD_ID [CARD_ID ...]`

Suspend one or more cards by ID.

**Output shape:** `{ "suspended": 2 }`

```bash
anki-ai cards suspend 1715001234568 1715001234569
```

---

### `anki-ai cards unsuspend CARD_ID [CARD_ID ...]`

Unsuspend one or more cards by ID.

**Output shape:** `{ "unsuspended": 2 }`

---

## Tags

### `anki-ai tags list`

List all tags in the collection as a sorted JSON array.

**Output shape:** `["irregular", "phrase", "verb"]`

---

### `anki-ai tags add TAG [TAG ...] [-q QUERY]`

Add one or more tags to notes matching a search query.

**Flags:** `-q` / `--query TEXT` — Anki search query (default: `deck:current`)

**Output shape:** `{ "updated": 12 }`

```bash
anki-ai tags add irregular -q "deck:Spanish"
anki-ai tags add verb irregular -q "tag:spanish-verb"
```

---

### `anki-ai tags remove TAG [TAG ...] -q QUERY`

Remove one or more tags from notes matching a search query. `-q` is required.

**Output shape:** `{ "updated": 5 }`

---

### `anki-ai tags rename OLD NEW`

Rename a tag across all notes in the collection.

**Output shape:** `{ "updated": 8 }`

---

## Note Types

### `anki-ai notetypes list`

List all note types with their note counts.

**Output shape:**
```json
[
  { "name": "Basic", "notes": 150 },
  { "name": "Cloze", "notes": 45 }
]
```

---

### `anki-ai notetypes fields NAME`

List the field names of a note type. Use before `notes add` to know which `--field` names to pass.

**Output shape:** `["Front", "Back"]`

```bash
anki-ai notetypes fields "Basic"
anki-ai notetypes fields "Cloze"    # ["Text", "Back Extra"]
```

---

## Media

### `anki-ai media upload FILE [FILE ...]`

Copy local files into the collection media folder. Returns the stored filename for each file.

Reference in note fields as: `<img src="file.jpg">` or `[sound:file.mp3]`

**Output shape:** `[{ "filename": "bark.mp3" }]`

```bash
anki-ai media upload /path/to/bark.mp3
```

---

## Typical Workflows

### Safe session pattern (always use this)
```bash
anki-ai sync               # pull latest state
anki-ai snapshot           # checkpoint before any writes
# ... do work ...
anki-ai sync               # push changes to AnkiWeb
```

### Undo a mistake
```bash
anki-ai restore --last --yes   # revert to last snapshot
anki-ai sync                   # push rollback to AnkiWeb
```

### Restructure deck hierarchy
```bash
anki-ai sync
anki-ai snapshot
anki-ai decks reparent "N5" --parent "Japanese"
anki-ai decks reparent "N4" --parent "Japanese"
anki-ai decks list            # verify
anki-ai sync
```

### Bulk field fix with find-replace
```bash
anki-ai sync
anki-ai snapshot
anki-ai notes find-replace "(?i)colour" "color" --field Back -q "deck:English"
anki-ai notes search "color" -q "deck:English"   # verify
anki-ai sync
```

### Move notes between decks
```bash
anki-ai notes search "deck:Japanese tag:n5" | jq '.[].id'
anki-ai notes move --deck "Japanese::N5" 1715001234567 1715001234568
anki-ai decks list            # verify due counts updated
```

### Add Basic cards
```bash
anki-ai sync
anki-ai snapshot
anki-ai notetypes fields "Basic"                  # confirm field names
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"
anki-ai sync
anki-ai notes search "rendir"                     # verify
```

### Set study limits
```bash
anki-ai decks config get "Japanese"               # see current limits
anki-ai decks config set "Japanese" --new-per-day 10 --reviews-per-day 150
```

---

## Error Handling

- Non-zero exit code on any error. Error message on stderr.

| Error message | Cause and fix |
|---|---|
| `Not authenticated. Run 'anki-ai auth login' first.` | Run `anki-ai auth login --email … --password …` |
| `Deck 'Name' not found.` | Use `anki-ai decks list` to check exact name |
| `Note type 'Name' not found.` | Use `anki-ai notetypes list` to check exact name |
| `Field 'Name' not found in note type 'Type'.` | Use `anki-ai notetypes fields <type>` |
| `No snapshots found.` | Run `anki-ai snapshot` first before using `restore --last` |

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `ANKI_COLLECTION_PATH` | Auto-detected from `~/.local/share/Anki2/` | Absolute path to `collection.anki2` |
| `ANKI_AUTH_PATH` | `~/.config/anki-ai/auth.json` | Path to auth token file |
| `ANKI_EMAIL` | — | AnkiWeb email for `auth login` (overridden by `--email` flag) |
| `ANKI_PASSWORD` | — | AnkiWeb password for `auth login` (overridden by `--password` flag) |
