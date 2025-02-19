use alumet::{
    measurement::{MeasurementAccumulator, MeasurementPoint, Timestamp},
    metrics::TypedMetricId,
    pipeline::{elements::error::PollError, Source},
    resources::{Resource, ResourceConsumer},
};
use anyhow::Result;

use crate::amd::utils::gather_metric;

pub struct Probe {
    /// Metric type based on GPU clock frequency data.
    pub clock: TypedMetricId<u64>,
    /// Metric type based on GPU energy consumption data.
    pub energy: TypedMetricId<f64>,
    /// Metric type based on GPU used GTT memory data.
    pub memory_usage_gtt: TypedMetricId<u64>,
    /// Metric type based on GPU used VRAM memory data.
    pub memory_usage_vram: TypedMetricId<u64>,
    /// Metric type based on GPU electric power consumption data.
    pub power_average: TypedMetricId<u64>,
    /// Metric type based on GPU temperature data.
    pub temperature: TypedMetricId<u64>,
    /// Metric type base on GPU process counter data.
    pub process_counter: TypedMetricId<u64>,
    ///  Metric type based on GPU process compute unit usage data.
    pub process_usage_compute_unit: TypedMetricId<u64>,
    /// Metric type based on GPU process VRAM memory usage data.
    pub process_usage_vram: TypedMetricId<u64>,
}

impl Source for Probe {
    fn poll(&mut self, measurement: &mut MeasurementAccumulator, timestamp: Timestamp) -> Result<(), PollError> {
        for metric in gather_metric()? {
            let id = metric.id;

            // GPU clock frequencies metrics pushed
            for (area, clock) in &metric.clocks {
                measurement.push(
                    MeasurementPoint::new(
                        timestamp,
                        self.clock,
                        Resource::Gpu {
                            bus_id: id.clone().into(),
                        },
                        ResourceConsumer::LocalMachine,
                        *clock,
                    )
                    .with_attr("clock_type", area.to_string()),
                );
            }

            // GPU energy consumption metric pushed
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.energy,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                metric.energy,
            ));

            // GPU electric average power consumption metric pushed
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.power_average,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                metric.power_average,
            ));

            // GPU temperatures metrics pushed
            for (area, temperature) in &metric.temperatures {
                measurement.push(
                    MeasurementPoint::new(
                        timestamp,
                        self.temperature,
                        Resource::Gpu {
                            bus_id: id.clone().into(),
                        },
                        ResourceConsumer::LocalMachine,
                        *temperature,
                    )
                    .with_attr("thermal_zone", area.to_string()),
                );
            }

            // GPU GTT memory used metrics pushed
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.memory_usage_gtt,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                metric.memory_usage_gtt,
            ));

            // GPU VRAM memory used metrics pushed
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.memory_usage_vram,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                metric.memory_usage_vram,
            ));

            // GPU compute processes counter metric pushed
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.process_counter,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                metric.process_counter,
            ));

            // Push compute process metrics if at least one process existing and is running
            if metric.process_counter > 0 {
                let pid = metric.process_pid;
                let consumer = ResourceConsumer::Process { pid };

                // GPU compute processes compute unit usage pushed
                measurement.push(MeasurementPoint::new(
                    timestamp,
                    self.process_usage_compute_unit,
                    Resource::Gpu {
                        bus_id: id.clone().into(),
                    },
                    consumer.clone(),
                    metric.process_usage_compute_unit,
                ));

                // GPU compute processes VRAM memory usage pushed
                measurement.push(MeasurementPoint::new(
                    timestamp,
                    self.process_usage_vram,
                    Resource::Gpu {
                        bus_id: id.clone().into(),
                    },
                    consumer.clone(),
                    metric.process_usage_vram,
                ));
            }
        }

        Ok(())
    }
}
