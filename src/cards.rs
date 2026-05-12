use anki::card::CardId;
use anki::search::SortMode;
use anki_proto::scheduler::bury_or_suspend_cards_request::Mode as BuryOrSuspendMode;
use serde::Serialize;

use crate::collection::CollectionHandle;

#[derive(Debug, Serialize)]
pub struct CardInfo {
    pub id: i64,
    pub note_id: i64,
    pub deck_id: i64,
    pub template: u16,
    #[serde(rename = "type")]
    pub card_type: String,
    pub queue: String,
    pub due: i32,
    pub interval_days: u32,
    pub ease_pct: u16,
    pub reviews: u32,
    pub lapses: u32,
}

/// Map the `ctype` integer from the proto to a human-readable string.
/// Values match `CardType` enum: 0=New, 1=Learn, 2=Review, 3=Relearn.
fn card_type_str(ct: u32) -> &'static str {
    match ct {
        0 => "new",
        1 => "learning",
        2 => "review",
        3 => "relearning",
        _ => "unknown",
    }
}

/// Map the `queue` integer from the proto to a human-readable string.
/// Values match `CardQueue` enum (i8): -3=SchedBuried, -2=UserBuried,
/// -1=Suspended, 0=New, 1=Learn, 2=Review, 3=DayLearn, 4=PreviewRepeat.
fn card_queue_str(cq: i32) -> &'static str {
    match cq {
        -3 => "sched-buried",
        -2 => "user-buried",
        -1 => "suspended",
        0 => "new",
        1 => "learning",
        2 => "review",
        3 => "day-learning",
        4 => "preview",
        _ => "unknown",
    }
}

fn card_to_info(col: &mut CollectionHandle, cid: CardId) -> anyhow::Result<CardInfo> {
    // Convert via the proto to avoid accessing pub(crate) fields.
    let card: anki_proto::cards::Card = col
        .storage
        .get_card(cid)?
        .ok_or_else(|| anyhow::anyhow!("Card {} not found", cid.0))?
        .into();
    Ok(CardInfo {
        id: card.id,
        note_id: card.note_id,
        deck_id: card.deck_id,
        template: card.template_idx as u16,
        card_type: card_type_str(card.ctype).to_owned(),
        queue: card_queue_str(card.queue).to_owned(),
        due: card.due,
        interval_days: card.interval,
        ease_pct: (card.ease_factor / 10) as u16,
        reviews: card.reps,
        lapses: card.lapses,
    })
}

pub fn find_cards(col: &mut CollectionHandle, query: &str) -> anyhow::Result<Vec<CardInfo>> {
    let cids = col.search_cards(query, SortMode::NoOrder)?;
    cids.into_iter().map(|cid| card_to_info(col, cid)).collect()
}

pub fn get_card_info(col: &mut CollectionHandle, card_id: i64) -> anyhow::Result<CardInfo> {
    card_to_info(col, CardId(card_id))
}

pub fn suspend_cards(col: &mut CollectionHandle, card_ids: &[i64]) -> anyhow::Result<usize> {
    let cids: Vec<CardId> = card_ids.iter().map(|&id| CardId(id)).collect();
    let result = col.bury_or_suspend_cards(&cids, BuryOrSuspendMode::Suspend)?;
    Ok(result.output)
}

pub fn unsuspend_cards(col: &mut CollectionHandle, card_ids: &[i64]) -> anyhow::Result<()> {
    let cids: Vec<CardId> = card_ids.iter().map(|&id| CardId(id)).collect();
    col.unbury_or_unsuspend_cards(&cids)?;
    Ok(())
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

    fn add_basic_note(col: &mut CollectionHandle) -> i64 {
        let mut fields = HashMap::new();
        fields.insert("Front".to_string(), "TestFront".to_string());
        fields.insert("Back".to_string(), "TestBack".to_string());
        add_note(col, "Default", "Basic", &fields).unwrap()
    }

    #[test]
    fn test_find_cards_returns_empty_for_no_match() {
        let (_dir, mut col) = setup();
        let results = find_cards(&mut col, "front:NonExistentXYZ123").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_card_info_returns_correct_fields() {
        let (_dir, mut col) = setup();
        let note_id = add_basic_note(&mut col);

        // Find the card created for the note
        let cids = col
            .search_cards(&format!("nid:{note_id}"), SortMode::NoOrder)
            .unwrap();
        assert!(!cids.is_empty(), "Expected at least one card for the note");
        let cid = cids[0].0;

        let info = get_card_info(&mut col, cid).unwrap();
        assert_eq!(info.id, cid);
        assert_eq!(info.note_id, note_id);
        assert!(info.deck_id > 0);
        assert_eq!(info.card_type, "new");
        assert_eq!(info.queue, "new");
    }

    #[test]
    fn test_suspend_and_unsuspend_cards() {
        let (_dir, mut col) = setup();
        let note_id = add_basic_note(&mut col);

        let cids = col
            .search_cards(&format!("nid:{note_id}"), SortMode::NoOrder)
            .unwrap();
        assert!(!cids.is_empty());
        let cid = cids[0].0;

        // Suspend the card
        let count = suspend_cards(&mut col, &[cid]).unwrap();
        assert_eq!(count, 1);

        // Verify it is suspended
        let info = get_card_info(&mut col, cid).unwrap();
        assert_eq!(info.queue, "suspended");

        // Unsuspend
        unsuspend_cards(&mut col, &[cid]).unwrap();

        // Verify it is no longer suspended
        let info = get_card_info(&mut col, cid).unwrap();
        assert_ne!(info.queue, "suspended");
    }

    #[test]
    fn test_find_cards_finds_by_query() {
        let (_dir, mut col) = setup();
        add_basic_note(&mut col);

        let results = find_cards(&mut col, "deck:Default").unwrap();
        assert!(!results.is_empty(), "Expected at least one card in Default deck");

        // Verify the result has expected structure
        let card = &results[0];
        assert!(card.id > 0);
        assert!(card.note_id > 0);
        assert!(card.deck_id > 0);
    }
}
