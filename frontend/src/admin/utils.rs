use web_sys::window;

pub fn get_stored_admin_token() -> Option<String> {
    window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
}

pub fn store_admin_token(token: &str) -> Result<(), String> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.session_storage() {
            storage
                .set_item("admin_token", token)
                .map_err(|_| "Failed to store token".to_string())?;
        }
    }
    Ok(())
}

pub fn remove_admin_token() -> Result<(), String> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.session_storage() {
            storage
                .remove_item("admin_token")
                .map_err(|_| "Failed to remove token".to_string())?;
        }
    }
    Ok(())
}

pub fn format_unix_time_since(unix_timestamp: u64) -> String {
    let now = chrono::Utc::now();
    let date = match chrono::DateTime::<chrono::Utc>::from_timestamp(unix_timestamp as i64, 0) {
        Some(d) => d,
        None => return String::from("Invalid date"),
    };

    let duration = now.signed_duration_since(date);
    let seconds = duration.num_seconds();

    if seconds < 60 {
        return format!("{}s", seconds);
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        let remaining_seconds = seconds % 60;
        return format!("{}m {}s ago", minutes, remaining_seconds);
    }

    let hours = minutes / 60;
    if hours < 24 {
        let remaining_minutes = minutes % 60;
        return format!("{}h {}m ago", hours, remaining_minutes);
    }

    let days = hours / 24;
    let remaining_hours = hours % 24;
    format!("{}d {}h ago", days, remaining_hours)
}
