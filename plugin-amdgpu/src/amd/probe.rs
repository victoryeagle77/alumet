use alumet::{
    measurement::{MeasurementAccumulator, MeasurementPoint, Timestamp},
    metrics::TypedMetricId,
    pipeline::{elements::error::PollError, Source},
    resources::{Resource, ResourceConsumer},
};
use anyhow::Result;

use crate::amd::utils::create_metric;

pub struct Probe {
    /// Metric type based on GPU clock frequency data.
    pub clock: TypedMetricId<u64>,
    /// Metric type based on GPU energy consumption data.
    pub energy: TypedMetricId<f64>,
    /// Metric type based on GPU electric power consumption data.
    pub power_average: TypedMetricId<u64>,
    /// Metric type based on GPU temperature data.
    pub temperature: TypedMetricId<u64>,
    /// Metric type based on GPU used GTT memory data.
    pub memory_gtt_usage: TypedMetricId<u64>,
    /// Metric type based on GPU used VRAM memory data.
    pub memory_vram_usage: TypedMetricId<u64>,
    /// Metric type base on GPU
    pub count_compute_process: TypedMetricId<u64>,
}

impl Source for Probe {
    fn poll(&mut self, measurement: &mut MeasurementAccumulator, timestamp: Timestamp) -> Result<(), PollError> {
        let metric = create_metric();
        let id = metric.id;

        // GPU clock frequencies metrics pushed
        for (_i, clock) in metric.clocks.iter().enumerate() {
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.clock,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                *clock,
            ));
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
        for (_i, temperature) in metric.temperatures.iter().enumerate() {
            measurement.push(MeasurementPoint::new(
                timestamp,
                self.temperature,
                Resource::Gpu {
                    bus_id: id.clone().into(),
                },
                ResourceConsumer::LocalMachine,
                *temperature,
            ));
        }

        // GPU GTT memory used metrics pushed
        measurement.push(MeasurementPoint::new(
            timestamp,
            self.memory_gtt_usage,
            Resource::Gpu {
                bus_id: id.clone().into(),
            },
            ResourceConsumer::LocalMachine,
            metric.memory_gtt_usage,
        ));

        // GPU VRAM memory used metrics pushed
        measurement.push(MeasurementPoint::new(
            timestamp,
            self.memory_vram_usage,
            Resource::Gpu {
                bus_id: id.clone().into(),
            },
            ResourceConsumer::LocalMachine,
            metric.memory_vram_usage,
        ));

        // GPU compute processes counter metric pushed
        measurement.push(MeasurementPoint::new(
            timestamp,
            self.count_compute_process,
            Resource::LocalMachine,
            ResourceConsumer::LocalMachine,
            metric.counter_compute_process,
        ));

        Ok(())
    }
}
