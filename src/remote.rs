use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

const MAX_REMOTE_BYTES: u64 = 5 * 1024 * 1024;

/// Download a remote file and cache it locally. Returns cached path.
pub fn fetch_to_cache(url: &str, refresh: bool) -> Result<PathBuf> {
    validate_url(url)?;

    let cache_dir = cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let filename = sanitize_filename(url);
    let cache_path = cache_dir.join(filename);

    if cache_path.exists() && !refresh {
        return Ok(cache_path);
    }

    let response = ureq::get(url)
        .call()
        .with_context(|| format!("Failed to fetch URL: {url}"))?;

    let mut reader = response.into_reader().take(MAX_REMOTE_BYTES + 1);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    if content.len() as u64 > MAX_REMOTE_BYTES {
        anyhow::bail!("Remote workflow exceeds {} bytes: {url}", MAX_REMOTE_BYTES);
    }

    let mut file = fs::File::create(&cache_path)?;
    file.write_all(content.as_bytes())?;

    Ok(cache_path)
}

fn cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir().context("No cache directory available")?;
    Ok(base.join("workflow-map").join("remote"))
}

fn sanitize_filename(url: &str) -> String {
    url.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn is_remote_path(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

fn validate_url(url: &str) -> Result<()> {
    if !is_remote_path(url) {
        anyhow::bail!("Remote workflow URL must use http:// or https://: {url}");
    }
    if url.trim() != url || url.contains(char::is_whitespace) {
        anyhow::bail!("Remote workflow URL contains whitespace: {url}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let name = sanitize_filename("https://github.com/user/repo/file.yaml");
        assert!(name.contains("github_com"));
        assert!(name.ends_with("file_yaml"));
    }

    #[test]
    fn test_is_remote() {
        assert!(is_remote_path("https://example.com/file.yaml"));
        assert!(!is_remote_path("./local.yaml"));
    }

    #[test]
    fn test_validate_url_rejects_invalid_schemes() {
        assert!(validate_url("file:///tmp/workflow.yaml").is_err());
        assert!(validate_url("https://example.com/workflow.yaml").is_ok());
    }

    #[test]
    fn test_validate_url_rejects_whitespace() {
        assert!(validate_url("https://example.com/work flow.yaml").is_err());
        assert!(validate_url(" https://example.com/workflow.yaml").is_err());
    }
}
