use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SessionType {
    Ws,
    Http,
    ExtWs,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub info: HashMap<String, serde_json::Value>,
    pub connect_at: f64,
    pub disconnect_at: Option<f64>,
    pub session_type: SessionType,
}

impl Session {
    pub fn new(id: String, info: HashMap<String, serde_json::Value>, session_type: SessionType) -> Self {
        Self {
            id,
            info,
            connect_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            disconnect_at: None,
            session_type,
        }
    }

    pub fn url(&self) -> String {
        self.info.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn is_active(&self) -> bool {
        self.disconnect_at.is_none()
    }

    pub fn mark_disconnected(&mut self) {
        self.disconnect_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64());
    }
}
