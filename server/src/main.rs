use axum::routing::{get, post, patch};
use axum::{Router, Json};
use axum::extract::DefaultBodyLimit;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use tower_http::limit::RequestBodyLimitLayer;
use axum::http::Method;
use axum::routing::get_service;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use chrono::Utc;

mod handlers;

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "w9"
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    
    let base_url = std::env::var("BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("https://w9.se"));
    
    tracing::info!("Base URL: {}", base_url);

    // Initialize Postgres Connection Pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to Postgres: {}", e))?;
    
    tracing::info!("Database connected successfully.");

    // Run Migrations (simplified for now, ideally use sqlx migrate!)
    // For this refactor, we assume the w9-db project or manual setup handles the schema, 
    // or we add basic table creation logic here similar to the sqlite version but for postgres.
    // Given the "update codes" instruction, I'll add the schema ensure logic.
    ensure_schema(&pool).await?;

    let base_url = std::env::var("BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("https://w9.se"));

    let fallback_sender_email = std::env::var("EMAIL_FROM_ADDRESS")
        .unwrap_or_else(|_| "W9 Tools <no-reply@w9.se>".to_string());

    let mut sender_config = match handlers::load_email_sender(&pool).await {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to load stored sender config: {}", e);
            None
        }
    };

    if sender_config.is_none() {
        let fallback = handlers::EmailSenderConfig {
            sender_type: None,
            sender_id: None,
            email: fallback_sender_email.clone(),
            display_label: Some("W9 Tools".to_string()),
            via_display: None,
        };
        if let Err(e) = handlers::save_email_sender(&pool, &fallback).await {
            tracing::warn!("Failed to persist default sender config: {}", e);
        }
        sender_config = Some(fallback);
    }

    let email_sender = Arc::new(RwLock::new(sender_config));

    let uploads_dir = std::env::var("UPLOADS_DIR")
        .unwrap_or_else(|_| "uploads".to_string());
    if let Err(e) = std::fs::create_dir_all(&uploads_dir) {
        tracing::error!("Failed to create uploads directory {}: {}", uploads_dir, e);
        return Err(anyhow::anyhow!("Failed to create uploads directory: {}", e));
    }

    let w9_mail_api_url = std::env::var("W9_MAIL_API_URL")
        .unwrap_or_else(|_| "https://w9.nu".to_string());
    
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-me-in-production".to_string());

    let password_reset_base_url = std::env::var("PASSWORD_RESET_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("{}/reset-password", base_url.trim_end_matches('/')));
    let verification_base_url = std::env::var("VERIFICATION_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("{}/verify-email", base_url.trim_end_matches('/')));
    let w9_mail_api_token = std::env::var("W9_MAIL_API_TOKEN").ok().filter(|v| !v.trim().is_empty());
    let turnstile_secret = std::env::var("TURNSTILE_SECRET_KEY").ok().filter(|v| !v.trim().is_empty());
    
    let default_logo_path = "/var/www/w9/android-chrome-512x512.png";
    let qr_logo_path = std::env::var("QR_LOGO_PATH")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            if std::path::Path::new(default_logo_path).exists() {
                Some(default_logo_path.to_string())
            } else {
                None
            }
        });
    
    let app_state = handlers::AppState { 
        pool: pool.clone(),
        base_url: base_url.clone(),
        uploads_dir: uploads_dir.clone(),
        w9_mail_api_url: w9_mail_api_url.clone(),
        jwt_secret: jwt_secret.clone(),
        password_reset_base_url,
        verification_base_url,
        w9_mail_api_token,
        email_sender,
        turnstile_secret,
        qr_logo_path,
    };

    let files_service = get_service(ServeDir::new(&uploads_dir))
        .handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") });

    let app = Router::new()
        .nest_service("/files", files_service)
        .route("/health", get(health_check))
        .route("/api/upload", axum::routing::options(handlers::cors_preflight))
        .route("/api/upload", post(handlers::api_upload))
        .route("/api/notepad", post(handlers::api_notepad))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/password-reset", post(handlers::request_password_reset))
        .route("/api/auth/confirm-password-reset", post(handlers::confirm_password_reset))
        .route("/api/auth/verify-email", post(handlers::verify_email_token))
        .route("/api/auth/change-password", post(handlers::change_password))
        .route("/api/user/items", get(handlers::user_items))
        .route("/api/user/items/:code/:kind", post(handlers::user_delete_item))
        .route("/api/user/items/:code/:kind/update", post(handlers::user_update_item))
        .route("/api/admin/users", get(handlers::admin_list_users).post(handlers::admin_create_user))
        .route("/api/admin/users/:id", patch(handlers::admin_update_user).delete(handlers::admin_delete_user))
        .route("/api/admin/users/send-reset", post(handlers::admin_send_password_reset))
        .route("/api/admin/email/senders", get(handlers::admin_list_email_senders))
        .route("/api/admin/email/sender", get(handlers::admin_get_email_sender).put(handlers::admin_set_email_sender))
        .route("/r/:code", get(handlers::result_handler))
        .route("/s/:code", get(handlers::short_handler))
        .route("/n/:code", get(handlers::notepad_handler))
        .route("/api/admin/items", get(handlers::admin_items))
        .route("/api/admin/items/:code/:kind", post(handlers::admin_delete_item_with_kind))
        .route("/api/admin/items/:code", post(handlers::admin_delete_item))
        .with_state(app_state)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any)
                .expose_headers(Any)
        );

    let addr: SocketAddr = format!("{}:{}", host, port).parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
    tracing::info!("🚀 Server listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await
        .map_err(|e| anyhow::anyhow!("Failed to bind: {}", e))?;
    
    tracing::info!("✓ Server started successfully");
    axum::serve(listener, app).await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

async fn ensure_schema(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    // Basic schema migration for Postgres
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            code TEXT NOT NULL,
            kind TEXT NOT NULL,
            value TEXT NOT NULL,
            created_at BIGINT NOT NULL,
            user_id TEXT,
            PRIMARY KEY (code, kind)
        );
        CREATE INDEX IF NOT EXISTS idx_items_code ON items(code);
        CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);
        
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            must_change_password BOOLEAN NOT NULL DEFAULT FALSE,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            is_verified BOOLEAN NOT NULL DEFAULT TRUE
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

        CREATE TABLE IF NOT EXISTS password_reset_tokens (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at BIGINT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            consumed BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user ON password_reset_tokens(user_id);

        CREATE TABLE IF NOT EXISTS email_verification_tokens (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at BIGINT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            consumed BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_user ON email_verification_tokens(user_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#
    ).execute(pool).await?;

    ensure_default_admin(pool).await?;

    Ok(())
}

async fn ensure_default_admin(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let admin_email = std::env::var("DEFAULT_ADMIN_EMAIL")
        .unwrap_or_else(|_| "admin@w9.se".to_string());
    let admin_password = std::env::var("DEFAULT_ADMIN_PASSWORD")
        .unwrap_or_else(|_| "Admin@123".to_string());

    let admin_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM users WHERE role = 'admin'")
        .fetch_one(pool)
        .await
        .unwrap_or(0) > 0;

    if !admin_exists {
        let salt = handlers::generate_token(32);
        let password_hash = handlers::hash_with_salt(&admin_password, &salt);
        let created_at = Utc::now().timestamp();
        
        sqlx::query(
            "INSERT INTO users(id, email, password_hash, salt, role, must_change_password, created_at, is_verified) \
             VALUES ($1, $2, $3, $4, 'admin', TRUE, $5, TRUE)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&admin_email)
        .bind(password_hash)
        .bind(salt)
        .bind(created_at)
        .execute(pool)
        .await?;

        tracing::info!(
            "Created default admin user {} (set DEFAULT_ADMIN_EMAIL/PASSWORD to override)",
            admin_email
        );
    }

    Ok(())
}
