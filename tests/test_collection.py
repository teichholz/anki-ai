import pytest
from unittest.mock import MagicMock, patch

from anki_ai.collection import open_collection


def test_collection_closes_on_success():
    mock_col = MagicMock()
    with patch("anki_ai.collection.Collection", return_value=mock_col):
        with open_collection("/fake/path"):
            pass
    mock_col.close.assert_called_once()


def test_collection_closes_on_exception():
    mock_col = MagicMock()
    with patch("anki_ai.collection.Collection", return_value=mock_col):
        with pytest.raises(RuntimeError):
            with open_collection("/fake/path"):
                raise RuntimeError("boom")
    mock_col.close.assert_called_once()
