use std::{fs, path::Path};

const ICON_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

fn ensure_icons() {
    let png_bytes = match decode_base64(ICON_PNG_BASE64) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };

    let icon_dir = Path::new("icons");
    let _ = fs::create_dir_all(icon_dir);

    write_if_missing(icon_dir.join("icon.png"), &png_bytes);
    write_if_missing(icon_dir.join("32x32.png"), &png_bytes);
    write_if_missing(icon_dir.join("128x128.png"), &png_bytes);
    write_if_missing(icon_dir.join("128x128@2x.png"), &png_bytes);

    let ico = build_ico(&png_bytes);
    write_if_missing(icon_dir.join("icon.ico"), &ico);

    let icns = build_icns(&png_bytes);
    write_if_missing(icon_dir.join("icon.icns"), &icns);
}

fn write_if_missing(path: impl AsRef<Path>, bytes: &[u8]) {
    let path = path.as_ref();
    if path.exists() {
        return;
    }
    let _ = fs::write(path, bytes);
}

fn build_ico(png: &[u8]) -> Vec<u8> {
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    ico.extend_from_slice(&1u16.to_le_bytes()); // count

    ico.push(1); // width
    ico.push(1); // height
    ico.push(0); // colors
    ico.push(0); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // color planes
    ico.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes()); // size
    ico.extend_from_slice(&(22u32).to_le_bytes()); // data offset
    ico.extend_from_slice(png);
    ico
}

fn build_icns(png: &[u8]) -> Vec<u8> {
    let total_len = 8 + 8 + png.len();
    let mut icns = Vec::new();
    icns.extend_from_slice(b"icns");
    icns.extend_from_slice(&(total_len as u32).to_be_bytes());
    icns.extend_from_slice(b"ic07");
    icns.extend_from_slice(&((8 + png.len()) as u32).to_be_bytes());
    icns.extend_from_slice(png);
    icns
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
    ensure_icons();
    tauri_build::build()
}
