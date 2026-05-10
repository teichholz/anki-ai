import pytest
from unittest.mock import MagicMock

from anki_ai.decks import create_deck, delete_deck, list_decks


def _make_tree_node(deck_id, name, new=0, learning=0, review=0, children=None):
    node = MagicMock()
    node.deck_id = deck_id
    node.name = name
    node.new_count = new
    node.learn_count = learning
    node.review_count = review
    node.children = children or []
    return node


def _make_col_with_tree(root):
    col = MagicMock()
    col.sched.deck_due_tree.return_value = root
    return col


def test_list_decks_returns_list_of_dicts():
    root = _make_tree_node(
        0,
        "",
        children=[
            _make_tree_node(1, "Default", new=3, learning=1, review=5),
            _make_tree_node(1715000000000, "Spanish", new=10),
        ],
    )
    col = _make_col_with_tree(root)
    result = list_decks(col)
    assert result == [
        {"id": 1, "name": "Default", "new": 3, "learning": 1, "review": 5},
        {"id": 1715000000000, "name": "Spanish", "new": 10, "learning": 0, "review": 0},
    ]


def test_list_decks_nested():
    vocab = _make_tree_node(3, "Vocab", new=2)
    spanish = _make_tree_node(2, "Spanish", children=[vocab])
    root = _make_tree_node(0, "", children=[spanish])
    col = _make_col_with_tree(root)
    result = list_decks(col)
    assert result[0]["name"] == "Spanish"
    assert result[1]["name"] == "Spanish::Vocab"


def test_list_decks_empty():
    root = _make_tree_node(0, "", children=[])
    col = _make_col_with_tree(root)
    assert list_decks(col) == []


def test_list_decks_tree_none():
    col = MagicMock()
    col.sched.deck_due_tree.return_value = None
    assert list_decks(col) == []


def test_create_deck_returns_id_and_name():
    col = MagicMock()
    col.decks.add_normal_deck_with_name.return_value = MagicMock(id=42)
    result = create_deck(col, "Spanish")
    assert result == {"id": 42, "name": "Spanish"}
    col.decks.add_normal_deck_with_name.assert_called_once_with("Spanish")


def test_delete_deck_calls_remove():
    col = MagicMock()
    col.decks.id_for_name.return_value = 42
    delete_deck(col, "Spanish")
    col.decks.remove.assert_called_once_with([42])


def test_delete_deck_not_found():
    col = MagicMock()
    col.decks.id_for_name.return_value = None
    with pytest.raises(ValueError, match="not found"):
        delete_deck(col, "NoSuchDeck")
