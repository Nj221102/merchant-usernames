use axum::{
    extract::{State, Extension},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};
use crate::{AppState, error::Result};
use crate::models::UserRepository;
use crate::services::{CryptoService, GreenlightService};

#[derive(Deserialize)]
pub struct NodeRegisterRequest {
    #[serde(rename = "encryptedSeed")]
    pub encrypted_seed: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct NodeRecoverRequest {
    #[serde(rename = "encryptedSeed")]
    pub encrypted_seed: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct NodeCredentialsResponse {
    #[serde(rename = "encryptedDeviceCreds")]
    pub encrypted_device_creds: String,
}

#[derive(Deserialize)]
pub struct CreateOfferRequest {
    pub amount_msat: Option<u64>,
    pub description: String,
}

/// POST /api/v1/node/register
pub async fn register_node(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(request): Json<NodeRegisterRequest>,
) -> Result<Json<NodeCredentialsResponse>> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let greenlight_service = GreenlightService::new(state.config.clone());

    // Get user from database
    let user = user_repo.find_by_id(user_id).await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

    // Check if user already has device credentials
    if user.encrypted_device_creds.is_some() {
        return Err(crate::error::AppError::BadRequest(
            "Node already registered for this user".to_string()
        ));
    }

    // Decrypt the seed
    let mnemonic = CryptoService::decrypt(&request.encrypted_seed, &request.password)?;
    
    // Validate the mnemonic
    if !CryptoService::validate_mnemonic(&mnemonic)? {
        return Err(crate::error::AppError::Validation("Invalid mnemonic".to_string()));
    }

    // Convert mnemonic to seed
    let seed = CryptoService::mnemonic_to_seed(&mnemonic)?;

    // Register node with Greenlight
    let device_creds = greenlight_service.register_node(&seed).await?;

    // Store device credentials as base64 for simplicity (in production, encrypt properly)
    let creds_base64 = general_purpose::STANDARD.encode(&device_creds.creds);

    // Store credentials in database
    user_repo.update_device_credentials(user_id, &creds_base64).await?;

    Ok(Json(NodeCredentialsResponse {
        encrypted_device_creds: creds_base64,
    }))
}

/// POST /api/v1/node/recover
pub async fn recover_node(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(request): Json<NodeRecoverRequest>,
) -> Result<Json<NodeCredentialsResponse>> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let greenlight_service = GreenlightService::new(state.config.clone());

    // Decrypt the seed
    let mnemonic = CryptoService::decrypt(&request.encrypted_seed, &request.password)?;
    
    // Validate the mnemonic
    if !CryptoService::validate_mnemonic(&mnemonic)? {
        return Err(crate::error::AppError::Validation("Invalid mnemonic".to_string()));
    }

    // Convert mnemonic to seed
    let seed = CryptoService::mnemonic_to_seed(&mnemonic)?;

    // Recover node with Greenlight
    let device_creds = greenlight_service.recover_node(&seed).await?;

    // Encrypt device credentials with user's password
    let creds_json = serde_json::to_string(&device_creds)
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to serialize credentials: {}", e)))?;
    
    let encrypted_device_creds = CryptoService::encrypt(&creds_json, &request.password)?;

    // Update credentials in database
    user_repo.update_device_credentials(user_id, &encrypted_device_creds).await?;

    Ok(Json(NodeCredentialsResponse {
        encrypted_device_creds,
    }))
}

/// GET /node/info - Get real node information
pub async fn get_node_info(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

    let encrypted_device_creds = user.encrypted_device_creds
        .ok_or_else(|| crate::error::AppError::BadRequest("No node registered for this user".to_string()))?;

    // For now, we'll assume the device creds are stored in a simple format
    // In production, you'd decrypt these with the user's password
    let device_creds = general_purpose::STANDARD.decode(&encrypted_device_creds)
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to decode device credentials: {}", e)))?;

    let greenlight_service = GreenlightService::new(state.config.clone());
    let node_info = greenlight_service.get_node_info(&device_creds).await?;

    Ok(Json(node_info))
}

/// GET /node/balance - Get real node balance
pub async fn get_balance(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

    let encrypted_device_creds = user.encrypted_device_creds
        .ok_or_else(|| crate::error::AppError::BadRequest("No node registered for this user".to_string()))?;

    let device_creds = general_purpose::STANDARD.decode(&encrypted_device_creds)
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to decode device credentials: {}", e)))?;

    let greenlight_service = GreenlightService::new(state.config.clone());
    let balance = greenlight_service.get_balance(&device_creds).await?;

    Ok(Json(balance))
}

/// POST /node/offer - Create a real Bolt12 offer
pub async fn create_offer(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

    let encrypted_device_creds = user.encrypted_device_creds
        .ok_or_else(|| crate::error::AppError::BadRequest("No node registered for this user".to_string()))?;

    let device_creds = general_purpose::STANDARD.decode(&encrypted_device_creds)
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to decode device credentials: {}", e)))?;

    let amount_msat = request.get("amount_msat").and_then(|v| v.as_u64());
    let description = request.get("description").and_then(|v| v.as_str()).unwrap_or("Bolt12 offer").to_string();

    let offer_request = CreateOfferRequest {
        amount_msat,
        description,
    };

    let greenlight_service = GreenlightService::new(state.config.clone());
    let offer = greenlight_service.create_offer(&device_creds, offer_request).await?;

    Ok(Json(offer))
}
