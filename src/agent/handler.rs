use std::path::PathBuf;
use crate::tools::WorkingMemory;

/// Manages the agent's state across turns
#[derive(Debug)]
pub struct AgentHandler {
    pub working: WorkingMemory,
    pub cwd: PathBuf,
    pub current_turn: u32,
    pub history_info: Vec<String>,
    pub max_turns: u32,
    pub verbose: bool,
}

impl AgentHandler {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            working: WorkingMemory {
                key_info: None,
                related_sop: None,
                in_plan_mode: None,
                passed_sessions: 0,
            },
            cwd,
            current_turn: 0,
            history_info: Vec::new(),
            max_turns: 40,
            verbose: true,
        }
    }

    pub fn update_working_memory(&mut self, key_info: Option<String>, related_sop: Option<String>) {
        if let Some(info) = key_info {
            self.working.key_info = Some(info);
        }
        if let Some(sop) = related_sop {
            self.working.related_sop = Some(sop);
        }
    }

    pub fn add_history(&mut self, info: String) {
        self.history_info.push(info);
        // Keep only last 30 entries
        if self.history_info.len() > 30 {
            self.history_info.remove(0);
        }
    }

    pub fn set_plan_mode(&mut self, plan_file: Option<String>) {
        self.working.in_plan_mode = plan_file;
    }
}
