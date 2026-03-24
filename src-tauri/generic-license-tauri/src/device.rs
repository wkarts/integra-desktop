use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::models::LicenseCheckInput;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceCollectedInfo {
    pub station_name: String,
    pub hostname: String,
    pub computer_name: String,
    pub serial_number: String,
    pub machine_guid: String,
    pub bios_serial: String,
    pub motherboard_serial: String,
    pub logged_user: String,
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub domain_name: String,
    pub mac_addresses: Vec<String>,
    pub install_mode: String,
}

pub fn enrich_input(input: &mut LicenseCheckInput) {
    let info = collect_device_metadata();

    if input.hostname.as_deref().unwrap_or("").is_empty() {
        input.hostname = Some(info.hostname.clone());
    }
    if input.computer_name.as_deref().unwrap_or("").is_empty() {
        input.computer_name = Some(info.computer_name.clone());
    }
    if input.station_name.as_deref().unwrap_or("").is_empty() {
        input.station_name = Some(info.station_name.clone());
    }
    if input.device_name.as_deref().unwrap_or("").is_empty() {
        input.device_name = Some(info.station_name.clone());
    }
    if input.serial_number.as_deref().unwrap_or("").is_empty() {
        input.serial_number = non_empty_or_none(info.serial_number.clone());
    }
    if input.machine_guid.as_deref().unwrap_or("").is_empty() {
        input.machine_guid = non_empty_or_none(info.machine_guid.clone());
    }
    if input.bios_serial.as_deref().unwrap_or("").is_empty() {
        input.bios_serial = non_empty_or_none(info.bios_serial.clone());
    }
    if input.motherboard_serial.as_deref().unwrap_or("").is_empty() {
        input.motherboard_serial = non_empty_or_none(info.motherboard_serial.clone());
    }
    if input.logged_user.as_deref().unwrap_or("").is_empty() {
        input.logged_user = non_empty_or_none(info.logged_user.clone());
    }
    if input.os_name.as_deref().unwrap_or("").is_empty() {
        input.os_name = non_empty_or_none(info.os_name.clone());
    }
    if input.os_version.as_deref().unwrap_or("").is_empty() {
        input.os_version = non_empty_or_none(info.os_version.clone());
    }
    if input.os_arch.as_deref().unwrap_or("").is_empty() {
        input.os_arch = non_empty_or_none(info.os_arch.clone());
    }
    if input.domain_name.as_deref().unwrap_or("").is_empty() {
        input.domain_name = non_empty_or_none(info.domain_name.clone());
    }
    if input.mac_addresses.is_empty() {
        input.mac_addresses = info.mac_addresses.clone();
    }
    if input.install_mode.as_deref().unwrap_or("").is_empty() {
        input.install_mode = non_empty_or_none(info.install_mode.clone());
    }

    if input.device_key.as_deref().unwrap_or("").is_empty() {
        input.device_key = Some(generate_device_key(input));
    }
}

pub fn hostname_or_unknown() -> String {
    hostname::get()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())
}

pub fn current_user_or_unknown() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".to_string())
}

pub fn default_device_name() -> String {
    let info = collect_device_metadata();
    info.station_name
}

pub fn generate_device_key(input: &LicenseCheckInput) -> String {
    let parts = vec![
        input.app_id.clone(),
        input.app_slug.clone().unwrap_or_default(),
        input.machine_guid.clone().unwrap_or_default(),
        input.bios_serial.clone().unwrap_or_default(),
        input.motherboard_serial.clone().unwrap_or_default(),
        input.mac_addresses.first().cloned().unwrap_or_default(),
        input.hostname.clone().unwrap_or_default(),
        input.os_name.clone().unwrap_or_default(),
        input.os_arch.clone().unwrap_or_default(),
    ];

    let mut hasher = Sha256::new();
    hasher.update(parts.join("|").as_bytes());
    hex::encode(hasher.finalize())
}

pub fn collect_device_metadata() -> DeviceCollectedInfo {
    let hostname = hostname_or_unknown();
    let computer_name = env::var("COMPUTERNAME").unwrap_or_else(|_| hostname.clone());
    let logged_user = current_user_or_unknown();
    let os_name = env::consts::OS.to_string();
    let os_arch = env::consts::ARCH.to_string();
    let os_version = os_version();
    let serial_number = first_non_empty(vec![system_serial_number(), bios_serial()]);
    let machine_guid = machine_guid();
    let bios_serial = bios_serial();
    let motherboard_serial = motherboard_serial();
    let domain_name = domain_name();
    let mac_addresses = mac_addresses();
    let install_mode = detect_install_mode();
    let station_name = hostname.clone();

    DeviceCollectedInfo {
        station_name,
        hostname,
        computer_name,
        serial_number,
        machine_guid,
        bios_serial,
        motherboard_serial,
        logged_user,
        os_name,
        os_version,
        os_arch,
        domain_name,
        mac_addresses,
        install_mode,
    }
}

fn detect_install_mode() -> String {
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_str = exe.to_string_lossy().to_string();

    if env::var("SESSIONNAME")
        .unwrap_or_default()
        .to_uppercase()
        .contains("RDP")
    {
        return "terminal-session".to_string();
    }

    if exe_str.starts_with(r"\\") || exe_str.starts_with("//") {
        return "shared-server".to_string();
    }

    "workstation".to_string()
}

