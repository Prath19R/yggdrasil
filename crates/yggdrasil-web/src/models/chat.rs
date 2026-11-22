use crate::models::diff::FileDiff;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub is_user: bool,
    pub text: String,
    pub thinking: Option<String>,
    pub diff: Option<FileDiff>,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn user(text: &str) -> Self {
        Self {
            id: js_sys::Date::now().to_string(),
            sender: "👤 You".into(),
            is_user: true,
            text: text.to_string(),
            thinking: None,
            diff: None,
            timestamp: Self::now_time(),
        }
    }

    pub fn assistant(sender: &str, text: &str) -> Self {
        Self {
            id: js_sys::Date::now().to_string(),
            sender: sender.to_string(),
            is_user: false,
            text: text.to_string(),
            thinking: None,
            diff: None,
            timestamp: Self::now_time(),
        }
    }

    pub fn system(sender: &str, text: &str) -> Self {
        Self {
            id: js_sys::Date::now().to_string(),
            sender: sender.to_string(),
            is_user: false,
            text: text.to_string(),
            thinking: None,
            diff: None,
            timestamp: Self::now_time(),
        }
    }

    fn now_time() -> String {
        let date = js_sys::Date::new_0();
        let hours = date.get_hours();
        let mins = date.get_minutes();
        let secs = date.get_seconds();
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }
}
