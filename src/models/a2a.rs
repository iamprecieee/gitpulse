use axum::{
    Json,
    response::{IntoResponse, Response},
};
use chrono::{SecondsFormat, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct A2ARequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: RequestParams,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct A2AResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestParams {
    pub message: Message,
    #[serde(default)]
    pub configuration: Option<Configuration>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Message {
    pub kind: String,
    pub role: String,
    pub parts: Vec<MessagePart>,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "taskId")]
    pub task_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Configuration {
    #[serde(default)]
    pub blocking: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum MessagePart {
    Text { kind: String, text: String },
    Data { kind: String, data: Vec<Value> },
}

impl MessagePart {
    pub fn is_text(&self) -> bool {
        matches!(self, MessagePart::Text { .. })
    }

    pub fn is_data(&self) -> bool {
        matches!(self, MessagePart::Data { .. })
    }

    pub fn kind(&self) -> &str {
        match self {
            MessagePart::Text { kind, .. } => kind,
            MessagePart::Data { kind, .. } => kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskResult {
    pub kind: String,
    pub id: String,
    #[serde(rename = "contextId")]
    pub context_id: String,
    pub status: TaskStatus,
    pub artifacts: Vec<Artifact>,
    pub history: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskStatus {
    pub state: String,
    pub timestamp: String,
    pub message: Message,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Artifact {
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub name: String,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl A2AResponse {
    pub fn success(
        request_id: String,
        task_id: Option<String>,
        response_text: String,
        artifacts: Vec<Artifact>,
        request_message: &Message,
    ) -> Self {
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let task_id = match task_id {
            Some(val) => val,
            None => Uuid::new_v4().to_string(),
        };
        let response_message = Message {
            message_id: Uuid::new_v4().to_string(),
            role: "agent".to_string(),
            parts: vec![MessagePart::Text {
                kind: "text".to_string(),
                text: response_text,
            }],
            task_id: Some(task_id.clone()),
            kind: "message".to_string(),
        };

        Self {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(TaskResult {
                kind: "task".to_string(),
                id: task_id,
                context_id: Uuid::new_v4().to_string(),
                status: TaskStatus {
                    state: "completed".to_string(),
                    timestamp: now,
                    message: response_message.clone(),
                },
                artifacts,
                history: vec![request_message.clone(), response_message],
            }),
            error: None,
        }
    }

    pub fn error(request_id: String, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            error: Some(ErrorDetail {
                code,
                message,
                data: None,
            }),
            result: None,
        }
    }
}

impl IntoResponse for A2AResponse {
    fn into_response(self) -> Response {
        let status = if let Some(error) = &self.error {
            match error.code {
                -32600 => StatusCode::BAD_REQUEST,
                -32603 => StatusCode::INTERNAL_SERVER_ERROR,
                -32000 => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        } else {
            StatusCode::OK
        };

        (status, Json(self)).into_response()
    }
}
