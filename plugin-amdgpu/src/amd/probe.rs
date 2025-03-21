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
    pub power: TypedMetricId<u64>,
    /// Metric type based on GPU temperature data.
    pub temperature: TypedMetricId<u64>,
    /// Metric type based on GPU used RAM memory data.
    pub vram_used: TypedMetricId<u64>,
    /// Metric type based on GPU used GTT memory data.
    pub gtt_used: TypedMetricId<u64>,
}

impl Source for Probe {
    fn poll(&mut self, measurement: &mut MeasurementAccumulator, timestamp: Timestamp) -> Result<(), PollError> {
        let metric = create_metric();

        // GPU clock frequency metric pushed
        let clk = MeasurementPoint::new(
            timestamp,
            self.clock,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.clock,
        );
        measurement.push(clk);

        // GPU energy consumption metric pushed
        let ergy = MeasurementPoint::new(
            timestamp,
            self.energy,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.energy,
        );
        measurement.push(ergy);

        // GPU electric power consumption metric pushed
        let pwr = MeasurementPoint::new(
            timestamp,
            self.power,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.power,
        );
        measurement.push(pwr);

        // GPU temperature metric pushed
        let tmp = MeasurementPoint::new(
            timestamp,
            self.temperature,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.temperature,
        );
        measurement.push(tmp);

        // GPU used RAM memory metric pushed
        let vram_use = MeasurementPoint::new(
            timestamp,
            self.vram_used,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.vram_used,
        );
        measurement.push(vram_use);

        // GPU used GTT memory metric pushed
        let gtt_use = MeasurementPoint::new(
            timestamp,
            self.gtt_used,
            Resource::Gpu { bus_id: "0".into() },
            ResourceConsumer::LocalMachine,
            metric.gtt_used,
        );
        measurement.push(gtt_use);

        Ok(())
    }
}
