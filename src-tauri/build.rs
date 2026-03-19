use std::{fs, path::Path};

const ICON_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

fn ensure_icon() {
    let icon_path = Path::new("icons/icon.png");
    if icon_path.exists() {
        return;
    }

    if let Some(parent) = icon_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(bytes) = decode_base64(ICON_PNG_BASE64) {
        let _ = fs::write(icon_path, bytes);
    }
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut cleaned = input.trim().replace('\n', "");
    while cleaned.ends_with('=') {
        cleaned.pop();
    }

    let mut buffer = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0u8;

    for ch in cleaned.bytes() {
        let value = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("base64 inválido".into()),
        } as u32;

        acc = (acc << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buffer.push(((acc >> bits) & 0xFF) as u8);
        }
    }

    Ok(buffer)
}

fn main() {
    ensure_icon();
    tauri_build::build()
}
