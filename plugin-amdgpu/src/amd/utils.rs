use log::{error, info, warn};
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

use amdsmi::*;

/// Initialize the AMD SMI library
fn init() {
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => info!("AMD SMI initialized successfully"),
        Err(e) => panic!("Failed to initialize AMD SMI: {e}"),
    }
}

/// Shutdown the AMD SMI library
fn quit() {
    match amdsmi_shut_down() {
        Ok(_) => info!("AMD SMI shut down successfully"),
        Err(e) => panic!("Failed to shut down AMD SMI: {e}"),
    }
}

fn test() {
    init();

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            panic!("Failed to get socket handles: {e}");
            quit();
            return;
        }
    };

    for socket_handle in socket_handles {
        // Get processor handles for each socket handle
        let processor_handles = match amdsmi_get_processor_handles(socket_handle) {
            Ok(handles) => handles,
            Err(e) => {
                error!("Failed to get processor handles for socket {socket_handle:?}: {e}");
                continue;
            }
        };

        for processor_handle in processor_handles {
            // Get GPU ID using the processor handle
            match amdsmi_get_gpu_id(processor_handle) {
                Ok(data) => {
                    let id = data;
                }
                Err(e) => error!("Failed to get GPU ID: {e}"),
            }

            // Get GPU energy consumption in Joules
            match amdsmi_get_energy_count(processor_handle) {
                Ok((energy_accumulator, counter_resolution, timestamp)) => {
                    let energy = (energy_accumulator * counter_resolution) / 1e3;
                }
                Err(e) => error!("Failed to get energy count: {e}"),
            }

            // Get average power consumption GPU in Watts
            match amdsmi_get_power_info(processor_handle, sensor_ind) {
                Ok(data) => {
                    let power = data.average_socket_power;
                }
                Err(e) => error!("Failed to get power information: {e}"),
            }
            match amdsmi_get_power_cap_info(processor_handle, 0) {
                Ok(data) => {
                    let power_test = data.power_cap;
                }
                Err(e) => panic!("Failed to get power cap information: {e}"),
            };

            // Get GPU VRAM memory usage in MB
            match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram) {
                Ok(data) => {
                    let vram_usage = data / 1e6;
                }
                Err(e) => error!("Failed to get GPU memory usage: {e}"),
            }
            // Get GPU GTT memory usage in MB
            match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt) {
                Ok(data) => {
                    let gtt_usage = data / 1e6;
                }
                Err(e) => error!("Failed to get GPU memory usage: {e}"),
            }

            // Get GPU current temperature metric by hardware sectors
            let sensor_type = AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeEdge;
            let metric = AmdsmiTemperatureMetricT::AmdsmiTempCurrent;
            match amdsmi_get_temp_metric(processor_handle, sensor_type, metric) {
                Ok(data) => println!("GPU Temperature Metric: {}", data),
                Err(e) => error!("Failed to get GPU temperature metric: {e}"),
            }
        }
    }

    quit();
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
