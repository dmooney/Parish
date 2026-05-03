//! Disk-backed cache for NLS historic map tiles.
//!
//! [`TileCache`] proxies tile requests from the client through the server
//! rather than having the client hit NLS's S3 bucket directly.  On a cache
//! miss it fetches `https://mapseries-tilesets.s3.amazonaws.com/os/{source_id}/{z}/{x}/{y}.png`,
//! writes the bytes to `cache_dir/{source_id}/{z}/{x}/{y}.png`, and returns
//! them.  On a cache hit it reads the file from disk.
//!
//! The cache directory is resolved once at server startup from config / env
//! (`PARISH_TILE_CACHE_DIR` or `<saves_dir>/tile-cache/`) and stored on
//! `GlobalState` — per CLAUDE.md rule #9 ("resolve runtime paths from
//! explicit config, not the cwd").

use std::path::PathBuf;

use crate::error::ParishError;

/// Upstream NLS S3 base URL for historic OS tile sets.
const NLS_BASE_URL: &str = "https://mapseries-tilesets.s3.amazonaws.com/os";

/// Disk-backed HTTP cache for NLS historic map tiles.
///
/// Constructed once at server boot and stored on `GlobalState` (web) or
/// `AppState` (desktop/CLI).  All clone-and-pass patterns are fine because
/// the inner [`reqwest::Client`] is already `Arc`-backed.
#[derive(Clone)]
pub struct TileCache {
    /// Root directory for cached tile files.
    cache_dir: PathBuf,
    /// Shared HTTP client (reuses connections across requests).
    http: reqwest::Client,
}

impl TileCache {
    /// Creates a new cache rooted at `cache_dir`.
    ///
    /// `cache_dir` must exist or be creatable at request time — the cache
    /// creates subdirectories lazily on first write per source/z/x combination.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            http: reqwest::Client::new(),
        }
    }

    /// Returns a tile from cache (disk hit) or fetches it from NLS S3 (miss).
    ///
    /// The `source_id` must be a safe path component — the route handler
    /// validates it against the registered tile sources before calling here.
    ///
    /// # Errors
    ///
    /// - [`ParishError::Io`] if the on-disk tile cannot be read or written.
    /// - [`ParishError::Network`] if the upstream fetch fails.
    pub async fn get(
        &self,
        source_id: &str,
        z: u32,
        x: u32,
        y: u32,
    ) -> Result<Vec<u8>, ParishError> {
        // Defence-in-depth: reject source_ids that are not safe path components.
        // The route handler already validates against a config allowlist, but
        // CodeQL tracks user-controlled data from HTTP path params into
        // PathBuf::join and the URL format string, so we add an explicit
        // character-class guard here that the analyser can follow statically.
        // Source ids must be non-empty slugs of ASCII alphanumeric chars plus
        // hyphens/underscores — the same constraint the config imposes.
        if source_id.is_empty()
            || !source_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(ParishError::Config(format!(
                "tile source id contains unsafe characters: {source_id:?}"
            )));
        }

        let tile_path = self
            .cache_dir
            .join(source_id)
            .join(z.to_string())
            .join(x.to_string())
            .join(format!("{y}.png"));

        // ── Cache hit ──────────────────────────────────────────────────────
        match tokio::fs::read(&tile_path).await {
            Ok(data) => return Ok(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        // ── Cache miss: fetch from upstream ───────────────────────────────
        let url = format!("{NLS_BASE_URL}/{source_id}/{z}/{x}/{y}.png");
        tracing::debug!(url = %url, "tile cache miss — fetching from NLS S3");

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ParishError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(ParishError::Network(
                resp.error_for_status().unwrap_err().to_string(),
            ));
        }

        let data = resp
            .bytes()
            .await
            .map_err(|e| ParishError::Network(e.to_string()))?
            .to_vec();

        // ── Persist to disk ────────────────────────────────────────────────
        // Create parent dirs lazily (source_id / z / x).
        if let Some(parent) = tile_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let tmp_path = tile_path.with_extension("tmp");
        tokio::fs::write(&tmp_path, &data).await?;
        tokio::fs::rename(&tmp_path, &tile_path).await?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache(dir: &TempDir) -> TileCache {
        TileCache::new(dir.path().to_path_buf())
    }

    #[test]
    fn tile_path_is_deterministic() {
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        let path = cache
            .cache_dir
            .join("roscommon1")
            .join("10")
            .join("500")
            .join("350.png");
        assert!(path.to_str().unwrap().contains("roscommon1/10/500/350.png"));
    }
}
