# Anki Backend Capabilities Audit

## Overview
This document catalogs every public API method available on the `Collection` type and related objects in the Anki Rust backend (`rslib`). This inventory is used to identify which capabilities are currently exposed via the CLI and which high-value features remain unexposed.

**Audit Date:** May 14, 2026  
**Backend Version:** From anki-9c9b125b7236058d/d52ca66  
**CLI Version:** anki-cli @ HEAD

---

## Decks

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `get_or_create_normal_deck(name: &str) -> Result<Deck>` | Fetch or create a deck by human-readable name; auto-creates if missing | ✅ Yes (create) |
| `get_deck(did: DeckId) -> Result<Option<Arc<Deck>>>` | Fetch a single deck by ID (cached) | ❌ No |
| `get_deck_id(human_name: &str) -> Result<Option<DeckId>>` | Get deck ID from human-readable name | ❌ No |
| `add_deck(deck: &mut Deck) -> Result<OpOutput<()>>` | Add a new normal deck (requires ID=0) | ✅ Partial (via create) |
| `update_deck(deck: &Deck) -> Result<OpOutput<()>>` | Update existing deck (mtime, conf, description) | ❌ No |
| `add_or_update_deck(deck: &mut Deck) -> Result<OpOutput<()>>` | Upsert deck | ❌ No |
| `remove_decks_and_child_decks(dids: &[DeckId]) -> Result<OpOutput<usize>>` | Delete decks and all their cards | ✅ Yes |
| `rename_deck(did: DeckId, name: &str) -> Result<OpOutput<()>>` | Rename a deck (updates child names too) | ✅ Yes |
| `reparent_decks(ids: &[DeckId], parent: Option<DeckId>) -> Result<OpOutput<usize>>` | Move decks to new parent | ❌ No |
| `set_current_deck(did: DeckId) -> Result<OpOutput<()>>` | Set the active deck for study | ❌ No |
| `get_current_deck() -> Result<Arc<Deck>>` | Fetch the current active deck | ❌ No |
| `deck_tree(timestamp: Option<TimestampSecs>) -> Result<DeckTreeNode>` | Get full deck hierarchy with due counts | ✅ Yes (list) |
| `current_deck_tree(timestamp: Option<TimestampSecs>) -> Result<DeckTreeNode>` | Get tree rooted at current deck | ❌ No |
| `get_deck_in_tree(id: DeckId) -> Result<Option<DeckTreeNode>>` | Locate a single deck node in tree | ❌ No |
| `set_deck_collapsed(did: DeckId, collapsed: bool) -> Result<OpOutput<()>>` | Toggle deck collapse state in browser/study | ❌ No |
| `get_all_normal_deck_names(skip_default: bool) -> Result<Vec<(DeckId, String)>>` | List all normal (non-filtered) deck names | ❌ No |
| `get_all_deck_names() -> Result<Vec<(DeckId, String)>>` | List all decks (normal + filtered) | ❌ No |
| `get_deck_and_child_names(did: DeckId) -> Result<Vec<(DeckId, String)>>` | List a deck and all its children | ❌ No |
| `current_review_limit() -> Result<u32>` | Cards allowed today in current deck (review) | ❌ No |
| `current_new_limit() -> Result<u32>` | Cards allowed today in current deck (new) | ❌ No |
| `review_limit_today(did: DeckId) -> Result<u32>` | Review limit for specific deck | ❌ No |
| `new_limit_today(did: DeckId) -> Result<u32>` | New limit for specific deck | ❌ No |

---

## Notes

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `add_note(note: &mut Note, did: DeckId) -> Result<OpOutput<usize>>` | Insert a new note (assigns ID, generates cards) | ✅ Yes |
| `add_notes(requests: &mut [AddNoteRequest]) -> Result<OpOutput<()>>` | Batch insert multiple notes | ❌ No (would improve perf) |
| `remove_notes(nids: &[NoteId]) -> Result<OpOutput<usize>>` | Delete notes and orphaned cards | ✅ Yes |
| `update_note(note: &mut Note) -> Result<OpOutput<usize>>` | Modify existing note content (mtime, fields, tags) | ✅ Yes |
| `after_note_updates(nids: &[NoteId], gen_cards: bool, mark_mod: bool) -> Result<OpOutput<usize>>` | Trigger card regeneration after external note edits | ❌ No |
| `note_fields_check(note: &Note) -> Result<NoteFieldsCheckResponse>` | Validate note against template (empty card detection) | ❌ No |
| `new_note(notetype: &Notetype) -> Result<Note>` | Create empty note from template | ❌ No |

