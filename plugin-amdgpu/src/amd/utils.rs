use alumet::pipeline::elements::{error::PollError, source::error::PollRetry};
use amdsmi::*;
use anyhow::Context;
use log::{error, warn};
use std::collections::HashMap;

use super::error::AmdError;

// All clock frequencies values available
const CLK_TYPE: [(AmdsmiClkTypeT, &str); 10] = [
    (AmdsmiClkTypeT::AmdsmiClkTypeSys, "System"),
    (AmdsmiClkTypeT::AmdsmiClkTypeDf, "DisplayFactory"),
    (AmdsmiClkTypeT::AmdsmiClkTypeDcef, "DisplayControllerEngineFrequency"),
    (AmdsmiClkTypeT::AmdsmiClkTypeSoc, "SystemOnChip"),
    (AmdsmiClkTypeT::AmdsmiClkTypeMem, "Memory"),
    (AmdsmiClkTypeT::AmdsmiClkTypePcie, "PCIe"),
    (AmdsmiClkTypeT::AmdsmiClkTypeVclk0, "VideoCore_0"),
    (AmdsmiClkTypeT::AmdsmiClkTypeVclk1, "VideoCore_1"),
    (AmdsmiClkTypeT::AmdsmiClkTypeDclk0, "Display_0"),
    (AmdsmiClkTypeT::AmdsmiClkTypeDclk1, "Display_1"),
];

// All temperature sensors values available
const SENSOR_TYPE: [(AmdsmiTemperatureTypeT, &str); 8] = [
    (AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeEdge, "Global"),
    (AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHotspot, "Hotspot"),
    (AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeVram, "Vram"),
    (
        AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm0,
        "HighBandwidthMemory_0",
    ),
    (
        AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm1,
        "HighBandwidthMemory_1",
    ),
    (
        AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm2,
        "HighBandwidthMemory_2",
    ),
    (
        AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm3,
        "HighBandwidthMemory_3",
    ),
    (AmdsmiTemperatureTypeT::AmdsmiTemperatureTypePlx, "PCIe"),
];

/// Structure to collect AMD GPU metrics.
pub struct AmdGpuMetric {
    /// GPU identification by BDF PCIe bus.
    pub id: String,
    /// GPU clock frequencies in Mhz, with label describing its associated clock type.
    pub clocks: HashMap<String, u64>,
    /// GPU energy consumption in J.
    pub energy: f64,
    /// GPU Video computing memory (VRAM) usage in MB.
    pub memory_usage_gtt: u64,
    /// GPU Graphic Translation Table memory (GTT) usage in MB.
    pub memory_usage_vram: u64,
    /// GPU electric power consumption in W.
    pub power_average: u64,
    /// GPU temperature in °C, with label describing its associated thermal zone.
    pub temperatures: HashMap<String, u64>,
    /// GPU counter of running compute processes.
    pub process_counter: u64,
    /// GPU process PID.
    pub process_pid: u32,
    /// GPU process VRAM memory usage in MB.
    pub process_usage_vram: u64,
    /// GPU process compute unit usage in percentage.
    pub process_usage_compute_unit: u64,
}

impl Default for AmdGpuMetric {
    fn default() -> Self {
        Self {
            id: String::new(),
            clocks: HashMap::with_capacity(10),
            energy: 0.0,
            memory_usage_gtt: 0,
            memory_usage_vram: 0,
            power_average: 0,
            temperatures: HashMap::with_capacity(8),
            process_counter: 0,
            process_pid: 0,
            process_usage_compute_unit: 0,
            process_usage_vram: 0,
        }
    }
}

