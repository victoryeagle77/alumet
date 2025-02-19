mod amd;
use amd::probe::Probe;

use serde::{Deserialize, Serialize};
use std::time::Duration;

use alumet::{
    pipeline::trigger,
    plugin::{
        rust::{deserialize_config, serialize_config, AlumetPlugin},
        AlumetPluginStart, ConfigTable,
    },
    units::{PrefixedUnit, Unit},
};

#[derive(Serialize, Deserialize)]
struct Config {
    /// Time between each activation of the counter source.
    #[serde(with = "humantime_serde")]
    poll_interval: Duration,
}

pub struct AMDGPUPlugin {
    config: Config,
}

impl AlumetPlugin for AMDGPUPlugin {
    // Name of plugin, in lowercase, without the "plugin-" prefix
    fn name() -> &'static str {
        "amdgpu"
    }

    // Gets the version from the Cargo.toml of the plugin crate
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn default_config() -> anyhow::Result<Option<ConfigTable>> {
        let config = serialize_config(Config::default())?;
        Ok(Some(config))
    }

    fn init(config: ConfigTable) -> anyhow::Result<Box<Self>> {
        let config = deserialize_config(config)?;
        Ok(Box::new(AMDGPUPlugin { config }))
    }

    fn start(&mut self, alumet: &mut AlumetPluginStart) -> anyhow::Result<()> {
        let mb = PrefixedUnit::mega(Unit::Byte);
        // Create the source
        let source = Probe {
            energy: alumet.create_metric::<f64>(
                "amd_gpu_energy_consumption",
                Unit::Joule,
                "Get GPU energy consumption in Joules",
            )?,
            power: alumet.create_metric::<u64>(
                "amd_gpu_power_consumption",
                Unit::Watt,
                "Get GPU electric power consumption in Watt",
            )?,
            memory_total: alumet.create_metric::<u64>(
                "amd_gpu_total_memory",
                mb.clone(),
                "Get GPU total RAM memory in MB",
            )?,
            memory_used: alumet.create_metric::<u64>(
                "amd_gpu_used_memory",
                mb.clone(),
                "Get GPU used RAM memory in MB",
            )?,
        };

        // Configure how the source is triggered: Alumet will call the source every 1s
        let trigger = trigger::builder::time_interval(self.config.poll_interval).build()?;

        // Add the source to the measurement pipeline
        alumet.add_source(Box::new(source), trigger);

        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    // Create a fake plugin structure for amdgpu plugin
    fn fake_config() -> AMDGPUPlugin {
        AMDGPUPlugin {
            config: Config {
                poll_interval: Duration::from_secs(1),
            },
        }
    }

    // Test `default_config` function of amdgpu plugin
    #[test]
    fn test_default_config() {
        let result = AMDGPUPlugin::default_config().unwrap();
        assert!(result.is_some());

        let config_table = result.unwrap();
        let config: Config = deserialize_config(config_table).expect("ERROR : Failed to deserialize config");

        assert_eq!(config.poll_interval, Duration::from_secs(1));
    }

    // Test `init` function to initialize amdgpu plugin configuration
    #[test]
    fn test_init() -> Result<()> {
        let config_table = serialize_config(Config::default())?;
        let _plugin = AMDGPUPlugin::init(config_table)?;
        Ok(())
    }

    // Test `stop` function to stop amdgpu plugin
    #[test]
    fn test_stop() {
        let mut plugin = fake_config();
        let result = plugin.stop();
        assert!(result.is_ok());
    }
}