---

## Cards

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `search_cards(search: Query, sort: SortMode) -> Result<Vec<CardId>>` | Find cards matching query with sort order | ✅ Yes (list) |
| `set_card_flag(cid: CardId, flag: u32) -> Result<OpOutput<()>>` | Set flag (red/orange/green/blue/purple) on card | ❌ No |
| `set_deck(cids: &[CardId], did: DeckId) -> Result<OpOutput<()>>` | Move cards to a different deck | ❌ No |
| `bury_or_suspend_cards(cids: &[CardId], mode: BuryOrSuspend) -> Result<OpOutput<()>>` | Suspend or bury cards | ✅ Yes |
| `unbury_or_unsuspend_cards(cids: &[CardId]) -> Result<OpOutput<()>>` | Revert suspend/bury | ✅ Yes |
| `reschedule_cards_as_new(cids: &[CardId], ...) -> Result<OpOutput<()>>` | Reset cards to new state | ❌ No |
| `reschedule_cards_as_new_defaults() -> Result<RescheduleCardsAsNewRequest>` | Get default params for reschedule | ❌ No |
| `set_due_date(cids: &[CardId], days: i32, ...) -> Result<OpOutput<()>>` | Change due date by days from today | ❌ No |
| `sort_cards(cids: &[CardId], starting_from: i32, ...) -> Result<OpOutput<()>>` | Reorder cards within deck | ❌ No |

---

## Scheduling & Study

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `get_next_card() -> Result<Option<CardId>>` | Fetch next card for review from queue | ❌ No |
| `get_queued_cards(limit: u32) -> Result<Vec<CardId>>` | Get upcoming cards without advancing queue | ❌ No |
| `get_scheduling_states(cid: CardId) -> Result<SchedulingStates>` | Show again/hard/good/easy outcomes for a card | ❌ No |
| `describe_next_states(states: &SchedulingStates) -> Result<Vec<String>>` | Human-readable descriptions of next states | ❌ No |
| `answer_card(answer: &mut CardAnswer) -> Result<OpOutput<()>>` | Submit a card answer (update due date, interval, etc.) | ❌ No |
| `congrats_info() -> Result<CongratsInfo>` | Get congratulations screen data (studied today, etc.) | ❌ No |
| `scheduler_info() -> Result<SchedulerInfo>` | Get current scheduler version, timing, counts | ❌ No |
| `timing_today() -> Result<SchedTimingToday>` | Daily schedule times (rollover, cutoff) | ❌ No |
| `current_due_day(delta: i32) -> Result<u32>` | Calculate due day with offset | ❌ No |
| `unbury_deck(did: DeckId) -> Result<OpOutput<()>>` | Unbury all cards in deck | ❌ No |
| `custom_study(cid: DeckId, ...) -> Result<OpOutput<DeckId>>` | Create temporary filtered deck for custom study | ❌ No |
| `custom_study_defaults() -> Result<CustomStudyDefaults>` | Get defaults for custom study dialog | ❌ No |
| `add_or_update_filtered_deck(deck: &mut Deck) -> Result<OpOutput<()>>` | Create/update filtered deck | ❌ No |
| `get_or_create_filtered_deck(name: &str) -> Result<Deck>` | Fetch or create filtered deck | ❌ No |
| `rebuild_filtered_deck(did: DeckId) -> Result<OpOutput<()>>` | Refill filtered deck | ❌ No |
| `empty_filtered_deck(did: DeckId) -> Result<OpOutput<()>>` | Clear cards from filtered deck | ❌ No |
| `reposition_defaults() -> Result<RepositionDefaultsRequest>` | Get defaults for reposition dialog | ❌ No |
| `rollover_for_current_scheduler() -> Result<u8>` | Hour for daily rollover in current scheduler version | ❌ No |
| `compute_optimal_retention(data: &FsrsRawInput) -> Result<RetentionValue>` | FSRS: calculate optimal retention | ❌ No |
| `get_optimal_retention_parameters() -> Result<OptimalRetentionParameters>` | FSRS: get pre-computed params | ❌ No |
| `simulate_review(request: &SimulateReviewRequest) -> Result<SimulateReviewResponse>` | FSRS: simulate review outcome | ❌ No |
| `evaluate_params(request: &EvaluateParamsRequest) -> Result<EvaluateParamsResponse>` | FSRS: evaluate parameter quality | ❌ No |
| `evaluate_params_legacy(request: &EvaluateParamsRequest) -> Result<EvaluateParamsResponse>` | FSRS: legacy evaluation | ❌ No |
| `export_dataset(cid: CardId, with_reviews: bool) -> Result<Vec<FsrsCard>>` | FSRS: export card review history | ❌ No |
| `simulate_workload(settings: &SimulateWorkloadSettings) -> Result<SimulateWorkloadResponse>` | FSRS: forecast workload | ❌ No |
| `parse_due_date_str(input: &str) -> Result<i32>` | Parse human-readable due date (e.g., "+3d") | ❌ No |

