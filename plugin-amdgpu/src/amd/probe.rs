use alumet::{
    measurement::{MeasurementAccumulator, MeasurementPoint, Timestamp},
    metrics::TypedMetricId,
    pipeline::{elements::error::PollError, Source},
    resources::{Resource, ResourceConsumer},
};
use anyhow::Result;

use crate::amd::utils::create_metric;

pub struct Probe {
    /// Metric type based on GPU energy consumption data.
    pub energy: TypedMetricId<f64>,
    /// Metric type based on GPU electric power consumption data.
    pub power: TypedMetricId<u64>,
    /// Metric type based on GPU used RAM memory data.
    pub vram_used: TypedMetricId<u64>,
    /// Metric type based on GPU used GTT memory data.
    pub gtt_used: TypedMetricId<u64>,
}

impl Source for Probe {
    fn poll(&mut self, measurement: &mut MeasurementAccumulator, timestamp: Timestamp) -> Result<(), PollError> {
        let consumer = ResourceConsumer::LocalMachine;
        let metric = create_metric();

        // GPU energy consumption metric pushed
        let ergy = MeasurementPoint::new(
            timestamp,
            self.energy,
            Resource::LocalMachine,
            consumer,
            metric.energy,
        );
        measurement.push(ergy);

        // GPU electric power consumption metric pushed
        let pwr: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.power,
            Resource::LocalMachine,
            consumer,
            metric.power,
        );
        measurement.push(pwr);

        // GPU used RAM memory metric pushed
        let mem_tot: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.vram_used,
            Resource::LocalMachine,
            consumer,
            metric.vram_used,
        );
        measurement.push(mem_tot);

        // GPU used GTT memory metric pushed
        let mem_use: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.gtt_used,
            Resource::LocalMachine,
            consumer,
            metric.gtt_used,
        );
        measurement.push(mem_use);

        Ok(())
    }
}
