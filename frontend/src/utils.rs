pub fn format_iso8601_date(iso_date: &str) -> String {
    if let Ok(datetime) = iso_date.parse::<chrono::DateTime<chrono::Utc>>() {
        datetime.format("%Y-%m-%d").to_string()
    } else {
        iso_date.to_string()
    }
}

// Formats each x1000 step
pub fn format_number(number: i64) -> String {
    let num_str = number.to_string();
    let mut result = String::new();
    let len = num_str.len();

    for (i, c) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

pub fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds as u32) / 60;
    let remaining_seconds = (seconds as u32) % 60;
    format!("{:02}:{:02}", minutes, remaining_seconds)
}

pub fn format_iso8601_duration(duration: &str) -> String {
    let hours = duration
        .find('H')
        .map_or(0, |h| duration[2..h].parse::<u32>().unwrap_or(0));
    let minutes = duration.find('M').map_or(0, |m| {
        let start = duration.find('H').map_or(2, |h| h + 1);
        duration[start..m].parse::<u32>().unwrap_or(0)
    });
    let seconds = duration.find('S').map_or(0, |s| {
        let start = duration
            .find('M')
            .map_or_else(|| duration.find('H').map_or(2, |h| h + 1), |m| m + 1);
        duration[start..s].parse::<u32>().unwrap_or(0)
    });
    if hours != 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}