---

## Tags

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `add_tags_to_notes(nids: &[NoteId], tags: &str) -> Result<OpOutput<usize>>` | Add tags to notes | ✅ Yes |
| `remove_tags_from_notes(nids: &[NoteId], tags: &str) -> Result<OpOutput<usize>>` | Remove tags from notes | ✅ Yes |
| `remove_tags(tags: &[String]) -> Result<OpOutput<()>>` | Delete unused tags entirely | ❌ No |
| `rename_tag(old_name: &str, new_name: &str) -> Result<OpOutput<()>>` | Rename tag across all notes | ✅ Yes |
| `complete_tag(text: &str) -> Result<Vec<String>>` | Get tag completions for autocomplete | ❌ No |
| `find_and_replace_tag(nids: &[NoteId], old: &str, new: &str) -> Result<OpOutput<usize>>` | Replace one tag with another | ❌ No |
| `reparent_tags(tags: &[String], new_parent: &str) -> Result<OpOutput<()>>` | Move tags under new parent | ❌ No |
| `tag_tree() -> Result<TagTreeNode>` | Get tag hierarchy with note counts | ❌ No |
| `set_tag_collapsed(tag: &str, collapsed: bool) -> Result<OpOutput<()>>` | Toggle tag collapse in browser | ❌ No |
| `clear_unused_tags() -> Result<OpOutput<()>>` | Delete all tags with no notes | ❌ No |

---

## Note Types

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `get_notetype(ntid: NotetypeId) -> Result<Option<Arc<Notetype>>>` | Fetch notetype by ID (cached) | ❌ No |
| `get_notetype_by_name(name: &str) -> Result<Option<Notetype>>` | Fetch notetype by name | ❌ No |
| `get_all_notetypes() -> Result<Vec<Arc<Notetype>>>` | List all notetypes | ✅ Partial (list) |
| `add_notetype(nt: &mut Notetype) -> Result<OpOutput<NotetypeId>>` | Add new notetype | ❌ No |
| `update_notetype(nt: &Notetype) -> Result<OpOutput<()>>` | Modify notetype fields/templates | ❌ No |
| `remove_notetype(ntid: NotetypeId) -> Result<OpOutput<()>>` | Delete notetype (cards + notes must be gone) | ❌ No |
| `change_notetype_of_notes(nids: &[NoteId], old_ntid: NotetypeId, new_ntid: NotetypeId, ...) -> Result<OpOutput<()>>` | Bulk convert notes to different notetype | ❌ No |
| `empty_cards(ntid: NotetypeId) -> Result<EmptyCardsReport>` | Find cards with empty fronts/backs | ❌ No |
| `empty_cards_report(report: &EmptyCardsReport) -> Result<String>` | Human-readable empty cards report | ❌ No |
| `notetype_change_info(old_ntid: NotetypeId, new_ntid: NotetypeId, ...) -> Result<NotetypeChangeInfo>` | Preview notetype conversion before commit | ❌ No |
| `new_note(nt: &Notetype) -> Result<Note>` | Create blank note from template | ❌ No |
| `get_single_notetype_of_notes(nids: &[NoteId]) -> Result<Option<Arc<Notetype>>>` | Get notetype if all notes use same one | ❌ No |
| `get_all_notetypes_of_search_notes(search: &str) -> Result<Vec<Arc<Notetype>>>` | Get notetypes for searched notes | ❌ No |
| `render_card(note: &Note, template_idx: usize, fill_empty: bool) -> Result<RenderedCard>` | Generate HTML for front/back | ❌ No |
| `render_existing_card(cid: CardId, ...) -> Result<RenderedCard>` | Render card from DB | ❌ No |
| `render_uncommitted_card(note: &Note, cid: CardId, ...) -> Result<RenderedCard>` | Render card with unsaved note | ❌ No |
| `get_template(ntid: NotetypeId, card_idx: usize) -> Result<CardTemplate>` | Fetch single card template | ❌ No |
| `report_media_field_referencing_templates(ntid: NotetypeId) -> Result<String>` | Validate media in templates | ❌ No |

