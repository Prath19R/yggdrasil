use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const NVIDIA_URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub tag: &'static str,
}

pub struct InferenceService;

#[derive(Serialize)]
struct NvidiaPayload<'a> {
    model: &'a str,
    messages: &'a [ChatMessagePayload],
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatKwargs>,
}

#[derive(Serialize)]
struct ChatKwargs {
    enable_thinking: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessagePayload {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponseJson {
    choices: Option<Vec<ChoiceJson>>,
}

#[derive(Deserialize)]
struct ChoiceJson {
    message: Option<MessageJson>,
}

#[derive(Deserialize)]
struct MessageJson {
    content: Option<String>,
    reasoning_content: Option<String>,
}

impl InferenceService {
    pub fn available_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "nvidia/nemotron-3.5-lightning-30b-a3b",
                name: "Nemotron 3.5 Lightning (Reasoning)",
                tag: "Reasoning",
            },
            ModelInfo {
                id: "mistralai/mistral-large-4-1t-a52b",
                name: "Mistral Large Multimodal (Vision & Architect)",
                tag: "Multimodal",
            },
            ModelInfo {
                id: "qwen/qwen2.5-coder-32b-instruct",
                name: "Qwen 2.5 Coder 32B (High-Speed Code)",
                tag: "Fast Code",
            },
            ModelInfo {
                id: "deepseek-ai/deepseek-r1",
                name: "DeepSeek R1 (Deep Algorithmic Logic)",
                tag: "Deep Logic",
            },
        ]
    }

    pub async fn query_llm(
        mode: &str,
        model: &str,
        api_key: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<(String, Option<String>), String> {
        let messages = vec![
            ChatMessagePayload {
                role: "system".into(),
                content: system_prompt.into(),
            },
            ChatMessagePayload {
                role: "user".into(),
                content: user_prompt.into(),
            },
        ];

        if mode == "local" {
            // Local Ollama proxy (OpenAI-compatible)
            let url = "http://localhost:11434/v1/chat/completions";
            let payload = serde_json::json!({
                "model": "qwen2.5-coder:7b",
                "messages": messages,
                "temperature": 0.3,
                "max_tokens": 4096,
            });

            let resp = Request::post(url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .map_err(|e| format!("Failed to serialize request: {}", e))?
                .send()
                .await
                .map_err(|e| format!("Failed to reach local Ollama daemon at {}: {}", url, e))?;

            if !resp.ok() {
                return Err(format!("Ollama HTTP Error: {}", resp.status()));
            }

            let json: ChatResponseJson = resp
                .json()
                .await
                .map_err(|e| format!("Invalid JSON from Ollama: {}", e))?;

            let content = json
                .choices
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.message)
                .and_then(|m| m.content)
                .unwrap_or_else(|| "No output received from Ollama.".to_string());

            return Ok((content, None));
        }

        // Cloud API (NVIDIA NIM)
        if api_key.trim().is_empty() {
            return Err("Missing NVIDIA NIM API Key. Please click '🔑 API Key' in the top bar to set your free key.".into());
        }

        let payload = NvidiaPayload {
            model,
            messages: &messages,
            temperature: 0.3,
            max_tokens: 4096,
            reasoning_effort: Some("high"),
            chat_template_kwargs: Some(ChatKwargs { enable_thinking: true }),
        };

        let resp = Request::post(NVIDIA_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_key.trim()))
            .json(&payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?
            .send()
            .await
            .map_err(|e| format!("NVIDIA NIM network error: {}", e))?;

        if !resp.ok() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("NVIDIA NIM Error ({status}): {err_text}"));
        }

        let json: ChatResponseJson = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let choice = json.choices.and_then(|mut c| c.pop()).and_then(|c| c.message);
        let content = choice.as_ref().and_then(|m| m.content.clone());
        let reasoning = choice.as_ref().and_then(|m| m.reasoning_content.clone());

        let final_text = match content {
            Some(c) if !c.trim().is_empty() => c,
            _ => reasoning.clone().unwrap_or_else(|| "Completed with no output.".to_string()),
        };

        Ok((final_text, reasoning))
    }
}
