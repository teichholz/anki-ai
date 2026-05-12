use crate::collection::CollectionHandle;

/// Return a sorted list of all tag names in the collection.
pub fn list_tags(col: &mut CollectionHandle) -> anyhow::Result<Vec<String>> {
    let mut tags: Vec<String> = col
        .storage
        .all_tags()?
        .into_iter()
        .map(|t| t.name)
        .collect();
    tags.sort();
    Ok(tags)
}

/// Add space-separated tags to all notes matching `query`.
/// Returns the number of notes modified (0 if no notes match).
pub fn bulk_add_tags(
    col: &mut CollectionHandle,
    query: &str,
    tags: &[String],
) -> anyhow::Result<usize> {
    let nids = col.search_notes_unordered(query)?;
    if nids.is_empty() {
        return Ok(0);
    }
    let tag_str = tags.join(" ");
    let result = col.add_tags_to_notes(&nids, &tag_str)?;
    Ok(result.output)
}

/// Remove space-separated tags from all notes matching `query`.
/// Returns the number of notes modified (0 if no notes match).
pub fn bulk_remove_tags(
    col: &mut CollectionHandle,
    query: &str,
    tags: &[String],
) -> anyhow::Result<usize> {
    let nids = col.search_notes_unordered(query)?;
    if nids.is_empty() {
        return Ok(0);
    }
    let tag_str = tags.join(" ");
    let result = col.remove_tags_from_notes(&nids, &tag_str)?;
    Ok(result.output)
}

/// Rename a tag (and all its children) across all notes.
/// Returns the number of notes modified.
pub fn rename_tag(col: &mut CollectionHandle, old: &str, new: &str) -> anyhow::Result<usize> {
    let result = col.rename_tag(old, new)?;
    Ok(result.output)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;
    use crate::collection::open_collection;
    use crate::notes::add_note;

    fn setup() -> (TempDir, CollectionHandle) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        let col = open_collection(Some(&path)).unwrap();
        (dir, col)
    }

    fn basic_fields(front: &str, back: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), front.to_string());
        fields.insert("Back".to_string(), back.to_string());
        fields
    }

    #[test]
    fn test_list_tags_empty_collection() {
        let (_dir, mut col) = setup();
        let tags = list_tags(&mut col).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_bulk_add_and_list_tags() {
        let (_dir, mut col) = setup();
        let fields = basic_fields("Q1", "A1");
        add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let tags = vec!["mytag".to_string()];
        let count = bulk_add_tags(&mut col, "Q1", &tags).unwrap();
        assert_eq!(count, 1);

        let all_tags = list_tags(&mut col).unwrap();
        assert!(all_tags.contains(&"mytag".to_string()));
    }

    #[test]
    fn test_bulk_remove_tags() {
        let (_dir, mut col) = setup();
        let fields = basic_fields("Q2", "A2");
        add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let tags = vec!["removeme".to_string()];
        bulk_add_tags(&mut col, "Q2", &tags).unwrap();

        // Verify tag is present
        let all_tags = list_tags(&mut col).unwrap();
        assert!(all_tags.contains(&"removeme".to_string()));

        // Remove the tag
        let count = bulk_remove_tags(&mut col, "Q2", &tags).unwrap();
        assert_eq!(count, 1);

        // Verify tag is gone from notes (it may remain in the tag list until
        // clear_unused_tags is called, so we check the note directly)
        let nids = col.search_notes_unordered("Q2").unwrap();
        assert!(!nids.is_empty());
        let note = col.storage.get_note(nids[0]).unwrap().unwrap();
        assert!(!note.tags.contains(&"removeme".to_string()));
    }

    #[test]
    fn test_rename_tag() {
        let (_dir, mut col) = setup();
        let fields = basic_fields("Q3", "A3");
        add_note(&mut col, "Default", "Basic", &fields).unwrap();

        let tags = vec!["oldname".to_string()];
        bulk_add_tags(&mut col, "Q3", &tags).unwrap();

        let count = rename_tag(&mut col, "oldname", "newname").unwrap();
        assert_eq!(count, 1);

        let nids = col.search_notes_unordered("Q3").unwrap();
        let note = col.storage.get_note(nids[0]).unwrap().unwrap();
        assert!(!note.tags.contains(&"oldname".to_string()));
        assert!(note.tags.contains(&"newname".to_string()));
    }

    #[test]
    fn test_bulk_add_no_matching_notes_returns_zero() {
        let (_dir, mut col) = setup();
        let tags = vec!["sometag".to_string()];
        let count = bulk_add_tags(&mut col, "nonexistentquery12345", &tags).unwrap();
        assert_eq!(count, 0);
    }
}