fn os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        return read_command_output("cmd", &["/C", "ver"]);
    }
    #[cfg(target_os = "linux")]
    {
        let release = fs::read_to_string("/etc/os-release").unwrap_or_default();
        let pretty = release
            .lines()
            .find(|l| l.starts_with("PRETTY_NAME="))
            .map(|l| l.replace("PRETTY_NAME=", "").trim_matches('"').to_string())
            .unwrap_or_default();
        if !pretty.is_empty() {
            return pretty;
        }
        return read_command_output("uname", &["-r"]);
    }
    #[cfg(target_os = "macos")]
    {
        return read_command_output("sw_vers", &["-productVersion"]);
    }
    #[allow(unreachable_code)]
    String::new()
}

fn machine_guid() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = read_command_output(
            "reg",
            &[
                "query",
                r"HKLM\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ],
        );
        return parse_last_token(output);
    }
    #[cfg(target_os = "linux")]
    {
        let candidates = ["/etc/machine-id", "/var/lib/dbus/machine-id"];
        for path in candidates {
            let content = fs::read_to_string(path)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !content.is_empty() {
                return content;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = read_command_output("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"]);
        for line in output.lines() {
            if line.contains("IOPlatformUUID") {
                return line
                    .split('=')
                    .nth(1)
                    .unwrap_or("")
                    .replace('"', "")
                    .trim()
                    .to_string();
            }
        }
    }
    String::new()
}

fn bios_serial() -> String {
    #[cfg(target_os = "windows")]
    {
        return parse_serial_lines(read_command_output(
            "wmic",
            &["bios", "get", "serialnumber"],
        ));
    }
    #[cfg(target_os = "linux")]
    {
        let val = fs::read_to_string("/sys/class/dmi/id/bios_version").unwrap_or_default();
        if !val.trim().is_empty() {
            return val.trim().to_string();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = read_command_output("system_profiler", &["SPHardwareDataType"]);
        for line in output.lines() {
            if line.contains("Serial Number") {
                return line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
    }
    String::new()
}

fn motherboard_serial() -> String {
    #[cfg(target_os = "windows")]
    {
        return parse_serial_lines(read_command_output(
            "wmic",
            &["baseboard", "get", "serialnumber"],
        ));
    }
    #[cfg(target_os = "linux")]
    {
        let val = fs::read_to_string("/sys/class/dmi/id/board_serial").unwrap_or_default();
        if !val.trim().is_empty() {
            return val.trim().to_string();
        }
    }
    #[cfg(target_os = "macos")]
    {
        return String::new();
    }
    String::new()
}

fn system_serial_number() -> String {
    #[cfg(target_os = "windows")]
    {
        return parse_serial_lines(read_command_output("wmic", &["csproduct", "get", "uuid"]));
    }
    #[cfg(target_os = "linux")]
    {
        let val = fs::read_to_string("/sys/class/dmi/id/product_uuid").unwrap_or_default();
        if !val.trim().is_empty() {
            return val.trim().to_string();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = read_command_output("system_profiler", &["SPHardwareDataType"]);
        for line in output.lines() {
            if line.contains("Hardware UUID") {
                return line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
    }
    String::new()
}

fn domain_name() -> String {
    env::var("USERDOMAIN")
        .or_else(|_| env::var("DOMAINNAME"))
        .unwrap_or_default()
}

fn mac_addresses() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let output = read_command_output("getmac", &["/fo", "csv", "/nh"]);
        return output
            .lines()
            .filter_map(|line| {
                let clean = line.replace('"', "");
                clean.split(',').next().map(|v| v.trim().to_string())
            })
            .filter(|v| !v.is_empty() && v.contains('-'))
            .collect();
    }
    #[cfg(target_os = "linux")]
    {
        let output = read_command_output(
            "sh",
            &["-c", "ip link | grep link/ether | awk '{print $2}'"],
        );
        return output
            .lines()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
    }
    #[cfg(target_os = "macos")]
    {
        let output = read_command_output(
            "sh",
            &[
                "-c",
                "networksetup -listallhardwareports | grep 'Ethernet Address' | awk '{print $3}'",
            ],
        );
        return output
            .lines()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
    }
    #[allow(unreachable_code)]
    Vec::new()
}

fn read_command_output(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_default()
}

#[allow(dead_code)]
fn parse_last_token(value: String) -> String {
    value
        .split_whitespace()
        .last()
        .unwrap_or("")
        .trim()
        .to_string()
}

#[allow(dead_code)]
fn parse_serial_lines(value: String) -> String {
    value
        .lines()
        .map(|v| v.trim())
        .find(|v| {
            !v.is_empty()
                && !v.eq_ignore_ascii_case("serialnumber")
                && !v.eq_ignore_ascii_case("uuid")
        })
        .unwrap_or("")
        .to_string()
}

fn first_non_empty(values: Vec<String>) -> String {
    values
        .into_iter()
        .find(|v| !v.trim().is_empty())
        .unwrap_or_default()
}

fn non_empty_or_none(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}
