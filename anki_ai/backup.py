import shutil
from datetime import datetime
from pathlib import Path


def _snapshots_dir(collection_path: str) -> Path:
    return Path(collection_path).parent / "snapshots"


def create_snapshot(collection_path: str) -> str:
    col_path = Path(collection_path)
    if not col_path.exists():
        raise FileNotFoundError(f"Collection not found: {collection_path}")
    snap_dir = _snapshots_dir(collection_path)
    snap_dir.mkdir(exist_ok=True)
    timestamp = datetime.now().strftime("%Y-%m-%d-%H.%M.%S")
    dest = snap_dir / f"snapshot-{timestamp}.anki2"
    shutil.copy2(col_path, dest)
    # Also copy WAL/SHM if present so the snapshot is self-consistent
    for suffix in ("-wal", "-shm"):
        sidecar = col_path.with_suffix(".anki2" + suffix)
        if sidecar.exists():
            shutil.copy2(sidecar, dest.with_suffix(".anki2" + suffix))
    return str(dest)


def list_snapshots(collection_path: str) -> list[dict]:
    snap_dir = _snapshots_dir(collection_path)
    if not snap_dir.exists():
        return []
    snaps = sorted(snap_dir.glob("snapshot-*.anki2"), reverse=True)
    return [{"name": s.name, "path": str(s), "bytes": s.stat().st_size} for s in snaps]


def restore_snapshot(collection_path: str, snapshot: str) -> str:
    snap_dir = _snapshots_dir(collection_path)
    # Accept bare filename or full path
    candidate = snap_dir / snapshot if "/" not in snapshot else Path(snapshot)
    if not candidate.exists():
        raise FileNotFoundError(f"Snapshot not found: {snapshot}")
    col_path = Path(collection_path)
    shutil.copy2(candidate, col_path)
    # Remove stale WAL/SHM so SQLite starts clean
    for suffix in ("-wal", "-shm"):
        stale = col_path.with_suffix(".anki2" + suffix)
        if stale.exists():
            stale.unlink()
    return str(candidate)
