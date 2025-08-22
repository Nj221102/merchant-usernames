use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use crate::AppState;
use crate::error::AppError;
use crate::models::UserRepository;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(AppError::Authentication("Missing authorization header".to_string())),
    };

    let user_id = state.jwt_service.get_user_id_from_token(token)?;
    
    // Verify user exists
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo.find_by_id(user_id).await?;
    
    if user.is_none() {
        return Err(AppError::Authentication("User not found".to_string()));
    }

    // Add user ID to request extensions for use in handlers
    request.extensions_mut().insert(user_id);
    
    Ok(next.run(request).await)
}
