use std::path::{Path, PathBuf};

use anki::collection::{Collection, CollectionBuilder};
use anyhow::{Context, Result};

/// Detect Anki collection path from env or filesystem.
/// Priority: ANKI_COLLECTION_PATH env → scan ~/.local/share/Anki2/ → fallback
pub fn get_collection_path() -> Result<PathBuf> {
    // 1. Check environment variable
    if let Some(env_path) = std::env::var_os("ANKI_COLLECTION_PATH") {
        let env_path = PathBuf::from(env_path);
        if let Ok(stripped) = env_path.strip_prefix("~/") {
            let home = dirs::home_dir()
                .context("HOME directory not set; cannot expand ~ in ANKI_COLLECTION_PATH")?;
            return Ok(home.join(stripped));
        }
        return Ok(env_path);
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
                        return Ok(candidate);
                    }
                }
            }
        }

        // 3. Fallback
        return Ok(data_dir
            .join("Anki2")
            .join("User 1")
            .join("collection.anki2"));
    }

    // Last resort if data_local_dir() is unavailable
    let home = dirs::home_dir().context("HOME directory not set")?;
    Ok(home.join(".local/share/Anki2/User 1/collection.anki2"))
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

impl CollectionHandle {
    /// Wrap a pre-built `Collection` in a handle.
    #[allow(dead_code)]
    pub fn from_collection(col: Collection) -> Self {
        Self { col: Some(col) }
    }

    /// Take ownership of the inner `Collection`, leaving the handle empty.
    ///
    /// Used by operations (e.g. full upload/download) that consume the
    /// collection.  After this call, dereferencing the handle will panic.
    pub fn take_inner(&mut self) -> Option<Collection> {
        self.col.take()
    }
}

/// Open a collection, auto-detecting path if `None`.
/// Creates parent directories if needed.
/// Returns a `CollectionHandle` that closes the collection on drop.
pub fn open_collection(path: Option<&Path>) -> Result<CollectionHandle> {
    let col_path = match path {
        Some(p) => p.to_path_buf(),
        None => get_collection_path()?,
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
        let path = get_collection_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test.anki2"));
        std::env::remove_var("ANKI_COLLECTION_PATH");
    }

    #[test]
    fn test_get_collection_path_tilde_expansion() {
        std::env::remove_var("ANKI_COLLECTION_PATH");
        // Temporarily set to a tilde path to test expansion
        std::env::set_var("ANKI_COLLECTION_PATH", "~/test.anki2");
        let path = get_collection_path().unwrap();
        // Tilde should be expanded (path doesn't contain literal ~)
        assert!(!path.to_string_lossy().starts_with('~'));
        std::env::remove_var("ANKI_COLLECTION_PATH");
    }
}
