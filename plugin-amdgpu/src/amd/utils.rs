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
    let mut metric = Metric {
        energy: 0.0,
        power: 0,
        vram_used: 0,
        gtt_used: 0,
    };

    // Initialize the AMD SMI library
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => info!("AMD SMI initialized successfully"),
        Err(e) => {
            error!("Failed to initialize AMD SMI: {e}");
            return metric
        }
    }

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            error!("Failed to get socket handles: {e}");
            exit_amdsmi();
            return metric
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
            if let Ok(id) = amdsmi_get_gpu_id(processor_handle) {

                // Get GPU energy consumption in Joules
                if let Ok((energy_accumulator, counter_resolution, _)) = amdsmi_get_energy_count(processor_handle) {
                    metric.energy = (energy_accumulator as f64 * counter_resolution as f64) / 1e3;
                } else {
                    error!("Failed to get energy count");
                }

                // Get average power consumption GPU in Watts
                if let Ok(data) = amdsmi_get_power_info(processor_handle) {
                    metric.power = data.average_socket_power as u64;
                } else {
                    error!("Failed to get power information");
                }

                // if let Ok(data) = amdsmi_get_power_cap_info(processor_handle, 0) {
                //     metric.power = data.power_cap as u64;
                // } else {
                //     error!("Failed to get power information");
                // }

                // Get GPU VRAM memory usage in MB
                if let Ok(data) = amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram) {
                    metric.vram_used = data / 1_000_000;
                } else {
                    error!("Failed to get GPU VRAM memory usage");
                }

                // Get GPU GTT memory usage in MB
                if let Ok(data) = amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt) {
                    metric.gtt_used = data / 1_000_000;
                } else {
                    error!("Failed to get GPU GTT memory usage");
                }

                // Get GPU current temperature metric by hardware sectors
                // let sensor_type = [
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeEdge,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHotspot,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeVram,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm0,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm1,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm2,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHbm3,
                //     AmdsmiTemperatureTypeT::AmdsmiTemperatureTypePlx,
                // ];
                // let metric = AmdsmiTemperatureMetricT::AmdsmiTempCurrent;
                // if let Ok(data) = amdsmi_get_temp_metric(processor_handle, sensor_type[0], metric) {
                //     metric.temperature = data;
                // } else {
                //     error!("Failed to get GPU VRAM memory usage");
                // }

                // Get GPU current clock metric by hardware sectors
                // let clk_type = [
                //     AmdsmiClkTypeT::AmdsmiClkTypeSys,
                //     AmdsmiClkTypeT::AmdsmiClkTypeDf,
                //     AmdsmiClkTypeT::AmdsmiClkTypeDcef,
                //     AmdsmiClkTypeT::AmdsmiClkTypeSoc,
                //     AmdsmiClkTypeT::AmdsmiClkTypeMem,
                //     AmdsmiClkTypeT::AmdsmiClkTypePcie,
                //     AmdsmiClkTypeT::AmdsmiClkTypeVclk0,
                //     AmdsmiClkTypeT::AmdsmiClkTypeVclk1,
                //     AmdsmiClkTypeT::AmdsmiClkTypeDclk0,
                //     AmdsmiClkTypeT::AmdsmiClkTypeDclk1,
                // ];
                // if let Ok(data) = amdsmi_get_clk_freq(processor_handle, clk_type[0]) {
                //     metric.clk = data;
                // } else {
                //     error!("Failed to get GPU VRAM memory usage");
                // }
            } else {
                error!("Failed to get GPU ID");
            }
        }
    }

    exit_amdsmi();
    metric
}