---

## Search

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `search_cards(query: SearchQuery, sort: SortMode) -> Result<Vec<CardId>>` | Find cards by search query | ✅ Yes |
| `search_notes(query: SearchQuery, sort: SortMode) -> Result<Vec<NoteId>>` | Find notes by search query with sort | ✅ Partial (search) |
| `search_notes_unordered(query: SearchQuery) -> Result<Vec<NoteId>>` | Find notes without sorting (faster) | ✅ Yes |

---

## Configuration

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `get_config_bool(key: BoolKey) -> Result<bool>` | Get boolean setting | ❌ No |
| `get_config_string(key: StringKey) -> Result<String>` | Get string setting | ❌ No |
| `get_config_i(key: IntKey) -> Result<i32>` | Get integer setting | ❌ No |
| `get_config_json(key: JsonKey) -> Result<Value>` | Get JSON setting (flexible schema) | ❌ No |
| `set_config_bool(key: BoolKey, val: bool) -> Result<OpOutput<()>>` | Set boolean setting | ❌ No |
| `set_config_string(key: StringKey, val: &str) -> Result<OpOutput<()>>` | Set string setting | ❌ No |
| `set_config_json(key: JsonKey, val: &Value) -> Result<OpOutput<()>>` | Set JSON setting | ❌ No |
| `remove_config(key: JsonKey) -> Result<OpOutput<()>>` | Delete JSON setting | ❌ No |
| `get_configured_utc_offset() -> Result<i32>` | Get user's UTC offset minutes | ❌ No |
| `set_configured_utc_offset(mins: i32) -> Result<OpOutput<()>>` | Set user's UTC offset | ❌ No |

---

## Deck Configuration

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `get_deck_config(dcid: DeckConfigId) -> Result<Option<DeckConfig>>` | Fetch a deck config (new/review/learnSteps) | ❌ No |
| `get_deck_configs_for_update(did: DeckId) -> Result<Vec<DeckConfig>>` | List configs for a deck | ❌ No |
| `update_deck_configs(configs: &[DeckConfig]) -> Result<OpOutput<()>>` | Update multiple configs | ❌ No |
| `fsrs_params(did: DeckId) -> Result<FsrsParameters>` | Get FSRS settings for deck | ❌ No |

---

## Statistics

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `card_stats(cid: CardId) -> Result<CardStatsResponse>` | Get stats for one card (history, intervals, etc.) | ❌ No |
| `studied_today() -> Result<String>` | Get "studied X cards in Y minutes today" | ❌ No |
| `get_review_logs(search: &str, days: i32) -> Result<Vec<RevlogEntry>>` | Query review log for cards/period | ❌ No |
| `graph_data_for_search(search: &str, days: u32) -> Result<GraphsResponse>` | Get data for review/added/forecast/intervals graphs | ❌ No |

---

