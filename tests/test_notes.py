import pytest
from unittest.mock import MagicMock

from anki_ai.notes import add_note, delete_note, get_note, search_notes, update_note


def _make_col():
    col = MagicMock()
    model = {"name": "Basic", "flds": [{"name": "Front"}, {"name": "Back"}]}
    col.models.by_name.return_value = model

    deck = {"id": 1}
    col.decks.by_name.return_value = deck

    note = MagicMock()
    note.id = 99
    note.fields = ["hello", "world"]
    note.tags = []
    note.keys.return_value = ["Front", "Back"]
    note.__contains__ = lambda self, key: key in ["Front", "Back"]
    note.note_type.return_value = model
    col.new_note.return_value = note
    col.get_note.return_value = note
    col.find_notes.return_value = [99]
    return col


def test_add_note_sets_fields():
    col = _make_col()
    note_id = add_note(col, "Default", "Basic", {"Front": "hello", "Back": "world"})
    assert note_id == 99
    col.add_note.assert_called_once()
    note = col.new_note.return_value
    note.__setitem__.assert_any_call("Front", "hello")
    note.__setitem__.assert_any_call("Back", "world")


def test_add_note_deck_not_found():
    col = _make_col()
    col.decks.by_name.return_value = None
    with pytest.raises(ValueError, match="not found"):
        add_note(col, "NoSuchDeck", "Basic", {"Front": "q"})


def test_add_note_model_not_found():
    col = _make_col()
    col.models.by_name.return_value = None
    with pytest.raises(ValueError, match="not found"):
        add_note(col, "Default", "NoSuchType", {"Front": "q"})


def test_add_note_invalid_field():
    col = _make_col()
    with pytest.raises(ValueError, match="Field 'Extra'"):
        add_note(col, "Default", "Basic", {"Extra": "x"})


def test_get_note_returns_dict():
    col = _make_col()
    result = get_note(col, 99)
    assert result["id"] == 99
    assert result["type"] == "Basic"
    assert result["fields"] == {"Front": "hello", "Back": "world"}
    assert result["tags"] == []


def test_get_note_not_found():
    col = _make_col()
    col.find_notes.return_value = []
    with pytest.raises(ValueError, match="not found"):
        get_note(col, 999)


def test_search_notes_returns_dicts():
    col = _make_col()
    results = search_notes(col, "deck:Default")
    assert len(results) == 1
    assert results[0]["id"] == 99
    assert results[0]["type"] == "Basic"
    assert results[0]["fields"] == {"Front": "hello", "Back": "world"}
    assert results[0]["tags"] == []


def test_search_notes_empty_result():
    col = _make_col()
    col.find_notes.return_value = []
    assert search_notes(col, "nothing") == []


def test_delete_note_calls_remove():
    col = _make_col()
    delete_note(col, 99)
    col.remove_notes.assert_called_once()


def test_delete_note_not_found():
    col = _make_col()
    col.find_notes.return_value = []
    with pytest.raises(ValueError, match="not found"):
        delete_note(col, 999)


def test_update_note_front_only():
    col = _make_col()
    result = update_note(col, 99, {"Front": "new front"})
    note = col.get_note.return_value
    note.__setitem__.assert_called_once_with("Front", "new front")
    col.update_note.assert_called_once_with(note)
    assert result["id"] == 99


def test_update_note_not_found():
    col = _make_col()
    col.find_notes.return_value = []
    with pytest.raises(ValueError, match="not found"):
        update_note(col, 999, {"Front": "x"})


def test_update_note_invalid_field():
    col = _make_col()
    with pytest.raises(ValueError, match="Field 'Extra'"):
        update_note(col, 99, {"Extra": "x"})
