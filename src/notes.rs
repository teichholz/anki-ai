use std::collections::HashMap;

use anki::card::CardId;
use anki::notes::Note;
use anki::notes::NoteId;
use anyhow::anyhow;
use serde::Serialize;

use crate::collection::CollectionHandle;

#[derive(Debug, Serialize)]
pub struct NoteInfo {
    pub id: i64,
    #[serde(rename = "type")]
    pub note_type: String,
    pub fields: HashMap<String, String>,
    pub tags: Vec<String>,
}

fn note_to_info(col: &mut CollectionHandle, note: &Note) -> anyhow::Result<NoteInfo> {
    let nt = col
        .get_notetype(note.notetype_id)?
        .ok_or_else(|| anyhow!("Notetype not found for note {}", note.id.0))?;
    let field_names: Vec<String> = nt.fields.iter().map(|f| f.name.clone()).collect();
    let fields = field_names
        .into_iter()
        .zip(note.fields().iter().cloned())
        .collect();
    Ok(NoteInfo {
        id: note.id.0,
        note_type: nt.name.clone(),
        fields,
        tags: note.tags.clone(),
    })
}

pub fn add_note(
    col: &mut CollectionHandle,
    deck_name: &str,
    note_type: &str,
    fields: &HashMap<String, String>,
) -> anyhow::Result<i64> {
    let nt = col
        .get_notetype_by_name(note_type)?
        .ok_or_else(|| anyhow!("Note type '{}' not found.", note_type))?;

    let deck_id = col
        .get_deck_id(deck_name)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", deck_name))?;

    let mut note = Note::new(&nt);

    let field_names: Vec<String> = nt.fields.iter().map(|f| f.name.clone()).collect();

    for (name, value) in fields {
        let idx = field_names.iter().position(|n| n == name).ok_or_else(|| {
            anyhow!(
                "Field '{}' not in '{}'. Valid: {:?}",
                name,
                note_type,
                field_names
            )
        })?;
        note.set_field(idx, value.as_str())?;
    }

    col.add_note(&mut note, deck_id)?;

    Ok(note.id.0)
}

pub fn get_note(col: &mut CollectionHandle, note_id: i64) -> anyhow::Result<NoteInfo> {
    let query = format!("nid:{note_id}");
    let nids = col.search_notes_unordered(query.as_str())?;
    if nids.is_empty() {
        return Err(anyhow!("Note {note_id} not found."));
    }
    let note = col
        .storage
        .get_note(NoteId(note_id))?
        .ok_or_else(|| anyhow!("Note {note_id} not found."))?;
    note_to_info(col, &note)
}

pub fn delete_note(col: &mut CollectionHandle, note_id: i64) -> anyhow::Result<()> {
    let query = format!("nid:{note_id}");
    let nids = col.search_notes_unordered(query.as_str())?;
    if nids.is_empty() {
        return Err(anyhow!("Note {note_id} not found."));
    }
    col.remove_notes(&[NoteId(note_id)])?;
    Ok(())
}

pub fn update_note(
    col: &mut CollectionHandle,
    note_id: i64,
    fields: &HashMap<String, String>,
) -> anyhow::Result<NoteInfo> {
    let query = format!("nid:{note_id}");
    let nids = col.search_notes_unordered(query.as_str())?;
    if nids.is_empty() {
        return Err(anyhow!("Note {note_id} not found."));
    }

    let mut note = col
        .storage
        .get_note(NoteId(note_id))?
        .ok_or_else(|| anyhow!("Note {note_id} not found."))?;

    let nt = col
        .get_notetype(note.notetype_id)?
        .ok_or_else(|| anyhow!("Notetype not found for note {note_id}"))?;
    let field_names: Vec<String> = nt.fields.iter().map(|f| f.name.clone()).collect();

    for (name, value) in fields {
        let idx = field_names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| anyhow!("Field '{}' not found. Valid: {:?}", name, field_names))?;
        note.set_field(idx, value.as_str())?;
    }

    col.update_note(&mut note)?;

    // Re-fetch to get the canonical state after update
    let updated = col
        .storage
        .get_note(NoteId(note_id))?
        .ok_or_else(|| anyhow!("Note {note_id} not found after update."))?;
    note_to_info(col, &updated)
}