## Media

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `add_file(fname: &str, data: &[u8]) -> Result<String>` | Add/overwrite media file | ✅ Yes |
| `sync_media() -> Result<()>` | Sync media with AnkiWeb | ❌ No (sync does it) |
| `check() -> Result<MediaCheckResult>` | Scan media folder for missing/unused files | ❌ No |
| `empty_trash() -> Result<usize>` | Delete files in trash folder | ❌ No |
| `restore_trash(fnames: &[&str]) -> Result<()>` | Recover files from trash | ❌ No |
| `media() -> Result<Arc<MediaManager>>` | Get media manager instance | ✅ Partial (internal) |
| `media_checker() -> Result<Arc<MediaChecker>>` | Get media checker instance | ❌ No |
| `all_checksums_after_checking() -> Result<HashMap<String, String>>` | Get checksums after check | ❌ No |
| `register_changes(changes: &MediaChanges) -> Result<()>` | Notify of external media changes | ❌ No |
| `remove_files(fnames: &[&str]) -> Result<()>` | Delete media files | ❌ No |

---

## Synchronization

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `sync(auth: &SyncAuth, progress: Handler) -> Result<SyncOutput>` | Full sync with AnkiWeb (download or upload) | ✅ Yes |
| `sync_status_offline() -> Result<SyncStatusResponse>` | Check sync needed (no network) | ❌ No |
| `online_sync_status_check(auth: &SyncAuth) -> Result<SyncStatusResponse>` | Check sync needed (requires login) | ❌ No |
| `full_upload(auth: &SyncAuth, progress: Handler) -> Result<()>` | Upload entire collection | ❌ No (sync handles) |
| `full_download(auth: &SyncAuth, progress: Handler) -> Result<()>` | Download entire collection | ❌ No (sync handles) |
| `normal_sync(auth: &SyncAuth, progress: Handler) -> Result<()>` | Incremental two-way sync | ❌ No (sync handles) |
| `sync_meta() -> Result<SyncMeta>` | Get sync metadata (last sync time, etc.) | ❌ No |
| `check_upload_limit() -> Result<()>` | Verify upload doesn't exceed limit | ❌ No |
| `apply_graves(graves: &Graves) -> Result<()>` | Process deleted items from sync | ❌ No (internal) |
| `handle_received_upload(data: &[u8]) -> Result<()>` | Apply upload chunk from AnkiWeb | ❌ No (internal) |
| `server_*` (various) | Server-side sync operations | ❌ No (server-only) |

---

## Import & Export

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `import_apkg(path: &Path, progress: Handler) -> Result<ImportLogReport>` | Import .apkg (notes, cards, media, models) | ❌ No |
| `import_csv(file: ImportFile, ...) -> Result<ImportLogReport>` | Import from CSV/TSV | ❌ No |
| `import_json_file(path: &Path, ...) -> Result<ImportLogReport>` | Import from .json | ❌ No |
| `import_json_string(json: &str, ...) -> Result<ImportLogReport>` | Import from JSON string | ❌ No |
| `export_apkg(did: DeckId, include_scheduling: bool, ...) -> Result<Vec<u8>>` | Export deck to .apkg file | ❌ No |
| `export_colpkg(include_media: bool, ...) -> Result<Vec<u8>>` | Export entire collection to .colpkg | ❌ No |
| `export_note_csv(search: &str, with_html: bool, ...) -> Result<Vec<u8>>` | Export notes as CSV | ❌ No |
| `export_card_csv(search: &str, ...) -> Result<Vec<u8>>` | Export cards as CSV | ❌ No |
| `get_csv_metadata() -> Result<CsvMetadata>` | Get CSV field mapping | ❌ No |

---

## Backup & Maintenance

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `maybe_backup(force: bool) -> Result<bool>` | Create backup if collection changed since last one | ✅ Yes (snapshot) |
| `close(desired_version: Option<SchemaVersion>) -> Result<()>` | Close collection (save + cleanup) | ✅ Implicit |
| `check_database() -> Result<Vec<DatabaseCheckProblem>>` | Integrity check (repair errors if needed) | ❌ No |
| `changed_since_last_backup() -> Result<bool>` | Check if backup needed | ❌ No |

---

