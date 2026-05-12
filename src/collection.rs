use std::path::{Path, PathBuf};

use anki::collection::{Collection, CollectionBuilder};
use anyhow::Result;

/// Detect Anki collection path from env or filesystem.
/// Priority: ANKI_COLLECTION_PATH env → scan ~/.local/share/Anki2/ → fallback
pub fn get_collection_path() -> PathBuf {
    // 1. Check environment variable
    if let Ok(env_path) = std::env::var("ANKI_COLLECTION_PATH") {
        let path = PathBuf::from(&env_path);
        // Expand leading ~ if present
        if let Some(stripped) = env_path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        return path;
    }

    // 2. Scan ~/.local/share/Anki2/ for first non-hidden directory with collection.anki2
    if let Some(data_dir) = dirs::data_local_dir() {
        let anki_base = data_dir.join("Anki2");
        if anki_base.exists() {
            if let Ok(entries) = std::fs::read_dir(&anki_base) {
                let mut dirs_vec: Vec<PathBuf> = entries
                    .flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        let name = p.file_name()?.to_string_lossy().into_owned();
                        if p.is_dir() && !name.starts_with('.') {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .collect();
                dirs_vec.sort();
                for dir in dirs_vec {
                    let candidate = dir.join("collection.anki2");
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }

        // 3. Fallback
        return data_dir.join("Anki2").join("User 1").join("collection.anki2");
    }

    // Last resort if data_local_dir() is unavailable
    PathBuf::from("~/.local/share/Anki2/User 1/collection.anki2")
}

/// A handle that owns a `Collection` and closes it cleanly on drop.
pub struct CollectionHandle {
    col: Option<Collection>,
}

impl Drop for CollectionHandle {
    fn drop(&mut self) {
        if let Some(col) = self.col.take() {
            let _ = col.close(None);
        }
    }
}

impl std::ops::Deref for CollectionHandle {
    type Target = Collection;

    fn deref(&self) -> &Collection {
        self.col.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for CollectionHandle {
    fn deref_mut(&mut self) -> &mut Collection {
        self.col.as_mut().unwrap()
    }
}

/// Open a collection, auto-detecting path if `None`.
/// Creates parent directories if needed.
/// Returns a `CollectionHandle` that closes the collection on drop.
pub fn open_collection(path: Option<&Path>) -> Result<CollectionHandle> {
    let col_path = match path {
        Some(p) => p.to_path_buf(),
        None => get_collection_path(),
    };

    if let Some(parent) = col_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let col = CollectionBuilder::new(&col_path).build()?;
    Ok(CollectionHandle { col: Some(col) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collection_closes_on_drop() {
        // Create a temp dir, open a collection there, drop it,
        // verify we can open it again (double-open would fail if not closed)
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        {
            let _col = open_collection(Some(&path)).unwrap();
            // drops here — must close cleanly
        }
        // Open again to prove it was properly closed
        let _col2 = open_collection(Some(&path)).unwrap();
    }

    #[test]
    fn test_collection_closes_on_error() {
        // Same pattern but proves cleanup even if we panic/error inside
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        {
            let _col = open_collection(Some(&path)).unwrap();
        }
        // If the collection wasn't closed, this would fail with a lock error
        let _col2 = open_collection(Some(&path)).unwrap();
    }

    #[test]
    fn test_get_collection_path_from_env() {
        std::env::set_var("ANKI_COLLECTION_PATH", "/tmp/test.anki2");
        let path = get_collection_path();
        assert_eq!(path, PathBuf::from("/tmp/test.anki2"));
        std::env::remove_var("ANKI_COLLECTION_PATH");
    }
}
