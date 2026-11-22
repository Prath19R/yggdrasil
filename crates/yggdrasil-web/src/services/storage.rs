use web_sys::window;

pub struct StorageService;

impl StorageService {
    pub fn has_api_key() -> bool {
        Self::get_api_key().map(|k| !k.trim().is_empty()).unwrap_or(false)
    }

    pub fn get_api_key() -> Option<String> {
        let storage = window()?.local_storage().ok()??;
        storage.get_item("yggdrasil_nvidia_api_key").ok()?
    }

    pub fn set_api_key(key: &str) {
        if let Some(w) = window() {
            if let Ok(Some(storage)) = w.local_storage() {
                let _ = storage.set_item("yggdrasil_nvidia_api_key", key);
            }
        }
    }

    pub fn get_mode() -> String {
        window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("yggdrasil_mode").ok())
            .flatten()
            .unwrap_or_else(|| "cloud".to_string())
    }

    pub fn set_mode(mode: &str) {
        if let Some(w) = window() {
            if let Ok(Some(storage)) = w.local_storage() {
                let _ = storage.set_item("yggdrasil_mode", mode);
            }
        }
    }
}
