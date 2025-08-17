use crate::admin::models::{AdminLoginRequest, AdminLoginResponse, AdminStats};
use gloo_net::http::Request;

pub async fn login_admin(token: &str) -> Result<AdminLoginResponse, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/login", backend_url);

    let request_body = AdminLoginRequest {
        token: token.to_string(),
    };

    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<AdminLoginResponse>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn load_admin_stats(token: &str) -> Result<AdminStats, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/stats", backend_url);

    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<AdminStats>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
