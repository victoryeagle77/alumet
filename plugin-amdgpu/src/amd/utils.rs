use regex::Regex;
use std::process::Command;

enum EnergyType {
    Float,
    Integer,
}

fn execute_amd_command() -> String {
    let output = Command::new("amd-smi")
        .arg("metric")
        .output()
        .expect("Error of amd-smi execution");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn get_simple_value(pattern: &str, energy_type: EnergyType) -> Result<f64, String> {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return Err(format!("Regex compilation error: {}", e)),
    };
    let output = execute_amd_command();

    if let Some(captures) = regex.captures(&output) {
        if let Some(value) = captures.get(1) {
            match energy_type {
                EnergyType::Float => {
                    return value
                        .as_str()
                        .parse::<f64>()
                        .map_err(|e| format!("f64 conversion error: {}", e));
                }
                EnergyType::Integer => {
                    return value
                        .as_str()
                        .parse::<u64>()
                        .map(|v| v as f64)
                        .map_err(|e| format!("u64 conversion error: {}", e));
                }
            }
        }
    }
    Err("Value not found.".to_string())
}

/// Retrieves GPU energy consumption.
fn get_energy() -> f64 {
    get_simple_value(r"TOTAL_ENERGY_CONSUMPTION:\s*([\d.]+)\s*J", EnergyType::Float).unwrap_or(0.0)
}

/// Retrieves GPU electric power consumption.
fn get_power() -> u64 {
    get_simple_value(r"SOCKET_POWER:\s*([\d.]+)\s*W", EnergyType::Integer).unwrap_or(0.0) as u64
}

/// Retrieves GPU total RAM memory.
fn get_total_ram() -> u64 {
    get_simple_value(r"TOTAL_VRAM:\s*([\d.]+)\s*MB", EnergyType::Integer).unwrap_or(0.0) as u64
}

/// Retrieves GPU used RAM memory.
fn get_used_ram() -> u64 {
    get_simple_value(r"USED_VRAM:\s*([\d.]+)\s*MB", EnergyType::Integer).unwrap_or(0.0) as u64
}

pub struct Metric {
    /// GPU energy consumption.
    pub energy: f64,
    /// GPU electric power consumption.
    pub power: u64,
    /// GPU total RAM memory.
    pub memory_total: u64,
    /// GPU used RAM memory.
    pub memory_used: u64,
}

pub fn create_metric() -> Metric {
    Metric {
        energy: get_energy(),
        power: get_power(),
        memory_total: get_total_ram(),
        memory_used: get_used_ram(),
    }
}