## Undo & Transactions

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `undo() -> Result<OpChangesAfterUndo>` | Undo last operation | ❌ No |
| `redo() -> Result<OpChangesAfterUndo>` | Redo last undo | ❌ No |
| `undo_status() -> Result<UndoStatus>` | Get undo/redo status strings | ❌ No |
| `add_custom_undo_step(name: &str) -> Result<usize>` | Insert checkpoint in undo stack | ❌ No |
| `merge_undoable_ops(from: usize) -> Result<OpChanges>` | Combine ops into single undo step | ❌ No |

---

## Collection Metadata

| Method | Summary | CLI Exposed? |
|--------|---------|-------------|
| `changes_since_open() -> Result<u64>` | Count of DB rows modified since open | ❌ No |
| `set_modified() -> Result<()>` | Mark collection as modified | ❌ No (automatic) |
| `set_schema_modified() -> Result<()>` | Mark schema as modified | ❌ No (automatic) |
| `collection_changed_since_sync() -> Result<bool>` | Check if content changed since last sync | ❌ No |
| `schema_changed_since_sync() -> Result<bool>` | Check if schema changed since last sync | ❌ No |
| `as_builder() -> CollectionBuilder` | Create builder from existing collection | ❌ No |
| `tr() -> &I18n` | Get translation provider | ❌ No |

---

## Storage Layer (Direct SQL Access)

The `col.storage` field provides low-level database access. Methods directly called from CLI:

| Method | Summary | CLI Used? |
|--------|---------|-----------|
| `get_note(nid: NoteId) -> Result<Option<Note>>` | Fetch note by ID | ✅ Yes (tags.rs) |
| `get_card(cid: CardId) -> Result<Option<Card>>` | Fetch card by ID | ❌ No |
| `get_notetype(ntid: NotetypeId) -> Result<Option<Notetype>>` | Fetch notetype by ID | ❌ No |
| `get_all_cards() -> Result<Vec<Card>>` | Fetch all cards | ❌ No |

Most storage methods are internal; only get_note is called externally in the current CLI.

---

## Gap Analysis

### High-Priority Features (High User Value, Currently Unexposed)

1. **Card Answering / Scheduling**
   - `answer_card()` – Core study functionality; needed for any study-mode CLI feature
   - `get_next_card()`, `get_scheduling_states()` – Essential for automated review workflows
   - Impact: Blocks implementation of "grade" commands and scheduled review simulation

2. **Undo/Redo**
   - `undo()`, `redo()`, `undo_status()`, `merge_undoable_ops()` – Critical for any destructive operations
   - Impact: CLI operations are not reversible; risky for batch operations

3. **Import/Export**
   - `import_apkg()`, `import_csv()`, `export_apkg()`, `export_colpkg()`, `export_note_csv()` – Needed for collection exchange, backups, bulk data movement
   - Impact: Users cannot exchange data with Anki Desktop or third-party tools from CLI

4. **Bulk Note Operations**
   - `add_notes()` – Batch import performance is critical for large datasets
   - `after_note_updates()` – Card regeneration after external changes
   - Impact: Single-note operations are slow for bulk workflows (e.g., AI-generated cards)

5. **Deck & Notetype Management**
   - `update_deck()`, `add_notetype()`, `update_notetype()` – Can't modify existing structures
   - `reparent_decks()` – Can't reorganize deck hierarchy
   - Impact: Limited deck/model configuration from CLI

### Medium-Priority Features (Moderate Value, Nice to Have)

1. **Advanced Card Operations**
   - `set_card_flag()`, `set_deck()`, `reschedule_cards_as_new()` – More granular control over cards
   - `parse_due_date_str()` – User-friendly date parsing ("+3d", "2025-12-25")

2. **Filtered Decks & Custom Study**
   - `custom_study()`, `add_or_update_filtered_deck()`, `rebuild_filtered_deck()` – For temporary study sessions
   - `empty_filtered_deck()` – Cleanup

3. **Tag Hierarchy**
   - `tag_tree()`, `reparent_tags()`, `find_and_replace_tag()` – Organize tags hierarchically
   - `set_tag_collapsed()` – UI state persistence

4. **Configuration**
   - `get_config_*`, `set_config_*` – Access global and deck-specific settings
   - `get_configured_utc_offset()` – Timezone handling for scheduling

