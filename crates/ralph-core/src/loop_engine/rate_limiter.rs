use crate::providers::AgentResult;

pub fn compute_wait(result: &AgentResult, default_secs: u64) -> u64 {
    if let Some(ref retry_hint) = result.retry_after_message {
        if let Some(secs) = parse_retry_time_to_secs(retry_hint) {
            return secs.min(3600);
        }
    }
    default_secs
}

fn parse_retry_time_to_secs(time_str: &str) -> Option<u64> {
    let now = chrono::Local::now();
    let cleaned = time_str.trim().replace('.', "").to_uppercase();

    let is_pm = cleaned.contains("PM");
    let is_am = cleaned.contains("AM");
    let digits_only = cleaned
        .replace("PM", "")
        .replace("AM", "")
        .trim()
        .to_string();

    let parts: Vec<&str> = digits_only.split(':').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }

    let mut hour: u32 = parts[0].trim().parse().ok()?;
    let minute: u32 = if parts.len() == 2 {
        parts[1].trim().parse().ok()?
    } else {
        0
    };

    if is_pm && hour < 12 {
        hour += 12;
    } else if is_am && hour == 12 {
        hour = 0;
    }

    let target = now.date_naive().and_hms_opt(hour, minute, 0)?;
    let target_dt = target.and_local_timezone(now.timezone()).single()?;

    let diff = target_dt.signed_duration_since(now);
    if diff.num_seconds() <= 0 {
        return Some(120);
    }
    Some(diff.num_seconds() as u64 + 30)
}