/// Move all cards belonging to the given notes to a different deck.
/// Returns the number of cards moved (one note may have multiple cards).
pub fn move_notes_to_deck(
    col: &mut CollectionHandle,
    note_ids: &[i64],
    deck_name: &str,
) -> anyhow::Result<usize> {
    let deck_id = col
        .get_deck_id(deck_name)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", deck_name))?;

    // Collect all card IDs for the given notes via search.
    let mut card_ids: Vec<CardId> = Vec::new();
    for &nid in note_ids {
        let query = format!("nid:{nid}");
        let cids = col.search_cards(&query, anki::search::SortMode::NoOrder)?;
        if cids.is_empty() {
            return Err(anyhow!("Note {nid} not found."));
        }
        card_ids.extend(cids);
    }

    let count = col.set_deck(&card_ids, deck_id)?.output;
    Ok(count)
}

/// Bulk regex find/replace across note fields.
/// `query` selects which notes to operate on (Anki search syntax; empty = all notes).
/// `field_name` restricts replacement to one field; `None` targets all fields.
/// Returns the number of notes modified.
pub fn find_replace(
    col: &mut CollectionHandle,
    query: &str,
    search_re: &str,
    replacement: &str,
    field_name: Option<String>,
) -> anyhow::Result<usize> {
    let nids = col.search_notes_unordered(query)?;
    if nids.is_empty() {
        return Ok(0);
    }
    let count = col
        .find_and_replace(nids, search_re, replacement, field_name)?
        .output;
    Ok(count)
}

