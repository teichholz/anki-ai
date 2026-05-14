use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct SnapshotInfo {
    pub name: String,
    pub path: String,
    pub bytes: u64,
}

fn snapshots_dir(collection_path: &Path) -> PathBuf {
    collection_path
        .parent()
        .expect("collection_path must have a parent directory")
        .join("snapshots")
}

// Proleptic Gregorian calendar decomposition from Unix epoch (seconds).
// Algorithm adapted from https://howardhinnant.github.io/date_algorithms.html

/// Format a `SystemTime` as `YYYY-MM-DD-HH.MM.SS` using UTC-equivalent
/// decomposition via the seconds-since-epoch.
fn format_timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    // Manual calendar decomposition (no external crate needed).
    // Algorithm: convert Unix timestamp to (year, month, day, hour, min, sec).
    let sec = (secs % 60) as u32;
    let mins_total = secs / 60;
    let min = (mins_total % 60) as u32;
    let hours_total = mins_total / 60;
    let hour = (hours_total % 24) as u32;
    let days_total = hours_total / 24; // days since 1970-01-01

    // Shift epoch: days since year 0 (using the proleptic Gregorian calendar).
    // 1970-01-01 = day 719_162 in the proleptic Gregorian calendar
    // (counting from 0001-01-01 as day 1).
    let z = days_total + 719_162; // days since 0000-03-01 (a convenient epoch)
    let era = z / 146_097;
    let doe = z % 146_097; // day of era [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}-{:02}.{:02}.{:02}",
        y, m, d, hour, min, sec
    )
}

pub fn create_snapshot(collection_path: &Path) -> Result<PathBuf> {
    // The collection must be closed before calling this function to ensure
    // consistency. When used through the CLI, open_collection's RAII wrapper
    // guarantees this. The main file is copied first, then WAL/SHM sidecars,
    // matching the behavior of the Python reference implementation.
    if !collection_path.exists() {
        anyhow::bail!("Collection not found: {}", collection_path.display());
    }

    let snap_dir = snapshots_dir(collection_path);
    fs::create_dir_all(&snap_dir)
        .with_context(|| format!("Failed to create snapshots dir: {}", snap_dir.display()))?;

    let timestamp = format_timestamp(SystemTime::now());
    let dest = snap_dir.join(format!("snapshot-{}.anki2", timestamp));

    fs::copy(collection_path, &dest).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            collection_path.display(),
            dest.display()
        )
    })?;

    // Copy WAL/SHM sidecars if present so the snapshot is self-consistent.
    let col_stem = collection_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid collection path"))?
        .to_string_lossy();
    let dest_stem = dest
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid dest path"))?
        .to_string_lossy();
    let wal = collection_path.with_file_name(format!("{}-wal", col_stem));
    let shm = collection_path.with_file_name(format!("{}-shm", col_stem));
    let dest_wal = dest.with_file_name(format!("{}-wal", dest_stem));
    let dest_shm = dest.with_file_name(format!("{}-shm", dest_stem));
    for (sidecar, dest_sidecar) in &[(&wal, &dest_wal), (&shm, &dest_shm)] {
        if sidecar.exists() {
            fs::copy(sidecar, dest_sidecar).with_context(|| {
                format!(
                    "Failed to copy sidecar {} to {}",
                    sidecar.display(),
                    dest_sidecar.display()
                )
            })?;
        }
    }

    Ok(dest)
}

pub fn list_snapshots(collection_path: &Path) -> Result<Vec<SnapshotInfo>> {
    let snap_dir = snapshots_dir(collection_path);
    if !snap_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&snap_dir)
        .with_context(|| format!("Failed to read snapshots dir: {}", snap_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("snapshot-") && n.ends_with(".anki2"))
                .unwrap_or(false)
        })
        .collect();

    // Sort reverse by name (newest first, since names encode timestamps).
    entries.sort_by(|a, b| {
        b.file_name()
            .unwrap_or_default()
            .cmp(a.file_name().unwrap_or_default())
    });

    entries
        .into_iter()
        .map(|p| {
            let bytes = fs::metadata(&p)
                .with_context(|| format!("Failed to stat {}", p.display()))?
                .len();
            Ok(SnapshotInfo {
                name: p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                path: p.to_string_lossy().into_owned(),
                bytes,
            })
        })
        .collect()
}

