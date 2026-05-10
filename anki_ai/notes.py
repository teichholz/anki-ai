from anki.collection import Collection
from anki.notes import NoteId


def add_note(
    col: Collection, deck_name: str, note_type: str, fields: dict[str, str]
) -> int:
    model = col.models.by_name(note_type)
    if model is None:
        raise ValueError(f"Note type '{note_type}' not found.")
    deck = col.decks.by_name(deck_name)
    if deck is None:
        raise ValueError(f"Deck '{deck_name}' not found.")
    note = col.new_note(model)
    valid = note.keys()
    for name, value in fields.items():
        if name not in valid:
            raise ValueError(f"Field '{name}' not in '{note_type}'. Valid: {valid}")
        note[name] = value
    col.add_note(note, deck["id"])
    return int(note.id)


def get_note(col: Collection, note_id: int) -> dict:
    nids = col.find_notes(f"nid:{note_id}")
    if not nids:
        raise ValueError(f"Note {note_id} not found.")
    note = col.get_note(NoteId(note_id))
    note_type = note.note_type()
    assert note_type is not None
    field_names = [f["name"] for f in note_type["flds"]]
    return {
        "id": int(note.id),
        "type": note_type["name"],
        "fields": dict(zip(field_names, note.fields)),
        "tags": note.tags,
    }


def delete_note(col: Collection, note_id: int) -> None:
    nids = col.find_notes(f"nid:{note_id}")
    if not nids:
        raise ValueError(f"Note {note_id} not found.")
    col.remove_notes([NoteId(note_id)])


def update_note(col: Collection, note_id: int, fields: dict[str, str]) -> dict:
    nids = col.find_notes(f"nid:{note_id}")
    if not nids:
        raise ValueError(f"Note {note_id} not found.")
    note = col.get_note(NoteId(note_id))
    valid = note.keys()
    for name, value in fields.items():
        if name not in valid:
            raise ValueError(f"Field '{name}' not found. Valid: {valid}")
        note[name] = value
    col.update_note(note)
    note_type = note.note_type()
    assert note_type is not None
    field_names = [f["name"] for f in note_type["flds"]]
    return {
        "id": note_id,
        "fields": dict(zip(field_names, note.fields)),
        "tags": note.tags,
    }


def search_notes(col: Collection, query: str) -> list[dict]:
    nids = col.find_notes(query)
    results = []
    for nid in nids:
        note = col.get_note(nid)
        note_type = note.note_type()
        assert note_type is not None
        field_names = [f["name"] for f in note_type["flds"]]
        results.append(
            {
                "id": int(nid),
                "type": note_type["name"],
                "fields": dict(zip(field_names, note.fields)),
                "tags": note.tags,
            }
        )
    return results
