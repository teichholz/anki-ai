from pathlib import Path

from anki.collection import Collection


def add_media_file(col: Collection, path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(f"File not found: {path}")
    return col.media.add_file(str(path))
