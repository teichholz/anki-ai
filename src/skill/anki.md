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
- **Snapshot before bulk operations.** Run `anki-ai snapshot` before any bulk add/update/delete session.
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
# Standard sync (collection + media):
anki-ai sync

# Skip media:
anki-ai sync --media=false

# Force full upload (e.g. after manual DB edits):
anki-ai sync --upload

# First-time sync or after schema change (download wins by default):
anki-ai sync
# Full download complete.
# Media sync complete.
```

**Full sync note:** `--upload` forces upload. Without it, download is preferred unless the server
cannot accept a download (in which case upload is used automatically).

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

```bash
anki-ai info
```

---

## Snapshots

### `anki-ai snapshot`

Create a timestamped snapshot of the collection file before making bulk changes.
Snapshots are stored in `~/.local/share/Anki2/<profile>/snapshots/`.

**Output shape:**
```json
{ "snapshot": "/home/user/.local/share/Anki2/User 1/snapshots/snapshot-2026-05-10-19.18.36.anki2" }
```

```bash
anki-ai snapshot
```

---

### `anki-ai snapshots`

List all available snapshots (newest first).

**Output shape:**
```json
[
  {
    "name": "snapshot-2026-05-10-19.18.36.anki2",
    "path": "/home/user/.local/share/Anki2/User 1/snapshots/snapshot-2026-05-10-19.18.36.anki2",
    "bytes": 10117120
  }
]
```

```bash
anki-ai snapshots
```

---

### `anki-ai restore SNAPSHOT`

Restore the collection from a snapshot, overwriting the current state.

**Argument:** bare filename (e.g. `snapshot-2026-05-10-19.18.36.anki2`) or full path.

**Flags:**
- `--yes` / `-y` — skip confirmation prompt (required for AI agents)

```bash
anki-ai restore snapshot-2026-05-10-19.18.36.anki2 --yes
```

**Rollback workflow:**
```bash
anki-ai snapshot                                           # before session
# ... make changes ...
anki-ai restore snapshot-2026-05-10-19.18.36.anki2 --yes  # rollback
anki-ai sync                                               # push rollback to AnkiWeb
```

---

## Decks

### `anki-ai decks list`

Print all decks as a JSON array. Nested deck names use `::` as separator.

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

---

### `anki-ai decks create NAME`

Create a deck. Returns the deck object (unchanged if name already exists).
Missing parent decks are created automatically.

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

Delete a deck and all its cards.

**Flags:**
- `--yes` / `-y` — skip confirmation (required for AI agents)

```bash
anki-ai decks delete "Spanish" --yes
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
anki-ai notes add --deck "Default" --type "Basic" --field Front="What is 2+2?" --field Back="4"

# Cloze — one sentence, one or more blanks; generates a card per {{cN::}} group:
anki-ai notes add --deck "Biology" --type "Cloze" \
  --field Text="The {{c1::mitochondria}} is the {{c2::powerhouse}} of the cell."
anki-ai notes add --deck "Spanish" --type "Cloze" \
  --field Text="{{c1::Haber}} is the auxiliary verb used to form the {{c2::present perfect}}."

# Cloze with extra hint shown on answer side:
anki-ai notes add --deck "Medicine" --type "Cloze" \
  --field Text="{{c1::Aspirin}} inhibits {{c2::COX-1 and COX-2}}." \
  --field "Back Extra"="NSAIDs mechanism"

