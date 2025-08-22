use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use crate::{AppState, error::Result};
use crate::models::{CreateUserRequest, LoginRequest, UserRepository};
use crate::services::CryptoService;

#[derive(Serialize)]
pub struct SignupResponse {
    #[serde(rename = "encryptedSeed")]
    pub encrypted_seed: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// POST /api/v1/users/signup
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<SignupResponse>)> {
    let user_repo = UserRepository::new(state.db_pool.clone());

    // Check if user already exists
    if user_repo.public_key_exists(&request.public_key).await? {
        return Err(crate::error::AppError::BadRequest(
            "User with this public key already exists".to_string()
        ));
    }

    // Validate inputs
    if request.public_key.is_empty() || request.password.len() < 8 {
        return Err(crate::error::AppError::Validation(
            "Public key cannot be empty and password must be at least 8 characters".to_string()
        ));
    }

    // Generate BIP39 mnemonic
    let mnemonic = CryptoService::generate_mnemonic()?;
    
    // Encrypt the mnemonic with the user's password
    let encrypted_seed = CryptoService::encrypt(&mnemonic, &request.password)?;
    
    // Hash the password
    let password_hash = CryptoService::hash_password(&request.password)?;
    
    // Create user in database
    let user = user_repo.create_user(
        &request.public_key,
        &password_hash,
        &encrypted_seed,
    ).await?;
    
    // Generate JWT token
    let token = state.jwt_service.generate_token(user.id)?;
    
    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            encrypted_seed,
            token,
        })
    ))
}

/// POST /api/v1/users/login
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let user_repo = UserRepository::new(state.db_pool.clone());

    // Find user by public key
    let user = user_repo.find_by_public_key(&request.public_key).await?
        .ok_or_else(|| crate::error::AppError::Authentication("Invalid credentials".to_string()))?;

    // Verify password
    if !CryptoService::verify_password(&request.password, &user.password_hash)? {
        return Err(crate::error::AppError::Authentication("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let token = state.jwt_service.generate_token(user.id)?;

    Ok(Json(LoginResponse { token }))
}
