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
    /// Metric type based on GPU total RAM memory data.
    pub memory_total: TypedMetricId<u64>,
    /// Metric type based on GPU used RAM memory data.
    pub memory_used: TypedMetricId<u64>,
}

impl Source for Probe {
    fn poll(&mut self, measurement: &mut MeasurementAccumulator, timestamp: Timestamp) -> Result<(), PollError> {
        let metric = create_metric();

        // GPU energy consumption metric pushed
        let ergy = MeasurementPoint::new(
            timestamp,
            self.energy,
            Resource::LocalMachine,
            ResourceConsumer::LocalMachine,
            metric.energy,
        );
        measurement.push(ergy);

        // GPU electric power consumption metric pushed
        let pwr: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.power,
            Resource::LocalMachine,
            ResourceConsumer::LocalMachine,
            metric.power,
        );
        measurement.push(pwr);

        // GPU total RAM memory metric pushed
        let mem_tot: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.memory_total,
            Resource::LocalMachine,
            ResourceConsumer::LocalMachine,
            metric.memory_total,
        );
        measurement.push(mem_tot);

        // GPU used RAM memory metric pushed
        let mem_use: MeasurementPoint = MeasurementPoint::new(
            timestamp,
            self.memory_used,
            Resource::LocalMachine,
            ResourceConsumer::LocalMachine,
            metric.memory_used,
        );
        measurement.push(mem_use);

        Ok(())
    }
}
