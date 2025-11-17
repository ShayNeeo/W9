// The final, corrected handlers.rs file

use axum::extract::{Form, Multipart, Path, Query, State};
use axum_extra::typed_header::TypedHeader;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Json;
use axum::debug_handler;
use axum_extra::headers::Cookie;
use mime_guess::from_path as mime_from_path;
use nanoid::nanoid;
use qrcode::render::svg::Color;
use qrcode::QrCode;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::path::{Path as StdPath}; // Use StdPath to avoid conflict with axum::extract::Path
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use askama::Template;
use w9::templates::{ImageOgTemplate, FileInfoTemplate, PdfTemplate, VideoTemplate};
use image::{imageops::FilterType, DynamicImage, ImageOutputFormat, GenericImageView};
use sha2::{Digest, Sha256};
use rand::{distributions::Alphanumeric, Rng};
pub async fn cors_preflight() -> impl IntoResponse {
    // Let CorsLayer attach the appropriate headers; return 204 No Content
    (StatusCode::NO_CONTENT, ())
}

// Removed CodeParams; using Path<String> directly for routes with one :code param

// Utility to extract the admin cookie token
fn extract_admin_token(cookie: Option<TypedHeader<Cookie>>) -> Option<String> {
    cookie
        .as_ref()
        .and_then(|TypedHeader(c)| c.get("w9_admin"))
        .map(|value| value.to_string())
}

// Maximum file size: 1 GiB
const MAX_FILE_SIZE: usize = 1024 * 1024 * 1024;

// Allowed file extensions for uploads
const ALLOWED_EXTENSIONS: &[&str] = &[
    // images
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff", "avif", "svg",
    // documents
    "pdf", "txt", "md", "csv", "json", "rtf",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    // archives
    "zip", "tar", "gz", "rar", "7z",
    // audio
    "mp3", "wav", "flac", "ogg",
    // video
    "mp4", "mov", "webm", "avi", "mkv"
];

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff", "avif"];

// Preview target size: strictly under 1 MiB
const PREVIEW_MAX_BYTES: usize = 1 * 1024 * 1024;

// ensure_dir was unused; removed to avoid dead_code warning