pub fn search_notes(col: &mut CollectionHandle, query: &str) -> anyhow::Result<Vec<NoteInfo>> {
    let nids = col.search_notes_unordered(query)?;
    let mut results = Vec::with_capacity(nids.len());
    for nid in nids {
        let note = col
            .storage
            .get_note(nid)?
            .ok_or_else(|| anyhow!("Note {} not found.", nid.0))?;
        results.push(note_to_info(col, &note)?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;
    use crate::collection::open_collection;

    fn setup() -> (TempDir, CollectionHandle) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        let col = open_collection(Some(&path)).unwrap();
        (dir, col)
    }

    #[test]
    fn test_add_note_returns_id() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "Hello".to_string());
        fields.insert("Back".to_string(), "World".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_note_returns_correct_fields() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "Question".to_string());
        fields.insert("Back".to_string(), "Answer".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let info = get_note(&mut col, id).unwrap();
        assert_eq!(info.id, id);
        assert_eq!(info.fields["Front"], "Question");
        assert_eq!(info.fields["Back"], "Answer");
    }

    #[test]
    fn test_search_notes_finds_by_query() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "UniqueSearchTerm123".to_string());
        fields.insert("Back".to_string(), "BackText".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let results = search_notes(&mut col, "UniqueSearchTerm123").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
    }

    #[test]
    fn test_update_note_modifies_field() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "Original".to_string());
        fields.insert("Back".to_string(), "OriginalBack".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let mut update = HashMap::new();
        update.insert("Front".to_string(), "Updated".to_string());
        let info = update_note(&mut col, id, &update).unwrap();
        assert_eq!(info.fields["Front"], "Updated");
        assert_eq!(info.fields["Back"], "OriginalBack");
    }

    #[test]
    fn test_delete_note_removes_note() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "ToDelete".to_string());
        fields.insert("Back".to_string(), "DeleteBack".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        delete_note(&mut col, id).unwrap();

        let err = get_note(&mut col, id).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_add_note_invalid_notetype() {
        let (_dir, mut col) = setup();
        let fields = HashMap::new();
        let err = add_note(&mut col, "Default", "NonExistentType", &fields).unwrap_err();
        assert!(err
            .to_string()
            .contains("Note type 'NonExistentType' not found."));
    }

    #[test]
    fn test_add_note_invalid_deck() {
        let (_dir, mut col) = setup();
        let fields = HashMap::new();
        let err = add_note(&mut col, "NonExistentDeck", "Basic", &fields).unwrap_err();
        assert!(err
            .to_string()
            .contains("Deck 'NonExistentDeck' not found."));
    }

    #[test]
    fn test_add_note_invalid_field() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("NonExistentField".to_string(), "value".to_string());
        let err = add_note(&mut col, "Default", "Basic", &fields).unwrap_err();
        assert!(err
            .to_string()
            .contains("Field 'NonExistentField' not in 'Basic'"));
    }

    #[test]
    fn test_get_note_not_found() {
        let (_dir, mut col) = setup();
        let err = get_note(&mut col, 999999999).unwrap_err();
        assert!(err.to_string().contains("Note 999999999 not found."));
    }

    #[test]
    fn test_search_notes_empty_result() {
        let (_dir, mut col) = setup();
        let results = search_notes(&mut col, "zzz_no_such_term_xyz_999").unwrap();
        assert!(
            results.is_empty(),
            "Expected empty results, got: {results:?}"
        );
    }

    #[test]
    fn test_update_note_not_found() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "New".to_string());
        let err = update_note(&mut col, 999999999, &fields).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_update_note_invalid_field() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "Hello".to_string());
        fields.insert("Back".to_string(), "World".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let mut bad_fields = HashMap::new();
        bad_fields.insert("NonExistentField".to_string(), "value".to_string());
        let err = update_note(&mut col, id, &bad_fields).unwrap_err();
        assert!(
            err.to_string().contains("Field"),
            "Expected 'Field' in error message, got: {err}"
        );
    }

    #[test]
    fn test_delete_note_not_found() {
        let (_dir, mut col) = setup();
        let err = delete_note(&mut col, 999999999).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_move_notes_to_deck() {
        let (_dir, mut col) = setup();
        crate::decks::create_deck(&mut col, "Target").unwrap();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "MoveMe".to_string());
        fields.insert("Back".to_string(), "Back".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let moved = move_notes_to_deck(&mut col, &[id], "Target").unwrap();
        assert!(moved > 0);

        // Verify the note's card is now in Target
        let cards = col
            .search_cards(&format!("nid:{id} deck:Target"), anki::search::SortMode::NoOrder)
            .unwrap();
        assert!(!cards.is_empty(), "card should be in Target deck");
    }

    #[test]
    fn test_move_notes_to_nonexistent_deck_errors() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "F".to_string());
        fields.insert("Back".to_string(), "B".to_string());
        let id = add_note(&mut col, "Default", "Basic", &fields).unwrap();
        let err = move_notes_to_deck(&mut col, &[id], "NoSuchDeck").unwrap_err();
        assert!(err.to_string().contains("NoSuchDeck"));
    }

    #[test]
    fn test_find_replace_all_fields() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "colour".to_string());
        fields.insert("Back".to_string(), "favourite colour".to_string());
        add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let updated = find_replace(&mut col, "colour", "colour", "color", None).unwrap();
        assert_eq!(updated, 1);

        let results = search_notes(&mut col, "color").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fields["Front"], "color");
        assert_eq!(results[0].fields["Back"], "favourite color");
    }

    #[test]
    fn test_find_replace_specific_field() {
        let (_dir, mut col) = setup();
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "hello".to_string());
        fields.insert("Back".to_string(), "hello".to_string());
        add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let updated =
            find_replace(&mut col, "hello", "hello", "world", Some("Front".to_string())).unwrap();
        assert_eq!(updated, 1);

        let results = search_notes(&mut col, "world").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fields["Front"], "world");
        assert_eq!(results[0].fields["Back"], "hello");
    }

    #[test]
    fn test_find_replace_no_matches_returns_zero() {
        let (_dir, mut col) = setup();
        let count = find_replace(&mut col, "", "zzz_no_match", "x", None).unwrap();
        assert_eq!(count, 0);
    }
}
