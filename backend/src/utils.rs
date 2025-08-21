use crate::services::search_service::SortOrder;

/// Parse ISO8601 date string to Unix timestamp for sorting
pub fn parse_iso8601_to_timestamp(date_str: &str) -> i64 {
    if date_str.is_empty() {
        return 0;
    }

    use chrono::{DateTime, Utc};
    if let Ok(dt) = date_str.parse::<DateTime<Utc>>() {
        return dt.timestamp();
    }

    0
}

/// Parse ISO8601 duration string (PT1H2M3S) to total seconds for sorting
pub fn parse_iso8601_duration_to_seconds(duration_str: &str) -> i64 {
    if duration_str.is_empty() {
        return 0;
    }

    // Simple parser for PT format (PT1H2M3S)
    if !duration_str.starts_with("PT") {
        return 0;
    }

    let duration_part = &duration_str[2..]; // Remove "PT"
    let mut total_seconds = 0.0;
    let mut current_number = String::new();

    for ch in duration_part.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current_number.push(ch);
        } else {
            if let Ok(num) = current_number.parse::<f64>() {
                match ch {
                    'H' => total_seconds += num * 3600.0, // Hours
                    'M' => total_seconds += num * 60.0,   // Minutes
                    'S' => total_seconds += num,          // Seconds
                    _ => {}
                }
            }
            current_number.clear();
        }
    }

    total_seconds as i64
}

pub fn compare_with_order_float(a: f64, b: f64, order: &SortOrder) -> std::cmp::Ordering {
    match order {
        SortOrder::Asc => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
        SortOrder::Desc => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
    }
}
pub fn compare_with_order_int(a: i64, b: i64, order: &SortOrder) -> std::cmp::Ordering {
    compare_with_order_float(a as f64, b as f64, order)
}

pub fn extract_youtube_video_id(url: &str) -> Option<String> {
    if let Some(captures) = regex::Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})",
    )
    .ok()?
    .captures(url)
    {
        return captures.get(1).map(|m| m.as_str().to_string());
    }
    None
}