/// Retrieve usefull data metrics on GPUs AMD based models.
///
/// # Return
///
/// - A vector of `AmdGpuMetric`, to store data concerning each AMD GPU installed on a machine.
/// - An error from the Alumet pipeline if a critic metric is not found.
pub fn gather_metric() -> Result<Vec<AmdGpuMetric>, PollError> {
    let mut metrics = Vec::new();

    // Get socket handles
    if let Err(_) = amdsmi_get_socket_handles()
        .map_err(|e| AmdError(e))
        .context("Failed to get socket handles")
    {
        // Shut down AMD SMI if no socket handles exists
        amdsmi_shut_down()
            .map_err(|e| AmdError(e))
            .context("Failed to shut down AMD SMI")?;

        return Ok(metrics);
    } else {
        for socket_handle in amdsmi_get_socket_handles().unwrap() {
            // Get processor handles for each socket handle
            let processor_handles = amdsmi_get_processor_handles(socket_handle)
                .map_err(|e| AmdError(e))
                .context(format!("Failed to get processor handles for socket {socket_handle:?}"))?;

            for processor_handle in processor_handles {
                let mut metric = AmdGpuMetric::default();

                // Get GPU device identification with Bus Device Function using the processor handle
                metric.id = amdsmi_get_gpu_device_bdf(processor_handle)
                    .map_err(|e| AmdError(e))
                    .context("Failed to get GPU compute process information by PID")?
                    .to_string();

                // Get GPU energy consumption in Joules
                let (energy_accumulator, counter_resolution, _timestamp) = amdsmi_get_energy_count(processor_handle)
                    .map_err(|e| AmdError(e))
                    .context("Failed to get energy count")
                    .retry_poll()?;
                metric.energy = (energy_accumulator as f64 * counter_resolution as f64) / 1e6;

                // Get average and current power consumption GPU in Watts
                let power = amdsmi_get_power_info(processor_handle)
                    .map_err(|e| AmdError(e))
                    .context("Failed to get average power")
                    .retry_poll()?;
                metric.power_average = power.average_socket_power as u64;

                // Get GPU Graphic Translation Table memory (GTT) usage in MB
                let memory_gtt = amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt)
                    .map_err(|e| AmdError(e))
                    .context("Failed to get GPU GTT memory usage")
                    .retry_poll()?;
                metric.memory_usage_gtt = memory_gtt / 1_000_000;

                // Get GPU video computing memory (VRAM) usage in MB
                let memory_vram = amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram)
                    .map_err(|e| AmdError(e))
                    .context("Failed to get GPU VRAM memory usage")
                    .retry_poll()?;
                metric.memory_usage_vram = memory_vram / 1_000_000;

                // Get GPU current clock frequencies metric by hardware sectors
                for (clk, area) in &CLK_TYPE {
                    match amdsmi_get_clock_info(processor_handle, *clk) {
                        Ok(data) => {
                            metric.clocks.insert(area.to_string(), data.clk as u64);
                        }
                        Err(e) => warn!("Failed to get GPU frequency for {clk:?} clock type : {e}"),
                    }
                }

                // Get GPU current temperatures metric by hardware sectors in Celsius
                for (sensor, area) in &SENSOR_TYPE {
                    match amdsmi_get_temp_metric(processor_handle, *sensor, AmdsmiTemperatureMetricT::AmdsmiTempCurrent)
                    {
                        Ok(data) => {
                            metric.temperatures.insert(area.to_string(), data as u64);
                        }
                        Err(e) => warn!("Failed to get GPU temperature for {sensor:?} sensor type: {e}"),
                    }
                }

                // Get the GPU compute process information
                let (procs, item) = amdsmi_get_gpu_compute_process_info()
                    .map_err(|e| AmdError(e))
                    .context("Failed to get GPU compute process information")
                    .retry_poll()?;
                metric.process_counter = item as u64;

                // Retrieve compute process metrics if at least one process existing and is running
                if item > 0 {
                    for proc in procs {
                        metric.process_pid = proc.process_id;

                        // Get the process information by PID
                        match amdsmi_get_gpu_compute_process_info_by_pid(proc.process_id) {
                            Ok(data) => {
                                metric.process_usage_vram = data.vram_usage / 1_000_000;
                                metric.process_usage_compute_unit = data.cu_occupancy as u64;
                            }
                            Err(e) => error!("Failed to get GPU compute process information by PID: {e}"),
                        }

                        // Retrieve the GPU indices for the process
                        match amdsmi_get_gpu_compute_process_gpus(proc.process_id) {
                            Ok(gpu_indices) => {
                                for index in gpu_indices {
                                    println!("GPU Index: {}", index);
                                }
                            }
                            Err(e) => error!("Failed to get GPU compute process devices: {e}"),
                        }
                    }
                }
                metrics.push(metric);
            }
        }
    }
    Ok(metrics)
}
