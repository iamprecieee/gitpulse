use gitpulse::{
    models::a2a::{A2ARequest, A2AResponse, Message, MessagePart},
    utils::helpers::create_artifacts,
};

#[test]
fn test_default_configuration() {
    let json = r#"{
        "jsonrpc": "2.0",
        "id": "test-456",
        "method": "message/send",
        "params": {
            "message": {
                "kind": "message",
                "role": "user",
                "parts": [
                    {"kind": "text", "text": "Hello!"}
                ],
                "messageId": "message-456"
            }
        }
    }"#;

    let request = serde_json::from_str::<A2ARequest>(json).unwrap();
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.params.message.role, "user");
    assert!(request.params.configuration.is_none());
}

#[test]
fn test_success_response() {
    let message = Message {
        kind: "text".to_string(),
        role: "user".to_string(),
        parts: vec![],
        message_id: "test-123".to_string(),
        task_id: Some("test-123".to_string()),
        telex_metadata: None
    };
    let response_text = "Here are trending repos...".to_string();

    let response = A2AResponse::success(
        "test-123".to_string(),
        Some("task-123".to_string()),
        response_text.clone(),
        create_artifacts(response_text),
        &message,
    );

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("test-123"));
    assert!(json.contains("completed"));
    assert!(json.contains("Here are trending repos"));
}

#[test]
fn test_success_response_without_task_id() {
    let message = Message {
        kind: "text".to_string(),
        role: "user".to_string(),
        parts: vec![],
        message_id: "test-123".to_string(),
        task_id: Some("test-123".to_string()),
        telex_metadata: None
    };
    let response_text = "Here are trending repos...".to_string();

    let response = A2AResponse::success(
        "req-111".to_string(),
        None,
        response_text.clone(),
        create_artifacts(response_text),
        &message,
    );

    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], "req-111");
    assert_eq!(parsed["result"]["status"]["state"], "completed");
    assert!(parsed["result"]["id"].as_str().unwrap().len() > 10);
}

#[test]
fn test_serialize_error_response() {
    let response = A2AResponse::error(
        "req-789".to_string(),
        -32601,
        "Method not found".to_string(),
    );

    let json = serde_json::to_string_pretty(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], "req-789");
    assert_eq!(parsed["error"]["code"], -32601);
    assert_eq!(parsed["error"]["message"], "Method not found");
    assert!(parsed.get("result").is_none());
}

#[test]
fn test_response_round_trip() {
    let message = Message {
        kind: "text".to_string(),
        role: "user".to_string(),
        parts: vec![],
        message_id: "test-123".to_string(),
        task_id: Some("test-123".to_string()),
        telex_metadata: None
    };
    let response_text = "Round-trip test".to_string();

    let original = A2AResponse::success(
        "req-roundtrip".to_string(),
        Some("task-roundtrip".to_string()),
        response_text.clone(),
        create_artifacts(response_text),
        &message,
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: A2AResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.jsonrpc, "2.0");
    assert!(deserialized.error.is_none());
    assert!(deserialized.result.is_some());
    let result = deserialized.result.unwrap();
    assert_eq!(result.status.state, "completed");

    match &result.status.message.parts[0] {
        MessagePart::Text { text, .. } => {
            assert_eq!(text, "Round-trip test");
        }
        MessagePart::Data { .. } => {
            panic!("Expected Text part, got Data part");
        }
    }
}
