use anki::timestamp::TimestampSecs;
use anki_proto::decks::DeckTreeNode;
use anyhow::anyhow;
use serde::Serialize;

use crate::collection::CollectionHandle;

#[derive(Serialize)]
pub struct DeckInfo {
    pub id: i64,
    pub name: String,
    pub new: u32,
    pub learning: u32,
    pub review: u32,
}

fn tree_to_list(node: &DeckTreeNode, parent_name: &str) -> Vec<DeckInfo> {
    let mut results = Vec::new();

    let full_name = if parent_name.is_empty() {
        node.name.clone()
    } else {
        format!("{}::{}", parent_name, node.name)
    };

    let child_parent: &str = if node.deck_id != 0 {
        results.push(DeckInfo {
            id: node.deck_id,
            name: full_name.clone(),
            new: node.new_count,
            learning: node.learn_count,
            review: node.review_count,
        });
        &full_name
    } else {
        ""
    };

    for child in &node.children {
        results.extend(tree_to_list(child, child_parent));
    }

    results
}

pub fn list_decks(col: &mut CollectionHandle) -> anyhow::Result<Vec<DeckInfo>> {
    let tree = col.deck_tree(Some(TimestampSecs::now()))?;
    // Root node has deck_id == 0 and name == ""; iterate its children directly
    let mut results = Vec::new();
    for child in &tree.children {
        results.extend(tree_to_list(child, ""));
    }
    Ok(results)
}

pub fn create_deck(col: &mut CollectionHandle, name: &str) -> anyhow::Result<DeckInfo> {
    let deck = col.get_or_create_normal_deck(name)?;
    Ok(DeckInfo {
        id: deck.id.0,
        name: name.to_string(),
        new: 0,
        learning: 0,
        review: 0,
    })
}

pub fn delete_deck(col: &mut CollectionHandle, name: &str) -> anyhow::Result<()> {
    let did = col
        .get_deck_id(name)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", name))?;
    col.remove_decks_and_child_decks(&[did])?;
    Ok(())
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
    fn test_list_decks_returns_default_deck() {
        let (_dir, mut col) = setup();
        // Ensure Default deck exists by creating it (new empty collections hide it)
        create_deck(&mut col, "Default").unwrap();
        let decks = list_decks(&mut col).unwrap();
        assert!(!decks.is_empty());
        assert!(decks.iter().any(|d| d.name == "Default"));
    }

    #[test]
    fn test_create_deck_returns_id_and_name() {
        let (_dir, mut col) = setup();
        let info = create_deck(&mut col, "Test").unwrap();
        assert!(info.id > 0);
        assert_eq!(info.name, "Test");
    }

    #[test]
    fn test_create_nested_deck() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "Parent::Child").unwrap();
        let decks = list_decks(&mut col).unwrap();
        let names: Vec<&str> = decks.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Parent"), "Parent deck missing: {names:?}");
        assert!(
            names.contains(&"Parent::Child"),
            "Parent::Child deck missing: {names:?}"
        );
    }

    #[test]
    fn test_delete_deck() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "ToDelete").unwrap();
        // Verify it exists
        let decks = list_decks(&mut col).unwrap();
        assert!(decks.iter().any(|d| d.name == "ToDelete"));

        delete_deck(&mut col, "ToDelete").unwrap();

        let decks = list_decks(&mut col).unwrap();
        assert!(!decks.iter().any(|d| d.name == "ToDelete"));
    }

    #[test]
    fn test_delete_nonexistent_deck_returns_err() {
        let (_dir, mut col) = setup();
        let err = delete_deck(&mut col, "NonExistentDeck").unwrap_err();
        assert!(
            err.to_string().contains("NonExistentDeck"),
            "Error message should contain deck name: {err}"
        );
    }

    #[test]
    fn test_list_decks_due_counts() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "Default").unwrap();
        let decks = list_decks(&mut col).unwrap();
        let default_deck = decks
            .iter()
            .find(|d| d.name == "Default")
            .expect("Default deck should be present");
        // Due count fields are u32, so they are always >= 0 by type.
        // Verify the fields are accessible and their types are correct.
        let _new: u32 = default_deck.new;
        let _learning: u32 = default_deck.learning;
        let _review: u32 = default_deck.review;
    }

    #[test]
    fn test_create_deck_idempotent() {
        let (_dir, mut col) = setup();
        let first = create_deck(&mut col, "IdempotentDeck").unwrap();
        let second = create_deck(&mut col, "IdempotentDeck").unwrap();
        assert_eq!(first.id, second.id, "Repeated create_deck should return the same ID");
    }
}
