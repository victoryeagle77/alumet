use amdsmi::*;
use log::{error, info};

pub struct Metric {
    /// GPU energy consumption.
    pub energy: f64,
    /// GPU electric power consumption.
    pub power: u64,
    /// GPU used RAM memory.
    pub vram_used: u64,
    /// GPU used GTT memory.
    pub gtt_used: u64,
}

/// Shutdown the AMD SMI library.
fn exit_amdsmi() {
    match amdsmi_shut_down() {
        Ok(_) => info!("AMD SMI shut down successfully"),
        Err(e) => error!("Failed to shut down AMD SMI: {e}"),
    }
}

pub fn create_metric() -> Metric {
    // Initialize the AMD SMI library
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => info!("AMD SMI initialized successfully"),
        Err(e) => {
            error!("Failed to initialize AMD SMI: {e}");
            return;
        }
    }

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            error!("Failed to get socket handles: {e}");
            exit_amdsmi();
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
            let id = match amdsmi_get_gpu_id(processor_handle) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to get GPU ID: {e}");
                    return 0;
                }
            };

            // Get GPU energy consumption in Joules
            let energy = match amdsmi_get_energy_count(processor_handle) {
                Ok((energy_accumulator, counter_resolution, _)) => {
                    (energy_accumulator as f64 * counter_resolution as f64) / 1e3
                }
                Err(e) => {
                    error!("Failed to get energy count: {e}");
                    return 0.0;
                }
            };

            // Get average power consumption GPU in Watts
            let power = match amdsmi_get_power_info(processor_handle) {
                Ok(data) => data.average_socket_power as u64,
                Err(e) => {
                    error!("Failed to get power information: {e}");
                    return 0;
                }
            };
            let power_test = match amdsmi_get_power_cap_info(processor_handle, 0) {
                Ok(data) => data.power_cap,
                Err(e) => {
                    error!("Failed to get power cap information: {e}");
                    return 0;
                }
            };

            // Get GPU VRAM memory usage in MB
            let vram_used = match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram) {
                Ok(data) => data / 1_000_000,
                Err(e) => {
                    error!("Failed to get GPU memory usage: {e}");
                    return 0;
                }
            };
            // Get GPU GTT memory usage in MB
            let gtt_used = match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt) {
                Ok(data) => data / 1_000_000,
                Err(e) => {
                    error!("Failed to get GPU memory usage: {e}");
                    return 0;
                }
            };

            // Get GPU current temperature metric by hardware sectors
            let sensor_type = [
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeEdge,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHotspot,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeVram,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm0,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm1,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm2,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm3,
                AmdsmiTemperatureTypeT::AmdsmiTemperatureTypePlx,
            ];
            let metric = AmdsmiTemperatureMetricT::AmdsmiTempCurrent;
            let temperature = match amdsmi_get_temp_metric(processor_handle, sensor_type[0], metric) {
                Ok(data) => data as u64,
                Err(e) => {
                    error!("Failed to get GPU temperature metric: {e}");
                    return 0;
                }
            };

            // Get GPU current clock metric by hardware sectors
            let clk_type = [
                AmdsmiClkTypeT::AmdsmiClkTypeSys,
                AmdsmiClkTypeT::AmdsmiClkTypeDf,
                AmdsmiClkTypeT::AmdsmiClkTypeDcef,
                AmdsmiClkTypeT::AmdsmiClkTypeSoc,
                AmdsmiClkTypeT::AmdsmiClkTypeMem,
                AmdsmiClkTypeT::AmdsmiClkTypePcie,
                AmdsmiClkTypeT::AmdsmiClkTypeVclk0,
                AmdsmiClkTypeT::AmdsmiClkTypeVclk1,
                AmdsmiClkTypeT::AmdsmiClkTypeDclk0,
                AmdsmiClkTypeT::AmdsmiClkTypeDclk1,
            ];
            let clk = match amdsmi_get_clk_freq(processor_handle, clk_type[0]) {
                Ok(data) => data.current,
                Err(e) => {
                    error!("Failed to get clock frequencies: {e}");
                    return 0;
                }
            };
        }
    }

    exit_amdsmi();

    Metric {
        energy,
        power,
        vram_used,
        gtt_used,
    }
}
