use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anki::sync::collection::normal::SyncActionRequired;
use anki::sync::login::SyncAuth;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::collection::CollectionHandle;

// ---------------------------------------------------------------------------
// Auth path helpers
// ---------------------------------------------------------------------------

/// Return the path to the auth token file.
/// In tests, ANKI_AUTH_PATH overrides the default location.
fn auth_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("ANKI_AUTH_PATH") {
        return Ok(PathBuf::from(p));
    }
    let home = dirs::home_dir().context("HOME directory not set")?;
    Ok(home.join(".config/anki-ai/auth.json"))
}

// ---------------------------------------------------------------------------
// Serialisation types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct AuthFile {
    hkey: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load the AnkiWeb host-key from `~/.config/anki-ai/auth.json`.
/// Returns `Err` when the file is absent or the `hkey` field is missing/empty.
pub fn load_hkey() -> Result<String> {
    load_hkey_from(&auth_path()?)
}

/// Save the AnkiWeb host-key to `~/.config/anki-ai/auth.json` (mode 0o600).
pub fn save_hkey(hkey: &str) -> Result<()> {
    save_hkey_to(&auth_path()?, hkey)
}

// ---------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------

/// Perform an AnkiWeb sync.
///
/// * `col`        – open collection handle
/// * `sync_media` – whether to sync media after the collection sync
/// * `upload`     – when a full sync is needed, prefer upload (vs download)
///
/// Prints a one-line status message on success.  On full-sync the collection
/// is consumed internally; the handle is left with its inner `Option` set to
/// `None`.  Callers that need the collection afterwards should re-open it.
pub async fn run_sync(col: &mut CollectionHandle, sync_media: bool, upload: bool) -> Result<()> {
    let hkey = load_hkey()?;
    let client = reqwest::Client::new();

    let auth = SyncAuth {
        hkey,
        endpoint: None,
        io_timeout_secs: None,
    };

    // --- normal (delta) sync ------------------------------------------------
    let output = col
        .normal_sync(auth.clone(), client.clone())
        .await
        .context("normal sync failed")?;

    // Possibly update endpoint from server response
    let auth = if let Some(new_ep) = &output.new_endpoint {
        SyncAuth {
            hkey: auth.hkey.clone(),
            endpoint: new_ep.parse().ok(),
            io_timeout_secs: auth.io_timeout_secs,
        }
    } else {
        auth
    };

    match output.required {
        SyncActionRequired::NoChanges => {
            println!("Already up to date.");
        }
        SyncActionRequired::NormalSyncRequired => {
            println!("Sync complete.");
        }
        SyncActionRequired::FullSyncRequired {
            upload_ok: _,
            download_ok,
        } => {
            let do_upload = upload || !download_ok;
            let direction = if do_upload { "upload" } else { "download" };

            // full_upload / full_download consume the Collection, so we take
            // it out of the CollectionHandle Option.
            let inner = col
                .take_inner()
                .context("collection was already closed before full sync")?;

            if do_upload {
                inner
                    .full_upload(auth.clone(), client.clone())
                    .await
                    .context("full upload failed")?;
            } else {
                inner
                    .full_download(auth.clone(), client.clone())
                    .await
                    .context("full download failed")?;
            }

            println!("Full {direction} complete.");
            // col.col is now None; caller must re-open.
            return Ok(());
        }
    }

    // --- media sync ---------------------------------------------------------
    if sync_media {
        // `server_media_usn` from SyncOutput is pub(crate) so we cannot access
        // it here.  Passing None causes MediaSyncer to fetch the server USN via
        // a begin_sync() request — one extra round-trip but functionally identical.
        let media_mgr = col
            .media()
            .context("cannot open media manager for media sync")?;
        let progress = col.new_progress_handler();
        media_mgr
            .sync_media(progress, auth, client, None)
            .await
            .context("media sync failed")?;
        println!("Media sync complete.");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers used by tests
// ---------------------------------------------------------------------------

/// Load hkey from a specific path (for tests and internal use).
fn load_hkey_from(path: &std::path::Path) -> Result<String> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("auth file not found: {}", path.display()))?;
    let auth: serde_json::Value = serde_json::from_str(&data)
        .with_context(|| format!("invalid JSON in auth file: {}", path.display()))?;
    let hkey = auth
        .get("hkey")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .with_context(|| {
            format!(
                "\"hkey\" field missing or empty in auth file: {}",
                path.display()
            )
        })?;
    Ok(hkey.to_owned())
}

/// Save hkey to a specific path (for tests and internal use).
fn save_hkey_to(path: &std::path::Path, hkey: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory: {}", parent.display()))?;
    }
    let contents = serde_json::to_string(&AuthFile {
        hkey: hkey.to_owned(),
    })?;
    std::fs::write(path, contents)
        .with_context(|| format!("cannot write auth file: {}", path.display()))?;
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, perms)
        .with_context(|| format!("cannot set permissions on: {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_hkey_missing_file_returns_err() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("auth.json");
        // File does not exist.
        let err = load_hkey_from(&path).unwrap_err();
        assert!(err.to_string().contains("auth file not found"), "{err}");
    }

    #[test]
    fn test_save_and_load_hkey() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("auth.json");

        save_hkey_to(&path, "abc123").unwrap();
        let loaded = load_hkey_from(&path).unwrap();
        assert_eq!(loaded, "abc123");

        // Verify 0o600 permissions.
        let meta = std::fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "expected mode 0600, got {:o}", mode);
    }

    #[test]
    fn test_load_hkey_missing_field_returns_err() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(&path, r#"{"other_key": "value"}"#).unwrap();

        let err = load_hkey_from(&path).unwrap_err();
        assert!(err.to_string().contains("\"hkey\" field missing"), "{err}");
    }

    #[test]
    fn test_load_hkey_empty_hkey_returns_err() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(&path, r#"{"hkey": ""}"#).unwrap();

        let err = load_hkey_from(&path).unwrap_err();
        assert!(err.to_string().contains("\"hkey\" field missing"), "{err}");
    }

    #[test]
    fn test_save_hkey_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let deep = dir.path().join("a/b/c/auth.json");

        save_hkey_to(&deep, "mykey").unwrap();
        assert!(deep.exists());
    }

    #[test]
    fn test_public_save_and_load_hkey_via_env() {
        // Serialise: only one test touches the env var at a time.
        static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_MUTEX.lock().unwrap();

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("auth.json");
        std::env::set_var("ANKI_AUTH_PATH", path.to_str().unwrap());

        save_hkey("envtest").unwrap();
        let loaded = load_hkey().unwrap();
        assert_eq!(loaded, "envtest");

        std::env::remove_var("ANKI_AUTH_PATH");
    }
}
