use amdsmi::*;
use log::{error, info, warn};

pub struct Metric {
    /// GPU energy consumption.
    pub energy: Option<f64>,
    /// GPU electric power consumption.
    pub power: Option<u64>,
    /// GPU used RAM memory.
    pub vram_used: Option<u64>,
    /// GPU used GTT memory.
    pub gtt_used: Option<u64>,
}

/// Shutdown the AMD SMI library
fn amdsmi_exit() {
    match amdsmi_shut_down() {
        Ok(_) => info!("AMD SMI shut down successfully"),
        Err(e) => panic!("Failed to shut down AMD SMI: {e}"),
    }
}

fn create_metric() -> Metric {
    // Initialize the AMD SMI library
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => info!("AMD SMI initialized successfully"),
        Err(e) => panic!("Failed to initialize AMD SMI: {e}"),
    }

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            panic!("Failed to get socket handles: {e}");
            amdsmi_exit();
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
                    None;
                }
            };

            // Get GPU energy consumption in Joules
            let energy = match amdsmi_get_energy_count(processor_handle) {
                Ok((energy_accumulator, counter_resolution, _)) => (energy_accumulator * counter_resolution) / 1e3,
                Err(e) => {
                    error!("Failed to get energy count: {e}");
                    None;
                }
            };

            // Get average power consumption GPU in Watts
            let power = match amdsmi_get_power_info(processor_handle, sensor_ind) {
                Ok(data) => data.average_socket_power as u64,
                Err(e) => {
                    error!("Failed to get power information: {e}");
                    None;
                }
            };
            match amdsmi_get_power_cap_info(processor_handle, 0) {
                Ok(data) => {
                    let power_test = data.power_cap;
                }
                Err(e) => panic!("Failed to get power cap information: {e}"),
            };

            // Get GPU VRAM memory usage in MB
            let vram_usage: u64 = match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram) {
                Ok(data) => (data / 1e6),
                Err(e) => {
                    error!("Failed to get GPU memory usage: {e}");
                    None;
                }
            };
            // Get GPU GTT memory usage in MB
            let gtt_usage: u64 = match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt) {
                Ok(data) => (data / 1e6),
                Err(e) => {
                    error!("Failed to get GPU memory usage: {e}");
                    None;
                }
            };

            // Get GPU current temperature metric by hardware sectors
            let sensor_type = AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeEdge;
            let metric = AmdsmiTemperatureMetricT::AmdsmiTempCurrent;
            match amdsmi_get_temp_metric(processor_handle, sensor_type, metric) {
                Ok(data) => {
                    let temperature = data;
                }
                Err(e) => error!("Failed to get GPU temperature metric: {e}"),
            }
        }
    }

    amdsmi_exit();

    Metric {
        energy,
        power,
        vram_used: vram_usage,
        gtt_used: gtt_usage,
    }
}
