use crate::effects::EffectKind;
use crate::extensions::ExtensionConfig;
use crate::render::PaletteName;
use crate::runner::{ExtensionRunOptions, RunOptions};
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
    pub extension: Option<ExtensionConfig>,
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
            extension: None,
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

    pub fn extension_run_options(&self) -> ExtensionRunOptions {
        ExtensionRunOptions {
            palette: self.palette,
            brightness: self.brightness,
            fps: self.fps,
            seconds: self.seconds,
            continuous: self.continuous,
        }
    }

    pub fn extension_config(&self) -> ExtensionConfig {
        self.extension
            .clone()
            .unwrap_or_else(|| ExtensionConfig::static_effect(self.effect))
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
