use std::env;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub gl_cert_path: String,
    pub gl_key_path: String,
    pub gl_network: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/greenlight_wallet".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default (not secure for production)");
                "default-secret-change-in-production".to_string()
            });

        let server_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        let gl_cert_path = env::var("GL_CERT_PATH")
            .unwrap_or_else(|_| "./client.crt".to_string());
        let gl_key_path = env::var("GL_KEY_PATH")
            .unwrap_or_else(|_| "./client-key.pem".to_string());
        let gl_network = env::var("GL_NETWORK")
            .unwrap_or_else(|_| "bitcoin".to_string());

        Ok(Config {
            database_url,
            jwt_secret,
            server_host,
            server_port,
            gl_cert_path,
            gl_key_path,
            gl_network,
        })
    }
}
