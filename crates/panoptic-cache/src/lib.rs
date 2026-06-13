use dashmap::DashMap;

pub struct AssetCache {
    url_to_path: DashMap<String, String>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            url_to_path: DashMap::new(),
        }
    }

    pub fn get_or_fetch(&self, url: &str) -> String {
        if let Some(path) = self.url_to_path.get(url) {
            return path.clone();
        }
        let saved_path = format!("/tmp/cached_{}.png", uuid::Uuid::new_v4());
        self.url_to_path.insert(url.to_string(), saved_path.clone());
        saved_path
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_cache_basic() {
        let cache = AssetCache::new();
        let url1 = "https://example.com/image1.png";
        let url2 = "https://example.com/image2.png";

        let path1 = cache.get_or_fetch(url1);
        let path1_again = cache.get_or_fetch(url1);
        let path2 = cache.get_or_fetch(url2);

        // Path should have the correct format
        assert!(path1.starts_with("/tmp/cached_"));
        assert!(path1.ends_with(".png"));

        // Idempotency: same URL returns the same path
        assert_eq!(path1, path1_again);

        // Different URLs return different paths
        assert_ne!(path1, path2);
        assert!(path2.starts_with("/tmp/cached_"));
    }
}
