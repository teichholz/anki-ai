from anki.cards import Card, CardId
from anki.collection import Collection

_TYPE = {0: "new", 1: "learning", 2: "review", 3: "relearning"}
_QUEUE = {
    -3: "sched-buried",
    -2: "user-buried",
    -1: "suspended",
    0: "new",
    1: "learning",
    2: "review",
    3: "day-learning",
    4: "preview",
}


def _card_to_dict(card: Card) -> dict:
    return {
        "id": int(card.id),
        "note_id": int(card.nid),
        "deck_id": int(card.did),
        "template": card.ord,
        "type": _TYPE.get(int(card.type), str(card.type)),
        "queue": _QUEUE.get(int(card.queue), str(card.queue)),
        "due": card.due,
        "interval_days": card.ivl,
        "ease_pct": card.factor // 10,
        "reviews": card.reps,
        "lapses": card.lapses,
    }


def find_cards(col: Collection, query: str) -> list[dict]:
    cids = col.find_cards(query)
    return [_card_to_dict(col.get_card(cid)) for cid in cids]


def get_card_info(col: Collection, card_id: int) -> dict:
    card = col.get_card(CardId(card_id))
    return _card_to_dict(card)


def suspend_cards(col: Collection, card_ids: list[int]) -> int:
    result = col.sched.suspend_cards([CardId(i) for i in card_ids])
    return result.count


def unsuspend_cards(col: Collection, card_ids: list[int]) -> None:
    col.sched.unsuspend_cards([CardId(i) for i in card_ids])
