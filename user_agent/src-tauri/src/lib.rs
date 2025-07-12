use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    os: String,
    arch: String,
    hostname: String,
    username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAgentInfo {
    name: String,
    version: String,
    capabilities: Vec<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    
    let hostname = Command::new("hostname")
        .output()
        .map_err(|e| format!("Failed to get hostname: {}", e))?
        .stdout;
    let hostname = String::from_utf8(hostname)
        .map_err(|e| format!("Invalid hostname encoding: {}", e))?
        .trim()
        .to_string();

    let username = std::env::var("USER")
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        os,
        arch,
        hostname,
        username,
    })
}

#[tauri::command]
fn get_user_agent_info() -> UserAgentInfo {
    UserAgentInfo {
        name: "Keahi Ambient Agent".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "System Information".to_string(),
            "File Operations".to_string(),
            "Network Communication".to_string(),
            "Process Management".to_string(),
        ],
    }
}

#[tauri::command]
fn execute_command(command: &str, args: Vec<String>) -> Result<String, String> {
    let output = Command::new(command)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid output encoding: {}", e))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| format!("Invalid error encoding: {}", e))?;
        Err(format!("Command failed: {}", stderr))
    }
}

#[tauri::command]
fn get_environment_vars() -> Result<HashMap<String, String>, String> {
    let mut env_vars = HashMap::new();
    for (key, value) in std::env::vars() {
        env_vars.insert(key, value);
    }
    Ok(env_vars)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            get_user_agent_info,
            execute_command,
            get_environment_vars
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
