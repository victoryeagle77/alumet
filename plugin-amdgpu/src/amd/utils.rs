use amdsmi::*;
use log::error;

pub struct Metric {
    /// GPU clock frequency in Mhz.
    pub clock: u64,
    /// GPU energy consumption in J.
    pub energy: f64,
    /// GPU electric power consumption in W.
    pub power: u64,
    /// GPU temperature in °C.
    pub temperature: u64,
    /// GPU used RAM memory in MB.
    pub vram_used: u64,
    /// GPU used GTT memory in MB.
    pub gtt_used: u64,
}

pub fn create_metric() -> Metric {
    let mut metric = Metric {
        clock: 0,
        energy: 0.0,
        power: 0,
        temperature: 0,
        vram_used: 0,
        gtt_used: 0,
    };

    // Initialize the AMD SMI library
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => (),
        Err(e) => {
            panic!("Failed to initialize AMD SMI: {e}");
            return metric;
        }
    }

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            error!("Failed to get socket handles: {e}");
            match amdsmi_shut_down() {
                Ok(_) => (),
                Err(e) => error!("Failed to shut down AMD SMI: {e}"),
            }
            return metric;
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

                // Get GPU current temperature metric by hardware sectors in Celsius
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
                if let Ok(data) = amdsmi_get_temp_metric(
                    processor_handle,
                    sensor_type[0],
                    AmdsmiTemperatureMetricT::AmdsmiTempCurrent,
                ) {
                    metric.temperature = data as u64;
                } else {
                    error!("Failed to get GPU VRAM memory usage");
                }

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
                if let Ok(data) = amdsmi_get_clock_info(processor_handle, clk_type[0]) {
                    metric.clock = data.clk as u64;
                } else {
                    error!("Failed to get GPU VRAM memory usage");
                }

            } else {
                error!("Failed to get GPU ID");
            }
        }
    }

    match amdsmi_shut_down() {
        Ok(_) => (),
        Err(e) => error!("Failed to shut down AMD SMI: {e}"),
    };

    metric
}
