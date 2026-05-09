use crate::effects::EffectKind;
use crate::render::PaletteName;
use crate::runner::{RunOptions, SignalRunOptions};
use crate::signals::SignalConfig;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub sdk_path: Option<PathBuf>,
    pub effect: EffectKind,
    pub palette: PaletteName,
    pub brightness: u8,
    pub fps: u32,
    pub seconds: Option<u64>,
    pub continuous: bool,
    pub warn_on_close_error: bool,
    #[serde(alias = "extension")]
    pub signal: Option<SignalConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let run = RunOptions::default();
        Self {
            sdk_path: None,
            effect: run.effect,
            palette: run.palette,
            brightness: run.brightness,
            fps: run.fps,
            seconds: run.seconds,
            continuous: false,
            warn_on_close_error: true,
            signal: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&content).map_err(ConfigError::Parse)
    }

    pub fn signal_run_options(&self) -> SignalRunOptions {
        SignalRunOptions {
            palette: self.palette,
            brightness: self.brightness,
            fps: self.fps,
            seconds: self.seconds,
            continuous: self.continuous,
        }
    }

    pub fn signal_config(&self) -> SignalConfig {
        self.signal
            .clone()
            .unwrap_or_else(|| SignalConfig::static_effect(self.effect))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::SignalKind;

    #[test]
    fn config_defaults_to_static_effect_signal() {
        let config = AppConfig::default();

        assert_eq!(config.brightness, 96);
        assert_eq!(config.fps, 30);
        assert_eq!(config.seconds, Some(10));
        assert!(!config.continuous);
        assert!(config.warn_on_close_error);
        assert_eq!(config.signal_config().kind, SignalKind::StaticEffect);
    }

    #[test]
    fn signal_run_options_reflect_config_values() {
        let config: AppConfig = toml::from_str(
            r#"
palette = "terminal"
brightness = 42
fps = 12
seconds = 0
continuous = true
"#,
        )
        .unwrap();

        let options = config.signal_run_options();
        assert_eq!(options.palette, PaletteName::Terminal);
        assert_eq!(options.brightness, 42);
        assert_eq!(options.fps, 12);
        assert_eq!(options.seconds, Some(0));
        assert!(options.continuous);
    }

    #[test]
    fn config_accepts_legacy_extension_alias() {
        let config: AppConfig = toml::from_str(
            r#"
[extension]
kind = "command-pulse"
command = ["true"]
"#,
        )
        .unwrap();

        let signal = config.signal_config();
        assert_eq!(signal.kind, SignalKind::CommandPulse);
        assert_eq!(signal.command_pulse.command, ["true"]);
    }

    #[test]
    fn config_rejects_unknown_top_level_fields() {
        let error = toml::from_str::<AppConfig>("unexpected = true").unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }
}
