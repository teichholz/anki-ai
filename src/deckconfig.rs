use anki::deckconfig::DeckConfig;
use anki::deckconfig::UpdateDeckConfigsRequest;
use anki_proto::deck_config::UpdateDeckConfigsMode;
use anyhow::anyhow;
use serde::Serialize;

use crate::collection::CollectionHandle;

#[derive(Debug, Serialize)]
pub struct DeckConfigInfo {
    pub config_id: i64,
    pub config_name: String,
    pub new_per_day: u32,
    pub reviews_per_day: u32,
}

fn resolve_config(col: &mut CollectionHandle, deck_name: &str) -> anyhow::Result<DeckConfig> {
    let did = col
        .get_deck_id(deck_name)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", deck_name))?;
    let deck = col
        .get_deck(did)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", deck_name))?;
    let config_id = deck
        .config_id()
        .ok_or_else(|| anyhow!("'{}' is a filtered deck and has no study config.", deck_name))?;
    col.get_deck_config(config_id, true)?
        .ok_or_else(|| anyhow!("Deck config not found."))
}

pub fn get_deck_config(
    col: &mut CollectionHandle,
    deck_name: &str,
) -> anyhow::Result<DeckConfigInfo> {
    let config = resolve_config(col, deck_name)?;
    Ok(DeckConfigInfo {
        config_id: config.id.0,
        config_name: config.name.clone(),
        new_per_day: config.inner.new_per_day,
        reviews_per_day: config.inner.reviews_per_day,
    })
}

pub fn set_deck_config(
    col: &mut CollectionHandle,
    deck_name: &str,
    new_per_day: Option<u32>,
    reviews_per_day: Option<u32>,
) -> anyhow::Result<DeckConfigInfo> {
    let did = col
        .get_deck_id(deck_name)?
        .ok_or_else(|| anyhow!("Deck '{}' not found.", deck_name))?;

    let mut config = resolve_config(col, deck_name)?;

    if let Some(v) = new_per_day {
        config.inner.new_per_day = v;
    }
    if let Some(v) = reviews_per_day {
        config.inner.reviews_per_day = v;
    }

    let info = DeckConfigInfo {
        config_id: config.id.0,
        config_name: config.name.clone(),
        new_per_day: config.inner.new_per_day,
        reviews_per_day: config.inner.reviews_per_day,
    };

    col.update_deck_configs(UpdateDeckConfigsRequest {
        target_deck_id: did,
        configs: vec![config],
        removed_config_ids: vec![],
        mode: UpdateDeckConfigsMode::Normal,
        card_state_customizer: String::new(),
        limits: Default::default(),
        new_cards_ignore_review_limit: false,
        apply_all_parent_limits: false,
        fsrs: false,
        fsrs_reschedule: false,
        fsrs_health_check: false,
    })?;

    Ok(info)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::collection::open_collection;
    use crate::decks::create_deck;

    fn setup() -> (TempDir, CollectionHandle) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        let col = open_collection(Some(&path)).unwrap();
        (dir, col)
    }

    #[test]
    fn test_get_deck_config_defaults() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "Default").unwrap();
        let info = get_deck_config(&mut col, "Default").unwrap();
        assert_eq!(info.new_per_day, 20);
        assert_eq!(info.reviews_per_day, 200);
    }

    #[test]
    fn test_set_deck_config_new_per_day() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "Default").unwrap();
        let info = set_deck_config(&mut col, "Default", Some(5), None).unwrap();
        assert_eq!(info.new_per_day, 5);
        assert_eq!(info.reviews_per_day, 200);

        // Verify persisted
        let read_back = get_deck_config(&mut col, "Default").unwrap();
        assert_eq!(read_back.new_per_day, 5);
    }

    #[test]
    fn test_set_deck_config_reviews_per_day() {
        let (_dir, mut col) = setup();
        create_deck(&mut col, "Default").unwrap();
        let info = set_deck_config(&mut col, "Default", None, Some(50)).unwrap();
        assert_eq!(info.new_per_day, 20);
        assert_eq!(info.reviews_per_day, 50);
    }

    #[test]
    fn test_get_deck_config_nonexistent_errors() {
        let (_dir, mut col) = setup();
        let err = get_deck_config(&mut col, "Ghost").unwrap_err();
        assert!(err.to_string().contains("Ghost"));
    }
}