# With media — upload first, then embed the returned filename:
anki-ai media upload /tmp/bark.mp3
# [{"filename": "bark.mp3"}]
anki-ai notes add --deck "Animals" --field Front="What sound does a dog make?" --field Back="[sound:bark.mp3]"
```

---

### `anki-ai notes get NOTE_ID`

Get a single note by ID.

**Output shape:**
```json
{
  "id": 1715001234567,
  "fields": {
    "Front": "rendir",
    "Back": "to yield"
  },
  "tags": ["verb", "irregular"]
}
```

```bash
anki-ai notes get 1715001234567
```

---

### `anki-ai notes search QUERY`

Run an Anki search query and return matching notes as a JSON array.

**Output shape:**
```json
[
  {
    "id": 1715001234567,
    "fields": { "Front": "rendir", "Back": "to yield" },
    "tags": ["verb", "irregular"]
  }
]
```

```bash
anki-ai notes search "deck:Spanish"          # all notes in a deck
anki-ai notes search "rendir"                # notes containing a word
anki-ai notes search "tag:irregular"         # notes with a tag
anki-ai notes search "deck:Spanish modified:7"  # modified in last 7 days
anki-ai notes search "nid:1715001234567"     # exact note by ID
anki-ai notes search ""                      # all notes
```

**Common search operators:**

| Syntax | Matches |
|---|---|
| `deck:Name` | Notes in deck (use `deck:Name::*` for sub-decks too) |
| `tag:name` | Notes with tag |
| `note:TypeName` | Notes of a note type |
| `is:due` | Cards due for review |
| `is:new` | Unseen cards |
| `is:suspended` | Suspended cards |
| `is:buried` | Buried cards |
| `modified:N` | Modified within last N days |
| `nid:ID` | Specific note ID |
| `cid:ID` | Specific card ID |
| `rated:N` | Reviewed within last N days |
| `"exact phrase"` | Exact phrase match |

---

### `anki-ai notes update NOTE_ID`

Update one or more fields of an existing note.

**Required flags:**
- `--field Name=Value` — field to update; repeat for each field (at least one required)

**Output shape:**
```json
{ "id": 1715001234567 }
```

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

## Cards

### `anki-ai cards list QUERY`

Find cards matching a search query. Returns card IDs and basic info.

**Output shape:**
```json
[
  { "id": 1715001234568, "note_id": 1715001234567, "deck_id": 1715000000000 }
]
```

```bash
anki-ai cards list "deck:Spanish"
anki-ai cards list "is:due"
anki-ai cards list "is:suspended"
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

```bash
anki-ai cards info 1715001234568
```

---

### `anki-ai cards suspend CARD_ID [CARD_ID ...]`

Suspend one or more cards by ID.

**Output shape:**
```json
{ "suspended": 2 }
```

```bash
anki-ai cards suspend 1715001234568
anki-ai cards suspend 1715001234568 1715001234569 1715001234570
```

---

### `anki-ai cards unsuspend CARD_ID [CARD_ID ...]`

Unsuspend one or more cards by ID.

**Output shape:**
```json
{ "unsuspended": 2 }
```

```bash
anki-ai cards unsuspend 1715001234568 1715001234569
```

---

## Tags

### `anki-ai tags list`

List all tags in the collection.

**Output shape:**
```json
["irregular", "phrase", "verb"]
```

```bash
anki-ai tags list
```

---

### `anki-ai tags add TAG [TAG ...] [-q QUERY]`

Add one or more tags to notes matching a search query.

**Flags:**
- `-q` / `--query TEXT` — Anki search query (default: `deck:current`)

**Output shape:**
```json
{ "updated": 12 }
```

```bash
anki-ai tags add irregular -q "deck:Spanish"
anki-ai tags add verb irregular -q "tag:spanish-verb"
```

---

### `anki-ai tags remove TAG [TAG ...] -q QUERY`

Remove one or more tags from notes matching a search query.

**Required flags:**
- `-q` / `--query TEXT` — Anki search query (no default; must be specified)

**Output shape:**
```json
{ "updated": 5 }
```

```bash
anki-ai tags remove irregular -q "deck:Spanish"
```

---

### `anki-ai tags rename OLD NEW`

Rename a tag across all notes in the collection.

**Output shape:**
```json
{ "updated": 8 }
```

```bash
anki-ai tags rename "irregular" "verb-irregular"
```

---

## Note Types

### `anki-ai notetypes list`

List all note types with their note counts.

**Output shape:**
```json
[
  { "name": "Basic", "notes": 150 },
  { "name": "Basic (and reversed card)", "notes": 30 },
  { "name": "Cloze", "notes": 45 }
]
```

```bash
anki-ai notetypes list
```

---

### `anki-ai notetypes fields NAME`