fn make_preview_filename(original_filename: &str) -> String {
    let stem = StdPath::new(original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("preview");
    format!("{}.jpg", stem)
}

fn is_image_ext(ext: &str) -> bool {
    IMAGE_EXTENSIONS.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

fn encode_jpeg_under_limit(img: &DynamicImage, max_bytes: usize) -> Option<Vec<u8>> {
    // Try multiple qualities and downscales until under limit
    // For very large images (>20MB), be more aggressive with scaling and quality reduction
    let mut scales = vec![1.0, 0.9, 0.75, 0.6, 0.5, 0.4, 0.3, 0.25, 0.2, 0.15, 0.1];
    let qualities = vec![85u8, 75, 65, 55, 45, 35, 25, 15];
    
    let (w, h) = img.dimensions();
    
    for scale in scales.drain(..) {
        let target_w = (w as f32 * scale).max(1.0) as u32;
        let target_h = (h as f32 * scale).max(1.0) as u32;
        let resized = if target_w == w && target_h == h { 
            img.clone() 
        } else { 
            img.resize(target_w, target_h, FilterType::Lanczos3) 
        };
        
        for q in &qualities {
            let mut buf = Vec::with_capacity(512 * 1024);
            let mut cursor = std::io::Cursor::new(&mut buf);
            if resized.write_to(&mut cursor, ImageOutputFormat::Jpeg(*q)).is_ok() {
                if buf.len() <= max_bytes {
                    tracing::info!("Preview compressed: scale={}, quality={}, size={}KB", scale, q, buf.len() / 1024);
                    return Some(buf);
                }
            }
        }
    }
    
    tracing::warn!("Could not compress image under {}MB limit", max_bytes / 1024 / 1024);
    None
}

fn try_generate_preview(original_path: &StdPath, preview_path: &StdPath) -> Result<(), String> {
    let img = image::open(original_path).map_err(|e| format!("open image: {}", e))?;
    match encode_jpeg_under_limit(&img, PREVIEW_MAX_BYTES) {
        Some(bytes) => {
            std::fs::write(preview_path, &bytes).map_err(|e| format!("write preview: {}", e))
        }
        None => {
            // Fallback: For very large images, aggressively scale down first
            let (w, h) = img.dimensions();
            let pixel_count = (w as u64) * (h as u64);
            let max_pixels = 1920u64 * 1080u64;  // Max ~2MP (1920x1080)
            
            let fallback_img = if pixel_count > max_pixels {
                let scale = ((max_pixels as f32) / (pixel_count as f32)).sqrt();
                let new_w = (w as f32 * scale).max(1.0) as u32;
                let new_h = (h as f32 * scale).max(1.0) as u32;
                tracing::warn!("Extreme fallback: scaling {}x{} to {}x{}", w, h, new_w, new_h);
                img.resize(new_w, new_h, FilterType::Lanczos3)
            } else {
                img.clone()
            };
            
            // Try to write at very low quality
            let mut buf = Vec::new();
            for quality in [45u8, 35, 25, 15] {
                buf.clear();
                let mut cur = std::io::Cursor::new(&mut buf);
                if fallback_img.write_to(&mut cur, ImageOutputFormat::Jpeg(quality)).is_ok() {
                    if buf.len() <= PREVIEW_MAX_BYTES {
                        tracing::warn!("Fallback succeeded at quality={}, size={}KB", quality, buf.len() / 1024);
                        return std::fs::write(preview_path, &buf).map_err(|e| format!("write preview: {}", e));
                    }
                }
            }
            
            // Last resort: use absolute minimum
            buf.clear();
            let mut cur = std::io::Cursor::new(&mut buf);
            fallback_img.write_to(&mut cur, ImageOutputFormat::Jpeg(10)).map_err(|e| format!("encode jpeg: {}", e))?;
            tracing::warn!("Last resort fallback: quality=10, size={}KB", buf.len() / 1024);
            std::fs::write(preview_path, &buf).map_err(|e| format!("write preview: {}", e))
        }
    }
}

#[derive(Clone)]
pub struct AppState { 
    pub db_path: String, 
    pub base_url: String,
    pub uploads_dir: String,
}

fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.iter().any(|&allowed| allowed.eq_ignore_ascii_case(ext))
}

fn ensure_absolute(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("{}/{}", base.trim_end_matches('/'), url.trim_start_matches('/'))
    }
}

