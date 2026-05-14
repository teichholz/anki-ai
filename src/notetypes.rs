use anki::notetype::NoteField;
use anki::services::NotetypesService;
use serde::Serialize;

use crate::collection::CollectionHandle;

/// Summary of a notetype (model) stored in the collection.
#[derive(Serialize)]
pub struct NotetypeInfo {
    pub id: i64,
    pub name: String,
    pub use_count: u32,
}

/// Return a list of all notetypes with their note-use counts.
pub fn list_notetypes(col: &mut CollectionHandle) -> anyhow::Result<Vec<NotetypeInfo>> {
    let result = col.get_notetype_names_and_counts()?;
    let infos = result
        .entries
        .into_iter()
        .map(|e| NotetypeInfo {
            id: e.id,
            name: e.name,
            use_count: e.use_count,
        })
        .collect();
    Ok(infos)
}

/// Add a new field to the notetype with the given name.
///
/// Returns `Err` if no notetype with that name exists or the update fails.
pub fn add_notetype_field(
    col: &mut CollectionHandle,
    notetype_name: &str,
    field_name: &str,
) -> anyhow::Result<()> {
    let arc_nt = col
        .get_notetype_by_name(notetype_name)?
        .ok_or_else(|| anyhow::anyhow!("Note type '{}' not found.", notetype_name))?;
    let mut nt = (*arc_nt).clone();

    nt.fields.push(NoteField::new(field_name));
    col.update_notetype(&mut nt, false)?;
    Ok(())
}

/// Return the ordered list of field names for the notetype with the given name.
///
/// Returns `Err` if no notetype with that name exists.
pub fn get_notetype_fields(col: &mut CollectionHandle, name: &str) -> anyhow::Result<Vec<String>> {
    let nt = col
        .get_notetype_by_name(name)?
        .ok_or_else(|| anyhow::anyhow!("Note type '{}' not found.", name))?;

    let fields = nt.fields.iter().map(|f| f.name.clone()).collect();
    Ok(fields)
}

#[cfg(test)]
mod tests {
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
    fn test_list_notetypes_includes_basic() {
        let (_dir, mut col) = setup();
        let notetypes = list_notetypes(&mut col).unwrap();
        let names: Vec<&str> = notetypes.iter().map(|n| n.name.as_str()).collect();
        assert!(
            names.contains(&"Basic"),
            "Expected 'Basic' in notetypes, got: {names:?}"
        );
    }

    #[test]
    fn test_get_notetype_fields_basic() {
        let (_dir, mut col) = setup();
        let fields = get_notetype_fields(&mut col, "Basic").unwrap();
        assert!(
            fields.contains(&"Front".to_string()),
            "Expected 'Front' field, got: {fields:?}"
        );
        assert!(
            fields.contains(&"Back".to_string()),
            "Expected 'Back' field, got: {fields:?}"
        );
    }

    #[test]
    fn test_add_notetype_field_appends_field() {
        let (_dir, mut col) = setup();
        add_notetype_field(&mut col, "Basic", "Extra").unwrap();
        let fields = get_notetype_fields(&mut col, "Basic").unwrap();
        assert!(
            fields.contains(&"Extra".to_string()),
            "Expected 'Extra' field after add, got: {fields:?}"
        );
    }

    #[test]
    fn test_add_notetype_field_not_found() {
        let (_dir, mut col) = setup();
        let err = add_notetype_field(&mut col, "NonExistentNotetype", "Extra").unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "Expected 'not found' error, got: {err}"
        );
    }

    #[test]
    fn test_get_notetype_fields_not_found() {
        let (_dir, mut col) = setup();
        let err = get_notetype_fields(&mut col, "NonExistentNotetype").unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "Expected 'not found' error, got: {err}"
        );
    }
}
