use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::{AppState, error::AppError};
use crate::services::GreenlightService;
use crate::handlers::node::CreateOfferRequest;

#[derive(Deserialize)]
pub struct WebSocketAuth {
    pub token: String,
}

#[derive(Deserialize)]
pub struct WebSocketMessage {
    pub command: String,
    pub payload: Option<Value>,
}

#[derive(Serialize)]
pub struct WebSocketResponse {
    pub command: String,
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthMessage {
    #[serde(rename = "encryptedDeviceCreds")]
    pub encrypted_device_creds: String,
    pub password: String,
}

/// GET /api/v1/ws/connect
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(auth): Query<WebSocketAuth>,
) -> Result<Response, AppError> {
    // Validate the token first
    let user_id = state.jwt_service.get_user_id_from_token(&auth.token)?;
    
    Ok(ws.on_upgrade(move |socket| websocket_connection(socket, state, user_id)))
}

async fn websocket_connection(socket: WebSocket, state: AppState, _user_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let greenlight_service = GreenlightService::new(state.config.clone());
    
    // Wait for authentication message
    let mut authenticated = false;
    let mut encrypted_creds: Option<String> = None;
    let mut password: Option<String> = None;

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if !authenticated {
                    // First message should be authentication
                    match serde_json::from_str::<AuthMessage>(&text) {
                        Ok(auth_msg) => {
                            encrypted_creds = Some(auth_msg.encrypted_device_creds);
                            password = Some(auth_msg.password);
                            authenticated = true;

                            let response = WebSocketResponse {
                                command: "auth".to_string(),
                                success: true,
                                data: Some(serde_json::json!({"message": "Authenticated successfully"})),
                                error: None,
                            };

                            if let Ok(response_text) = serde_json::to_string(&response) {
                                if sender.send(Message::Text(response_text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            let response = WebSocketResponse {
                                command: "auth".to_string(),
                                success: false,
                                data: None,
                                error: Some("Invalid authentication message format".to_string()),
                            };

                            if let Ok(response_text) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(response_text)).await;
                            }
                            break;
                        }
                    }
                } else {
                    // Handle commands after authentication
                    match serde_json::from_str::<WebSocketMessage>(&text) {
                        Ok(ws_msg) => {
                            let response = handle_websocket_command(
                                &greenlight_service,
                                &ws_msg,
                                encrypted_creds.as_ref().unwrap(),
                                password.as_ref().unwrap(),
                            ).await;

                            if let Ok(response_text) = serde_json::to_string(&response) {
                                if sender.send(Message::Text(response_text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            let response = WebSocketResponse {
                                command: "error".to_string(),
                                success: false,
                                data: None,
                                error: Some("Invalid message format".to_string()),
                            };

                            if let Ok(response_text) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(response_text)).await;
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            _ => {}
        }
    }
}

async fn handle_websocket_command(
    greenlight_service: &GreenlightService,
    message: &WebSocketMessage,
    _encrypted_creds: &str,
    _password: &str,
) -> WebSocketResponse {
    let result = match message.command.as_str() {
        "get_info" => {
            greenlight_service.get_node_info(&[0u8; 32]).await
        }
        "create_offer" => {
            match message.payload.as_ref().and_then(|p| serde_json::from_value::<CreateOfferRequest>(p.clone()).ok()) {
                Some(request) => {
                    greenlight_service.create_offer(&[0u8; 32], request).await
                }
                None => Err(AppError::BadRequest("Invalid create_offer payload".to_string()))
            }
        }
        "list_offers" => {
            // This would need to be implemented in the greenlight service
            // For now, return an error
            Err(AppError::BadRequest("list_offers not yet implemented".to_string()))
        }
        _ => Err(AppError::BadRequest(format!("Unknown command: {}", message.command)))
    };

    match result {
        Ok(data) => WebSocketResponse {
            command: message.command.clone(),
            success: true,
            data: Some(data),
            error: None,
        },
        Err(error) => WebSocketResponse {
            command: message.command.clone(),
            success: false,
            data: None,
            error: Some(error.to_string()),
        }
    }
}