/// Return the path of the most-recent snapshot, or an error if none exist.
pub fn latest_snapshot(collection_path: &Path) -> Result<PathBuf> {
    let snaps = list_snapshots(collection_path)?;
    snaps
        .into_iter()
        .next()
        .map(|s| PathBuf::from(s.path))
        .ok_or_else(|| anyhow::anyhow!("No snapshots found."))
}

pub fn restore_snapshot(collection_path: &Path, snapshot: &str) -> Result<PathBuf> {
    let snap_dir = snapshots_dir(collection_path);

    let candidate = if snapshot.contains('/') {
        PathBuf::from(snapshot)
    } else {
        snap_dir.join(snapshot)
    };

    if !candidate.exists() {
        anyhow::bail!("Snapshot not found: {}", snapshot);
    }

    fs::copy(&candidate, collection_path).with_context(|| {
        format!(
            "Failed to restore {} to {}",
            candidate.display(),
            collection_path.display()
        )
    })?;

    // Remove stale WAL/SHM so SQLite starts clean.
    let col_stem = collection_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid collection path"))?
        .to_string_lossy();
    let stale_wal = collection_path.with_file_name(format!("{}-wal", col_stem));
    let stale_shm = collection_path.with_file_name(format!("{}-shm", col_stem));
    for stale in &[&stale_wal, &stale_shm] {
        if stale.exists() {
            fs::remove_file(stale)
                .with_context(|| format!("Failed to remove stale sidecar {}", stale.display()))?;
        }
    }

    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_collection(dir: &TempDir) -> PathBuf {
        let col = dir.path().join("collection.anki2");
        fs::write(&col, b"fake anki collection data").unwrap();
        col
    }

    #[test]
    fn test_create_snapshot_creates_file_in_snapshots_dir() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);

        let snap = create_snapshot(&col).unwrap();

        // The snapshot must live inside the snapshots/ subdirectory.
        let snap_dir = dir.path().join("snapshots");
        assert!(snap.starts_with(&snap_dir), "snapshot not in snapshots dir");
        assert!(snap.exists(), "snapshot file does not exist");

        // File name must match pattern snapshot-YYYY-MM-DD-HH.MM.SS.anki2
        let name = snap.file_name().unwrap().to_string_lossy();
        assert!(
            name.starts_with("snapshot-") && name.ends_with(".anki2"),
            "unexpected snapshot name: {name}"
        );

        // Content must match the original.
        let original = fs::read(&col).unwrap();
        let copied = fs::read(&snap).unwrap();
        assert_eq!(original, copied);
    }

    #[test]
    fn test_create_snapshot_copies_wal_shm_sidecars() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);

        // Create WAL and SHM sidecars.
        fs::write(dir.path().join("collection.anki2-wal"), b"wal data").unwrap();
        fs::write(dir.path().join("collection.anki2-shm"), b"shm data").unwrap();

        let snap = create_snapshot(&col).unwrap();

        let snap_wal = snap.with_file_name({
            let mut n = snap.file_name().unwrap().to_os_string();
            n.push("-wal");
            n
        });
        let snap_shm = snap.with_file_name({
            let mut n = snap.file_name().unwrap().to_os_string();
            n.push("-shm");
            n
        });

        assert!(snap_wal.exists(), "WAL sidecar not copied");
        assert!(snap_shm.exists(), "SHM sidecar not copied");
        assert_eq!(fs::read(&snap_wal).unwrap(), b"wal data");
        assert_eq!(fs::read(&snap_shm).unwrap(), b"shm data");
    }

    #[test]
    fn test_create_snapshot_missing_collection_errors() {
        let dir = TempDir::new().unwrap();
        let col = dir.path().join("nonexistent.anki2");
        let result = create_snapshot(&col);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_snapshots_returns_sorted_reverse() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        let snap_dir = dir.path().join("snapshots");
        fs::create_dir_all(&snap_dir).unwrap();

        // Write three fake snapshots with distinct timestamps.
        let names = [
            "snapshot-2024-01-01-10.00.00.anki2",
            "snapshot-2024-01-03-10.00.00.anki2",
            "snapshot-2024-01-02-10.00.00.anki2",
        ];
        for name in &names {
            fs::write(snap_dir.join(name), b"data").unwrap();
        }

        let list = list_snapshots(&col).unwrap();
        assert_eq!(list.len(), 3);

        // Newest first.
        assert_eq!(list[0].name, "snapshot-2024-01-03-10.00.00.anki2");
        assert_eq!(list[1].name, "snapshot-2024-01-02-10.00.00.anki2");
        assert_eq!(list[2].name, "snapshot-2024-01-01-10.00.00.anki2");

        // Check path and bytes fields are populated.
        for info in &list {
            assert!(!info.path.is_empty(), "path should not be empty");
            assert!(info.bytes > 0, "bytes should be non-zero");
        }
    }

    #[test]
    fn test_list_snapshots_empty_dir_returns_empty_vec() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        // No snapshots dir created — should return [].
        let list = list_snapshots(&col).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_snapshots_existing_empty_dir_returns_empty_vec() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        fs::create_dir_all(dir.path().join("snapshots")).unwrap();
        let list = list_snapshots(&col).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_restore_snapshot_copies_file_back() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);

        // Create a snapshot to restore from.
        let snap = create_snapshot(&col).unwrap();

        // Overwrite the collection with different data.
        fs::write(&col, b"corrupted data").unwrap();

        let returned = restore_snapshot(&col, snap.file_name().unwrap().to_str().unwrap()).unwrap();

        assert_eq!(returned, snap);
        let restored = fs::read(&col).unwrap();
        assert_eq!(restored, b"fake anki collection data");
    }

    #[test]
    fn test_restore_snapshot_removes_stale_wal_shm() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);

        // Create a snapshot first.
        let snap = create_snapshot(&col).unwrap();

        // Place stale WAL/SHM next to the collection.
        let wal = dir.path().join("collection.anki2-wal");
        let shm = dir.path().join("collection.anki2-shm");
        fs::write(&wal, b"stale wal").unwrap();
        fs::write(&shm, b"stale shm").unwrap();

        restore_snapshot(&col, snap.file_name().unwrap().to_str().unwrap()).unwrap();

        assert!(!wal.exists(), "stale WAL should have been removed");
        assert!(!shm.exists(), "stale SHM should have been removed");
    }

    #[test]
    fn test_restore_snapshot_accepts_full_path() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        let snap = create_snapshot(&col).unwrap();

        fs::write(&col, b"corrupted").unwrap();

        // Pass the full path (contains '/').
        let returned = restore_snapshot(&col, snap.to_str().unwrap()).unwrap();
        assert_eq!(returned, snap);
        assert_eq!(fs::read(&col).unwrap(), b"fake anki collection data");
    }

    #[test]
    fn test_restore_snapshot_missing_errors() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        let result = restore_snapshot(&col, "snapshot-9999-01-01-00.00.00.anki2");
        assert!(result.is_err());
    }

    #[test]
    fn test_latest_snapshot_returns_newest() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        let snap_dir = dir.path().join("snapshots");
        fs::create_dir_all(&snap_dir).unwrap();

        fs::write(snap_dir.join("snapshot-2024-01-01-10.00.00.anki2"), b"old").unwrap();
        fs::write(snap_dir.join("snapshot-2024-01-03-10.00.00.anki2"), b"new").unwrap();
        fs::write(snap_dir.join("snapshot-2024-01-02-10.00.00.anki2"), b"mid").unwrap();

        let latest = latest_snapshot(&col).unwrap();
        assert!(
            latest.to_string_lossy().contains("2024-01-03"),
            "expected newest snapshot, got: {}",
            latest.display()
        );
    }

    #[test]
    fn test_latest_snapshot_no_snapshots_errors() {
        let dir = TempDir::new().unwrap();
        let col = make_collection(&dir);
        let err = latest_snapshot(&col).unwrap_err();
        assert!(err.to_string().contains("No snapshots"));
    }
}
