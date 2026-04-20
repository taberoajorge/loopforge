use std::time::Duration;

pub async fn check_health(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    match client
        .post(url)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status().as_u16();
            status != 0
        }
        Err(_) => false,
    }
}

pub async fn wait_for_health(service_name: &str, url: &str, max_wait_secs: u64) -> bool {
    let interval = Duration::from_secs(3);
    let mut elapsed: u64 = 0;

    while elapsed < max_wait_secs {
        if check_health(url).await {
            crate::logger::log_success(&format!("{service_name} is healthy after {elapsed}s"));
            return true;
        }
        tokio::time::sleep(interval).await;
        elapsed += 3;
        tracing::debug!(
            service = service_name,
            elapsed,
            max_wait = max_wait_secs,
            "Waiting for service health"
        );
    }
    tracing::warn!(service = service_name, "Health check timed out");
    false
}