pub async fn result_handler(State(state): State<AppState>, Path(code): Path<String>, Query(q): Query<std::collections::HashMap<String,String>>) -> (StatusCode, String) {
    let conn = Connection::open(&state.db_path).unwrap();
    let mut stmt = conn.prepare("SELECT kind, value FROM items WHERE code = ?1").unwrap();
    let row = stmt.query_row(params![code.clone()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
    let (_kind, _value) = match row { Ok(v) => v, Err(_) => return (StatusCode::NOT_FOUND, r#"{"error":"Not found"}"#.to_string()) };
    let short_link = format!("{}/s/{}", state.base_url, code);
    let qr_svg = if q.get("qr").map(|v| v=="1").unwrap_or(false) {
        let qr_target = ensure_absolute(&state.base_url, &short_link);
        QrCode::new(qr_target.as_bytes())
            .map(|c| c
                .render::<Color>()
                .min_dimensions(320,320)
                .quiet_zone(true)
                .dark_color(Color("#000000"))
                .light_color(Color("#ffffff"))
                .build())
            .unwrap_or_default()
    } else { String::new() };
    let body = format!(
        r#"{{"code":"{}","short_link":"{}","qr_svg":"{}"}}"#,
        code,
        short_link,
        qr_svg.replace('"', "\\\"")
    );
    (StatusCode::OK, body)
}

pub async fn short_handler(State(state): State<AppState>, Path(code): Path<String>, headers: HeaderMap) -> axum::response::Response {
    let (kind, value) = {
        let conn = Connection::open(&state.db_path).unwrap();
        let mut stmt = conn.prepare("SELECT kind, value FROM items WHERE code = ?1").unwrap();
        let row = stmt.query_row(params![code.clone()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
        match row { Ok(v) => v, Err(_) => return (StatusCode::NOT_FOUND, "Not found").into_response() }
    };

    match kind.as_str() {
        "url" => Redirect::permanent(&value).into_response(),
        "file" => {
            let filename = value.strip_prefix("file:").unwrap_or(&value);
            if let Some(ext) = StdPath::new(filename).extension().and_then(|e| e.to_str()) {
                let mime = mime_from_path(filename).first_or_octet_stream();
                if is_image_ext(ext) || ext.eq_ignore_ascii_case("svg") {
                    let page_url = format!("{}/s/{}", state.base_url, code);
                    let image_url_full = format!("{}/files/{}", state.base_url, filename);
                    // For raster images: if original <= 1MB, use original; else generate a preview
                    let og_image_url = if !ext.eq_ignore_ascii_case("svg") {
                        let original_fs_path = StdPath::new(&state.uploads_dir).join(filename);
                        let original_is_small = std::fs::metadata(&original_fs_path)
                            .map(|m| m.len() as usize <= PREVIEW_MAX_BYTES)
                            .unwrap_or(false);
                        if original_is_small {
                            image_url_full.clone()
                        } else {
                            let preview_dir = StdPath::new(&state.uploads_dir).join("previews");
                            let preview_name = make_preview_filename(filename);
                            let preview_fs_path = preview_dir.join(&preview_name);
                            let preview_web_path = format!("previews/{}", preview_name);
                            if !preview_fs_path.exists() {
                                // Generate preview asynchronously in background
                                let orig_path = original_fs_path.clone();
                                let prev_path = preview_fs_path.clone();
                                let prev_dir = preview_dir.clone();
                                tokio::spawn(async move {
                                    let _ = tokio::fs::create_dir_all(&prev_dir).await;
                                    // Run CPU-intensive preview generation in blocking thread
                                    let _ = tokio::task::spawn_blocking(move || {
                                        try_generate_preview(&orig_path, &prev_path)
                                    }).await;
                                });
                            }
                            // Use preview if exists, otherwise fall back to original
                            if preview_fs_path.exists() {
                                format!("{}/files/{}", state.base_url, preview_web_path)
                            } else {
                                image_url_full.clone()
                            }
                        }
                    } else {
                        // For SVG use the original (usually tiny)
                        image_url_full.clone()
                    };
                    // Content negotiation: return HTML by default (for browsers)
                    // Only return raw image if explicitly requesting non-HTML (e.g., for embeds)
                    let accept = headers
                        .get(axum::http::header::ACCEPT)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("*/*")
                        .to_ascii_lowercase();
                    // Check if the primary/preferred content type is an image
                    let wants_raw_image = accept.starts_with("image/") || 
                                         accept.split(',')
                                               .next()
                                               .and_then(|first| first.split(';').next())
                                               .map(|mime| mime.trim().starts_with("image/"))
                                               .unwrap_or(false);
                    if wants_raw_image {
                        // For non-HTML (e.g., direct image fetch), stream the file instead of redirecting to avoid user-agent caching/transform issues
                        let fs_path = StdPath::new(&state.uploads_dir).join(filename);
                        if let Ok(bytes) = std::fs::read(&fs_path) {
                            let mut resp = axum::response::Response::new(bytes.into());
                            resp.headers_mut().insert(axum::http::header::CONTENT_TYPE, axum::http::HeaderValue::from_str(mime.as_ref()).unwrap_or(axum::http::HeaderValue::from_static("image/jpeg")));
                            return resp;
                        }
                        return Redirect::permanent(&image_url_full).into_response();
                    }
                    let tpl = ImageOgTemplate {
                        og_image_url: og_image_url,
                        full_image_url: image_url_full,
                        page_url,
                        title: "Shared Image".to_string(),
                        description: "Shared via w9.se".to_string(),
                    };
                    let html = Html(tpl.render().unwrap_or_else(|_| "Template error".to_string()));
                    let mut response = html.into_response();
                    // Add cache headers for better performance
                    response.headers_mut().insert(
                        axum::http::header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=3600")
                    );
                    return response;
                }
                let filename_display = StdPath::new(filename).file_name().and_then(|f| f.to_str()).unwrap_or(filename).to_string();
                let file_url = format!("{}/files/{}", state.base_url, filename);
                let page_url = format!("{}/s/{}", state.base_url, code);
                
                // PDF files: embedded viewer
                if ext.eq_ignore_ascii_case("pdf") {
                    let tpl = PdfTemplate { filename: filename_display, file_url, page_url };
                    let html = Html(tpl.render().unwrap_or_else(|_| "Template error".to_string()));
                    let mut response = html.into_response();
                    response.headers_mut().insert(
                        axum::http::header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=3600")
                    );
                    return response;
                }
                
                // Video files: HTML5 player
                if ext.eq_ignore_ascii_case("mp4") || ext.eq_ignore_ascii_case("webm") || 
                   ext.eq_ignore_ascii_case("mov") || ext.eq_ignore_ascii_case("avi") || 
                   ext.eq_ignore_ascii_case("mkv") {
                    let tpl = VideoTemplate { 
                        filename: filename_display, 
                        file_url, 
                        mime: mime.to_string(), 
                        page_url 
                    };
                    let html = Html(tpl.render().unwrap_or_else(|_| "Template error".to_string()));
                    let mut response = html.into_response();
                    response.headers_mut().insert(
                        axum::http::header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=3600")
                    );
                    return response;
                }
                
                // Other files: generic download page
                let tpl = FileInfoTemplate { filename: filename_display, file_url, mime: mime.to_string(), page_url };
                let html = Html(tpl.render().unwrap_or_else(|_| "Template error".to_string()));
                let mut response = html.into_response();
                response.headers_mut().insert(
                    axum::http::header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=3600")
                );
                return response;
            }
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

fn hash_with_salt(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn generate_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

async fn require_admin_token(db_path: &str, token_opt: Option<&str>) -> bool {
    if let Some(token) = token_opt {
        if let Ok(conn) = Connection::open(db_path) {
            if let Ok(mut stmt) = conn.prepare("SELECT 1 FROM sessions WHERE token = ?1") {
                let exists: Result<i32, _> = stmt.query_row(params![token], |r| r.get(0));
                return exists.is_ok();
            }
        }
    }
    false
}

#[derive(Deserialize)]
pub struct AdminLoginForm { pub username: String, pub password: String }

pub async fn admin_login_post(State(state): State<AppState>, Form(f): Form<AdminLoginForm>) -> impl IntoResponse {
    let conn = match Connection::open(&state.db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to open database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))).into_response();
        }
    };

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM admin", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        let salt = generate_token(16);
        let hash = hash_with_salt(&f.password, &salt);
        if let Err(e) = conn.execute("INSERT INTO admin (id, username, password_hash, salt) VALUES (1, ?1, ?2, ?3)", params![f.username, hash, salt]) {
            tracing::error!("Failed to create admin user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to create user"}))).into_response();
        }
        tracing::info!("Created first admin user: {}", f.username);
    }

    let row = conn
        .prepare("SELECT password_hash, salt FROM admin WHERE username = ?1")
        .and_then(|mut s| s.query_row(params![f.username], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))));
    
    let (hash, salt) = match row {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Login failed for user '{}': {}", f.username, e);
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid credentials"}))).into_response();
        }
    };

    if hash != hash_with_salt(&f.password, &salt) {
        tracing::warn!("Invalid password for user '{}'", f.username);
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid credentials"}))).into_response();
    }

    let token = generate_token(48);
    if let Err(e) = conn.execute("INSERT INTO sessions (token, created_at) VALUES (?1, strftime('%s','now'))", params![token.clone()]) {
        tracing::error!("Failed to create session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to create session"}))).into_response();
    }

    let mut headers = HeaderMap::new();
    let cookie = format!("w9_admin={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000", token);
    if let Err(e) = HeaderValue::from_str(&cookie) {
        tracing::error!("Failed to create cookie header: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Server error"}))).into_response();
    }
    headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    
    tracing::info!("Successful login for user '{}'", f.username);
    (StatusCode::OK, headers, Json(serde_json::json!({"success": true}))).into_response()
}

