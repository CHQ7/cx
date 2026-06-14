use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

use super::session::Session;

#[derive(Debug, Clone)]
pub struct JsResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub new_tabs: Vec<serde_json::Value>,
}

pub struct TMWebDriver {
    pub host: String,
    pub port: u16,
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub results: Arc<Mutex<HashMap<String, JsResult>>>,
    pub default_session_id: Arc<Mutex<Option<String>>>,
    pub latest_session_id: Arc<Mutex<Option<String>>>,
}

impl TMWebDriver {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            default_session_id: Arc::new(Mutex::new(None)),
            latest_session_id: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn execute_js(&self, code: &str, _timeout_secs: u64, session_id: Option<&str>) -> Result<JsResult, String> {
        // TODO: Send JS code to browser via WebSocket/HTTP
        // For now, return a placeholder
        let _ = code;
        let _ = session_id;

        // Simulate async operation
        sleep(Duration::from_millis(100)).await;

        Ok(JsResult {
            success: true,
            data: serde_json::json!({"result": "placeholder"}),
            new_tabs: vec![],
        })
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }

    pub fn add_session(&self, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session);
    }

    pub fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id);
    }
}
