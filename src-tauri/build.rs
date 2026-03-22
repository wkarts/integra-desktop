use std::{fs, path::Path, process::Command};

const ICON_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAQAAAAEACAYAAABccqhmAAAE0klEQVR42u3dsZHCQBAAwfWJkTyIlxBwMEXJUADgIN20qiaAv91tk5/583e7Pzap2hQ+g5ZCOBieFATBkKQgBIYiBSEwBCkKgYeXogh4cCmIgEeWohB4WCmKgAeVogh4SCmMgEeUogB4QCmKgIeTogh4MCmMgMeSogB4KCmMgEeSogB4ICmMgMeRogB4GCmMgEeRACAJAJIyAHgQKYyAx5AAIAkAkgAgCQCSACBpRQAKf+jz9Za+LoGAI5e6OIzDl7oQjMOXuhCMw5e6EIzDl7oQjOOXugiM45e6CIzjl7oIjOOXugiM45e6CABAAoDjl4oIAEACgOOXiggAQAKA45eKCABAAoDjl4oIAEACAAAkAABAAgAAJAAAQAIAACQAOH5pOQQAIAEAABIAACABAAASAAAgAQAAEgAAsOwPQ/pZOAAYNgAAAADDBgAAAGDYALATADBsANgJABg2AOwEAAwbAHYCAIYNADsBAMMGgJ0AgGEDwE4AwLABYCcAYNgAsBMAMGwA2AkAGDYA7AQADBsAdgIAhg0AOwEAADhwOwEAAMhOAAAAAgAADFwAAICBCwAAMHAA2AcAGDgA7AMADBwA9gEABg4A+wAAAweAfQCAgQPAPgDAwAFgHwDgIHXVHQQAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAAAAAAQAAABAAACAAxAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAACAAAAAAAgAAHIAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAAAABAAAAEAAAcCIAgOMgAWDYAAAAAAwbAAAAgGEDwE4AwLABYCcAYNgAsBMAMGwA2AkAGDYA7AQADBsAdgIAhg0AOwEAwwaAnQCAYQPATgDAsAFgJwBg2ACwEwAwbADYCQAYNgDsBAAA4MDtBAAAIDsBAAAIAAAwcAEAAAYuAADAwAFgHwBg4ACwDwAwcADYBwAYOADsAwAMHAD2AQAGDgD7AAADB4B9AIAEAABIAACABAAASAAAgAQAAEgAOCcAEJDjB4AEAABIAACABAAASAAAgAQAAEgAWBIACMjxA0ACQBUACMjxA0ACQBUACMjxA0ACQBUACMjxA0ACQBUACMjxxwGAgBx/HAAIyPHHAYCAHH8cAAjI8ccBAIEcPgBAIIcPABDI4QMABHL4AICDHPmPAOyf/0kv9Zrj8xgSACQBQBIAJAFAEgAkrQoABKTw8QNAAoCHkQAgKQcABKTw8QNAigMAASl8/ACQ4gBAQAofPwCkOAAQkMLHDwEpfvwQkOLHDwApDgAEpPDxQ0CKHz8EpPjxg0CKHz4EJMcPAal+/CCQ4ocPAsnhg0By+ECQHDwcpMsc+QeZ0O7dSJFcSgAAAABJRU5ErkJggg==";

fn ensure_png_icons() {
    let png_bytes = match decode_base64(ICON_PNG_BASE64) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };

    let icon_dir = Path::new("icons");
    let _ = fs::create_dir_all(icon_dir);
    let _ = fs::write(icon_dir.join("icon.png"), &png_bytes);
    let _ = fs::write(icon_dir.join("32x32.png"), &png_bytes);
    let _ = fs::write(icon_dir.join("128x128.png"), &png_bytes);
    let _ = fs::write(icon_dir.join("128x128@2x.png"), &png_bytes);
}

fn emit_git_hash() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "dev-local".to_string());

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut cleaned: String = input
        .trim()
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .collect();

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
    ensure_png_icons();
    emit_git_hash();
    tauri_build::build()
}