List the field names of a note type. Use this to know which `--field` names to pass to `notes add`.

**Output shape:**
```json
["Front", "Back"]
```

```bash
anki-ai notetypes fields "Basic"
anki-ai notetypes fields "Cloze"
```

---

## Media

### `anki-ai media upload FILE [FILE ...]`

Copy one or more local files into the collection's media folder (`collection.media/`).
Returns the stored filename for each file.

- **Images:** `<img src="filename.jpg">`
- **Audio:** `[sound:filename.mp3]`

Media is included in `anki-ai sync` by default.

**Output shape:**
```json
[{ "filename": "bark.mp3" }]
```

```bash
anki-ai media upload /path/to/bark.mp3
anki-ai media upload photo.jpg audio.mp3

# Full workflow:
anki-ai media upload /tmp/bark.mp3
anki-ai notes add --deck "Animals" \
  --field Front="What sound does a dog make?" \
  --field Back="[sound:bark.mp3]"
anki-ai sync
```

---

## Typical Workflows

### Initial setup (once)
```bash
# Via flags:
anki-ai auth login --email user@example.com --password s3cr3t

# Or via env vars (CI / containers):
ANKI_EMAIL=user@example.com ANKI_PASSWORD=s3cr3t anki-ai auth login

anki-ai sync
```

### Add Basic cards (agent session)
```bash
anki-ai sync                                        # pull latest from phone
anki-ai snapshot                                    # safety checkpoint
anki-ai decks list                                  # check deck names
anki-ai notetypes fields "Basic"                    # fields: Front, Back
anki-ai notes add --deck "Spanish" --field Front="rendir" --field Back="to yield"
anki-ai sync                                        # push new card to AnkiWeb
anki-ai notes search "rendir"                       # verify
```

### Add Cloze cards (agent session)
Use Cloze when the fact lives inside a sentence and context helps recall.
Each `{{cN::answer}}` group produces one card; multiple groups in one note = multiple cards.
```bash
anki-ai sync
anki-ai snapshot
anki-ai notetypes fields "Cloze"                    # fields: Text, Back Extra
anki-ai notes add --deck "Biology" --type "Cloze" \
  --field Text="The {{c1::mitochondria}} produces {{c2::ATP}} via oxidative phosphorylation."
anki-ai notes add --deck "History" --type "Cloze" \
  --field Text="{{c1::World War II}} ended in {{c2::1945}}." \
  --field "Back Extra"="VE Day: May 8; VJ Day: Sep 2"
anki-ai sync
anki-ai notes search "deck:Biology note:Cloze"      # verify
```

### Bulk tag operation
```bash
anki-ai sync
anki-ai snapshot
anki-ai tags add verb -q "deck:Spanish"
anki-ai sync
```

### Suspend low-quality cards
```bash
anki-ai cards list "deck:Spanish tag:low-quality" | jq '.[].id'
anki-ai cards suspend 1715001234568 1715001234569
anki-ai sync
```

---

## Error Handling

- Non-zero exit code on any error.
- Error message on stderr.

| Error message | Cause and fix |
|---|---|
| `Not authenticated. Run 'anki-ai auth login' first.` | Run `anki-ai auth login --email … --password …` |
| `Deck 'Name' not found.` | Use `anki-ai decks list` to check exact name |
| `Note type 'Name' not found.` | Use `anki-ai notetypes list` to check exact name |
| `Field 'Name' not found in note type 'Type'.` | Use `anki-ai notetypes fields <type>` |
| `auth file not found: …` | Run `anki-ai auth login` first |

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `ANKI_COLLECTION_PATH` | Auto-detected from `~/.local/share/Anki2/` | Absolute path to `collection.anki2` |
| `ANKI_AUTH_PATH` | `~/.config/anki-ai/auth.json` | Path to auth token file (useful for isolated environments) |
| `ANKI_EMAIL` | — | AnkiWeb email for `auth login` (overridden by `--email` flag) |
| `ANKI_PASSWORD` | — | AnkiWeb password for `auth login` (overridden by `--password` flag) |