5. **Deck Configuration**
   - `get_deck_configs_for_update()`, `update_deck_configs()`, `fsrs_params()` – Modify scheduler settings
   - Impact: Users must use Desktop to tune deck parameters

6. **Statistics & Reporting**
   - `card_stats()`, `studied_today()`, `get_review_logs()`, `graph_data_for_search()` – Learning analytics
   - Impact: Can't generate custom reports from CLI

7. **Media Management**
   - `check()`, `empty_trash()`, `restore_trash()`, `register_changes()` – Full media lifecycle
   - Impact: Limited media operations (can only add files)

### Low-Priority Features (Low Value, Advanced/Niche)

1. **Notetype Conversion**
   - `change_notetype_of_notes()`, `notetype_change_info()`, `empty_cards()` – Advanced model migration

2. **FSRS Optimization**
   - `compute_optimal_retention()`, `evaluate_params()`, `export_dataset()`, `simulate_*` – Advanced scheduler research

3. **Advanced Sync**
   - `sync_status_offline()`, `online_sync_status_check()`, `check_upload_limit()` – Sync introspection

4. **Database Maintenance**
   - `check_database()`, `changed_since_last_backup()` – Internal database health

---

## CLI Implementation Notes

### Current CLI Coverage

| Domain | Count | Exposed | % |
|--------|-------|---------|---|
| Decks | 23 | 3 | 13% |
| Notes | 7 | 2 | 29% |
| Cards | 9 | 2 | 22% |
| Tags | 9 | 3 | 33% |
| Scheduling | 28 | 0 | 0% |
| Notetypes | 17 | 1 | 6% |
| Search | 3 | 3 | 100% |
| Config | 11 | 0 | 0% |
| Deck Config | 4 | 0 | 0% |
| Statistics | 4 | 0 | 0% |
| Media | 11 | 1 | 9% |
| Sync | 18 | 1 | 6% |
| Import/Export | 9 | 0 | 0% |
| Backup | 3 | 1 | 33% |
| Undo | 5 | 0 | 0% |
| **Total** | **161** | **20** | **12%** |

### Recommendations for Next Phases

**Phase 1 (Quick Wins):** Low effort, high value
- [ ] Expose undo/redo commands (`undo`, `redo`, `undo-status`)
- [ ] Card operations: `set-flag`, `set-deck`, `reschedule-as-new`
- [ ] Export: `export-apkg`, `export-csv`
- [ ] Deck management: `update-deck`, `reparent-decks`

**Phase 2 (Core Features):** Medium effort
- [ ] Card answering for study-mode: `study start`, `grade [1-4]`, `undo`
- [ ] Bulk note operations: batch import with progress, `add-notes` endpoint
- [ ] Import: `import-apkg`, `import-csv` commands
- [ ] Configuration: `get-config`, `set-config` for global settings

**Phase 3 (Polish):** Higher effort, specialized use cases
- [ ] Statistics: `card-stats`, `review-logs`, `studied-today`
- [ ] Notetype operations: create, update, remove models
- [ ] Filtered decks: create/rebuild for custom study
- [ ] Media management: check, trash, restore
- [ ] FSRS: parameter simulation and optimization tools

---

## Storage Layer Methods

The `Collection.storage` field is a `SqliteStorage` instance. Key public methods used in CLI:

```rust
pub struct SqliteStorage {
    pub db: Connection,  // Direct SQLite access
    // ... internals
}

impl SqliteStorage {
    pub fn get_note(nid: NoteId) -> Result<Option<Note>>
    pub fn get_card(cid: CardId) -> Result<Option<Card>>
    pub fn get_notetype(ntid: NotetypeId) -> Result<Option<Notetype>>
    pub fn get_deck(did: DeckId) -> Result<Option<Deck>>
    pub fn get_all_cards() -> Result<Vec<Card>>
    // ... 100+ internal methods for schema management, sync, etc.
}
```

Storage methods are primarily internal; the high-level Collection API is preferred.

---

## Changelog & Future Directions

- **Original audit date:** May 14, 2026
- **Backend version at audit:** anki-9c9b125b7236058d/d52ca66
- **CLI version at audit:** HEAD

For updates to this inventory, re-run the audit script across `rslib/src/` to capture new Collection impl blocks and public method signatures.

