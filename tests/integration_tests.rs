// Integration tests for ga-core
// Verifies the library compiles and basic types work correctly

use ga_core::llm::models::{Role, Message, ContentBlock};
use ga_core::agent::outcome::{StepOutcome, ToolCall};

#[test]
fn test_library_compiles() {
    // If this test compiles, the library is structurally sound
    let msg = Message {
        role: Role::User,
        content: vec![ContentBlock::Text { text: "hello".to_string() }],
        tool_results: None,
    };

    assert!(matches!(msg.role, Role::User));
}

#[test]
fn test_role_variants() {
    let system = Role::System;
    let user = Role::User;
    let assistant = Role::Assistant;

    assert!(matches!(system, Role::System));
    assert!(matches!(user, Role::User));
    assert!(matches!(assistant, Role::Assistant));
}

#[test]
fn test_content_block_text() {
    let block = ContentBlock::Text { text: "test content".to_string() };

    match block {
        ContentBlock::Text { text } => assert_eq!(text, "test content"),
        _ => panic!("Expected Text content block"),
    }
}

#[test]
fn test_message_construction() {
    let msg = Message {
        role: Role::Assistant,
        content: vec![
            ContentBlock::Text { text: "first".to_string() },
            ContentBlock::Text { text: "second".to_string() },
        ],
        tool_results: None,
    };

    assert!(matches!(msg.role, Role::Assistant));
    assert_eq!(msg.content.len(), 2);
}

#[test]
fn test_step_outcome_exit() {
    let exit = StepOutcome::exit(None);
    assert!(exit.should_exit);
    assert!(exit.next_prompt.is_none());
    assert!(exit.data.is_none());
}

#[test]
fn test_step_outcome_continue_with() {
    let cont = StepOutcome::continue_with("next".to_string(), None);
    assert!(!cont.should_exit);
    assert_eq!(cont.next_prompt, Some("next".to_string()));
    assert!(cont.data.is_none());
}

#[test]
fn test_step_outcome_done() {
    let done = StepOutcome::done(None);
    assert!(!done.should_exit);
    assert!(done.next_prompt.is_none());
    assert!(done.data.is_none());
}

#[test]
fn test_step_outcome_with_data() {
    let data = serde_json::json!({"key": "value"});
    let exit = StepOutcome::exit(Some(data.clone()));
    assert!(exit.should_exit);
    assert_eq!(exit.data, Some(data));
}

#[test]
fn test_tool_call_construction() {
    let call = ToolCall {
        tool_name: "file_read".to_string(),
        args: serde_json::json!({"path": "test.txt"}),
        id: Some("call_1".to_string()),
    };

    assert_eq!(call.tool_name, "file_read");
    assert_eq!(call.id, Some("call_1".to_string()));
}

#[test]
fn test_tool_call_without_id() {
    let call = ToolCall {
        tool_name: "code_run".to_string(),
        args: serde_json::json!({"script": "print(1)"}),
        id: None,
    };

    assert_eq!(call.tool_name, "code_run");
    assert!(call.id.is_none());
}
