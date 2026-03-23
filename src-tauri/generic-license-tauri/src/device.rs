use sha2::{Digest, Sha256};

pub fn hostname_or_unknown() -> String {
    hostname::get()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())
}

pub fn default_device_name() -> String {
    format!(
        "{} - {} - {}",
        hostname_or_unknown(),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

pub fn generate_device_key(app_id: &str) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        app_id,
        hostname_or_unknown(),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}