pub async fn admin_logout(State(state): State<AppState>, cookie: Option<TypedHeader<Cookie>>) -> impl IntoResponse {
    if let Some(tok) = extract_admin_token(cookie) {
        if let Ok(conn) = Connection::open(&state.db_path) {
            let _ = conn.execute("DELETE FROM sessions WHERE token = ?1", params![tok]);
        }
    }
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_static("w9_admin=; Max-Age=0; Path=/"));
    (StatusCode::OK, headers, Json(serde_json::json!({"success": true}))).into_response()
}

#[debug_handler]
pub async fn admin_items(State(state): State<AppState>, cookie: Option<TypedHeader<Cookie>>) -> impl IntoResponse {
    if !require_admin_token(&state.db_path, extract_admin_token(cookie).as_deref()).await {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response();
    }
    
    let conn = match Connection::open(&state.db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to open database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"Database error"}))).into_response();
        }
    };
    
    let mut stmt = match conn.prepare("SELECT code, kind, value, created_at FROM items ORDER BY created_at DESC LIMIT 500") {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare statement: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"Query error"}))).into_response();
        }
    };
    
    let rows = match stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?))) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to query items: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"Query error"}))).into_response();
        }
    };
    
    let mut items: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        if let Ok((code, kind, value, created_at)) = row {
            items.push(serde_json::json!({
                "code": code,
                "kind": kind,
                "value": value,
                "created_at": created_at
            }));
        }
    }
    
    (StatusCode::OK, Json(items)).into_response()
}

