use anki::services::MediaService;
use anki_proto::media::AddMediaFileRequest;

use crate::collection::CollectionHandle;

/// Add a media file to the collection's media folder.
///
/// Reads the file at `path`, uploads it under the file's own name, and
/// returns the name Anki ultimately stored it under (Anki may de-duplicate
/// by appending a suffix).
pub fn add_media_file(
    col: &mut CollectionHandle,
    path: &std::path::Path,
) -> anyhow::Result<String> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let data = std::fs::read(path)?;
    let desired_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Path has no filename: {}", path.display()))?
        .to_string_lossy()
        .into_owned();

    let result = col.add_media_file(AddMediaFileRequest { desired_name, data })?;
    Ok(result.val)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anki::collection::CollectionBuilder;
    use tempfile::TempDir;

    use super::*;
    use crate::collection::CollectionHandle;

    /// Open a collection with media paths configured (required for media ops).
    fn setup_with_media() -> (TempDir, CollectionHandle) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("collection.anki2");
        let col = CollectionBuilder::new(&path)
            .with_desktop_media_paths()
            .build()
            .unwrap();
        (dir, CollectionHandle::from_collection(col))
    }

    #[test]
    fn test_add_media_file() {
        let (dir, mut col) = setup_with_media();

        // Create a temporary media file
        let media_path = dir.path().join("test_image.jpg");
        let mut f = std::fs::File::create(&media_path).unwrap();
        f.write_all(b"fake jpeg content").unwrap();

        let stored_name = add_media_file(&mut col, &media_path).unwrap();
        assert!(!stored_name.is_empty());
    }

    #[test]
    fn test_add_media_file_not_found() {
        let (_dir, mut col) = setup_with_media();
        let nonexistent = std::path::Path::new("/tmp/this_file_does_not_exist_xyz.jpg");
        let err = add_media_file(&mut col, nonexistent).unwrap_err();
        assert!(err.to_string().contains("File not found"));
    }
}
