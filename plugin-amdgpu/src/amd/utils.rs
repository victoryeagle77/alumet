use amdsmi::*;
use log::{error, info, warn};

pub struct Metric {
    pub id: String,
    /// GPU clock frequencies in Mhz.
    pub clocks: Vec<u64>,
    /// GPU energy consumption in J.
    pub energy: f64,
    /// GPU electric power consumption in W.
    pub power_average: u64,
    /// GPU temperature in °C.
    pub temperatures: Vec<u64>,
    /// GPU Video computing memory (VRAM) usage in MB.
    pub memory_gtt_usage: u64,
    /// GPU Graphic Translation Table memory (GTT) usage in MB.
    pub memory_vram_usage: u64,
    /// GPU compute processes counter.
    pub counter_compute_process: u64,
}

pub fn create_metric() -> Metric {
    let mut metric = Metric {
        id: String::new(),
        clocks: Vec::new(),
        energy: 0.0,
        power_average: 0,
        temperatures: Vec::new(),
        memory_gtt_usage: 0,
        memory_vram_usage: 0,
        counter_compute_process: 0,
    };

    // Initialize the AMD SMI library
    match amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus) {
        Ok(_) => info!("AMD SMI initialized successfully"),
        Err(e) => {
            panic!("Failed to initialize AMD SMI: {e}");
        }
    }

    // Get the GPU compute process information
    match amdsmi_get_gpu_compute_process_info() {
        Ok((procs, item)) => {
            metric.counter_compute_process = item as u64;
            for proc in procs {
                println!("Proc info process_id {:?}", proc.process_id);
                println!("Proc info vram_usage {:?}", proc.vram_usage);
                println!("Proc info sdma_usage {:?}", proc.sdma_usage);
                println!("Proc info cu_occupancy {:?}", proc.cu_occupancy);

                // Get the process information by PID
                match amdsmi_get_gpu_compute_process_info_by_pid(proc.process_id) {
                    Ok(data) => println!("{:?}", data),
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
        Err(AmdsmiStatusT::AmdsmiStatusNotSupported) => {
            error!("amdsmi_get_gpu_compute_process_info() not supported on this device")
        }
        Err(e) => error!("Failed to get GPU compute process information: {e}"),
    }

    // Get socket handles
    let socket_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => handles,
        Err(e) => {
            error!("Failed to get socket handles: {e}");
            match amdsmi_shut_down() {
                Ok(_) => info!("AMD SMI shut down successfully"),
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
            // Get GPU device identification with Bus Device Function using the processor handle
            if let Ok(data) = amdsmi_get_gpu_device_bdf(processor_handle) {
                metric.id = data.to_string();

                // Verifying if the power management is enable (useful for consumption analysis)
                match amdsmi_is_gpu_power_management_enabled(processor_handle) {
                    Ok(enabled) => info!("GPU power management enabled: {enabled}"),
                    Err(e) => error!("Failed to check GPU power management status: {e}"),
                }

                // Get GPU energy consumption in Joules
                match amdsmi_get_energy_count(processor_handle) {
                    Ok((energy_accumulator, counter_resolution, _timestamp)) => {
                        metric.energy = (energy_accumulator as f64 * counter_resolution as f64) / 1e6
                    }
                    Err(e) => error!("Failed to get energy count: {e}"),
                }

                // Get average and current power consumption GPU in Watts
                match amdsmi_get_power_info(processor_handle) {
                    Ok(data) => {
                        metric.power_average = data.average_socket_power as u64;
                    }
                    Err(e) => error!("Failed to get power count: {e}"),
                }

                // Get GPU video computing memory (VRAM) usage in MB
                match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram) {
                    Ok(data) => {
                        metric.memory_vram_usage = data / 1_000_000;
                    }
                    Err(e) => error!("Failed to get GPU VRAM memory usage: {e}"),
                }
                // Get GPU Graphic Translation Table memory (GTT) usage in MB
                match amdsmi_get_gpu_memory_usage(processor_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeGtt) {
                    Ok(data) => {
                        metric.memory_gtt_usage = data / 1_000_000;
                    }
                    Err(e) => error!("Failed to get GPU GTT memory usage: {e}"),
                }

                // Get GPU current temperatures metric by hardware sectors in Celsius
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
                for &sensor in &sensor_type {
                    match amdsmi_get_temp_metric(processor_handle, sensor, AmdsmiTemperatureMetricT::AmdsmiTempCurrent)
                    {
                        Ok(data) => {
                            metric.temperatures.push(data as u64);
                        }
                        Err(e) => warn!("Failed to get GPU temperature for {sensor:?} sensor type: {e}"),
                    }
                }

                // Get GPU current clock frequencies metric by hardware sectors
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
                for &clk in &clk_type {
                    match amdsmi_get_clock_info(processor_handle, clk) {
                        Ok(data) => {
                            metric.clocks.push(data.clk as u64);
                        }
                        Err(e) => warn!("Failed to get GPU frequency for {clk:?} clock type : {e}"),
                    }
                }

                // Retrieve the list of GPU processes
                match amdsmi_get_gpu_process_list(processor_handle) {
                    Ok(process_list) => {
                        for process in process_list {
                            println!(">>>>> Process Info pid : {:?}", process.pid);
                            println!(">>>>> Process Info mem : {:?}", process.mem);
                            println!(">>>>> Process Info memory_usage : {:?}", process.memory_usage);
                            println!(">>>>> Process Info engine_usage : {:?}", process.engine_usage);
                        }
                    }
                    Err(e) => error!("Failed to get GPU process list: {e}"),
                }
            } else {
                error!("Failed to get GPU device BDF identification");
            }
        }
    }

    match amdsmi_shut_down() {
        Ok(_) => info!("AMD SMI shut down successfully"),
        Err(e) => error!("Failed to shut down AMD SMI: {e}"),
    };

    metric
}