#[debug_handler]
pub async fn admin_delete_item(
    State(state): State<AppState>,
    Path(code): Path<String>,
    cookie: Option<TypedHeader<Cookie>>,
) -> impl IntoResponse {
    if !require_admin_token(&state.db_path, extract_admin_token(cookie).as_deref()).await {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response();
    }
    // Query needed info inside a short-lived DB connection (avoid holding Connection across awaits)
    let kind_value: Option<(String, String)> = {
        let conn = Connection::open(&state.db_path).unwrap();
        conn.query_row(
            "SELECT kind, value FROM items WHERE code = ?1",
            params![code.clone()],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        ).ok()
    };

    if let Some((kind, value)) = kind_value {
        if kind == "file" {
            if let Some(fname) = value.strip_prefix("file:") {
                let path_to_delete = std::path::PathBuf::from(&state.uploads_dir).join(fname);
                let _ = tokio::fs::remove_file(&path_to_delete).await;
                let preview_name = make_preview_filename(fname);
                let preview_path = std::path::PathBuf::from(&state.uploads_dir).join("previews").join(preview_name);
                let _ = tokio::fs::remove_file(preview_path).await;
            }
        }
    }

    // Now delete the DB row in a fresh connection
    {
        let conn = Connection::open(&state.db_path).unwrap();
        let _ = conn.execute("DELETE FROM items WHERE code = ?1", params![code]);
    }
    (StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response()
}


#[debug_handler]
pub async fn api_upload(State(state): State<AppState>, mut multipart: Multipart) -> axum::response::Response {
    let mut link_value: Option<String> = None;
    let mut saved_filename: Option<String> = None;
    let mut qr_required: bool = false;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        match name {
            "content" => {
                if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                    let ext = StdPath::new(&fname).extension().and_then(|e| e.to_str()).unwrap_or("bin");
                    if !is_allowed_extension(ext) { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "File type not allowed"}))).into_response(); }
                    if let Err(e) = fs::create_dir_all(&state.uploads_dir).await { tracing::error!("create uploads dir: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); }
                    let id = Uuid::new_v4();
                    let filename_saved = format!("{}.{}", id, ext);
                    let path = format!("{}/{}", state.uploads_dir, filename_saved);
                    let mut out = match tokio::fs::File::create(&path).await { Ok(f) => f, Err(e) => { tracing::error!("create file: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); } };
                    let mut written: usize = 0;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        written = written.saturating_add(chunk.len());
                        if written > MAX_FILE_SIZE { let _ = tokio::fs::remove_file(&path).await; return (StatusCode::PAYLOAD_TOO_LARGE, Json(serde_json::json!({"success": false, "error": "File too large"}))).into_response(); }
                        if let Err(e) = out.write_all(&chunk).await { tracing::error!("write: {}", e); let _ = tokio::fs::remove_file(&path).await; return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); }
                    }
                    saved_filename = Some(filename_saved);
                } else if let Ok(text) = field.text().await {
                    if !text.trim().is_empty() { link_value = Some(text.trim().to_string()); }
                }
            }
            "qr_required" => {
                if let Ok(v) = field.text().await { qr_required = v.trim().eq_ignore_ascii_case("true"); }
            }
            _ => {}
        }
    }

    if let Some(filename_saved) = saved_filename {
        let short_code = nanoid!(8);
        {
            let conn = Connection::open(&state.db_path).unwrap();
            let original = format!("file:{}", filename_saved);
            conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "file", original]).ok();
        }
        let short_url = format!("{}/s/{}", state.base_url, short_code);
        let qr_code_data = if qr_required {
            let qr_target = ensure_absolute(&state.base_url, &short_url);
            match QrCode::new(qr_target.as_bytes()) {
                Ok(c) => {
                    let image = c
                        .render::<qrcode::render::svg::Color>()
                        .min_dimensions(320,320)
                        .quiet_zone(true)
                        .dark_color(Color("#000000"))
                        .light_color(Color("#ffffff"))
                        .build();
                    let data_url = format!("data:image/svg+xml;utf8,{}", urlencoding::encode(&image));
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else { None };
        return Json(serde_json::json!({"success": true, "short_url": short_url, "qr_code_data": qr_code_data})).into_response();
    }

    if let Some(link) = link_value {
        if !link.starts_with("http://") && !link.starts_with("https://") { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Invalid URL"}))).into_response(); }
        let short_code = nanoid!(8);
        {
            let conn = Connection::open(&state.db_path).unwrap();
            conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "url", link]).ok();
        }
        let short_url = format!("{}/s/{}", state.base_url, short_code);
        let qr_code_data = if qr_required {
            let qr_target = ensure_absolute(&state.base_url, &short_url);
            match QrCode::new(qr_target.as_bytes()) {
                Ok(c) => {
                    let image = c
                        .render::<qrcode::render::svg::Color>()
                        .min_dimensions(320,320)
                        .quiet_zone(true)
                        .dark_color(Color("#000000"))
                        .light_color(Color("#ffffff"))
                        .build();
                    let data_url = format!("data:image/svg+xml;utf8,{}", urlencoding::encode(&image));
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else { None };
        return Json(serde_json::json!({"success": true, "short_url": short_url, "qr_code_data": qr_code_data})).into_response();
    }

    (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Provide content or file"}))).into_response()
}