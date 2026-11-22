use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = isTauriApp)]
    fn is_tauri_app() -> bool;

    #[wasm_bindgen(js_name = yggdrasilInvoke, catch)]
    async fn js_yggdrasil_invoke(cmd: &str, args_json: &str) -> Result<JsValue, JsValue>;
}

pub struct TauriService;

impl TauriService {
    pub fn is_available() -> bool {
        is_tauri_app()
    }

    pub async fn invoke<T: serde::de::DeserializeOwned>(cmd: &str, payload: &serde_json::Value) -> Result<T, String> {
        let args_json = payload.to_string();
        match js_yggdrasil_invoke(cmd, &args_json).await {
            Ok(js_val) => {
                let s = js_val.as_string().unwrap_or_default();
                serde_json::from_str::<T>(&s).map_err(|e| format!("JSON decode error for {}: {}", cmd, e))
            }
            Err(err) => {
                let js_err: js_sys::Error = js_sys::Error::from(err);
                let err_str: String = js_err.message().into();
                Err(err_str)
            }
        }
    }
}
