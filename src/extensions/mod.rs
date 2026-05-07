pub mod command_pulse;
pub mod static_effect;

use crate::effects::EffectKind;
use crate::render::{Frame, RenderContext};
use clap::ValueEnum;
pub use command_pulse::{CommandPulseConfig, CommandPulseExtension};
use serde::Deserialize;
pub use static_effect::StaticEffectExtension;
use std::sync::atomic::AtomicBool;

pub trait KeyboardExtension {
    fn tick(&mut self, interrupted: &AtomicBool);
    fn render(&self, ctx: &RenderContext<'_>) -> Frame;
    fn finished(&self) -> bool;
    fn shutdown(&mut self, interrupted: bool);
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum ExtensionKind {
    #[default]
    StaticEffect,
    CommandPulse,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ExtensionConfig {
    pub kind: ExtensionKind,
    pub effect: Option<EffectKind>,
    #[serde(flatten)]
    pub command_pulse: CommandPulseConfig,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            kind: ExtensionKind::StaticEffect,
            effect: Some(EffectKind::default()),
            command_pulse: CommandPulseConfig::default(),
        }
    }
}

impl ExtensionConfig {
    pub fn static_effect(effect: EffectKind) -> Self {
        Self {
            kind: ExtensionKind::StaticEffect,
            effect: Some(effect),
            command_pulse: CommandPulseConfig::default(),
        }
    }

    pub fn command_pulse(command_pulse: CommandPulseConfig) -> Self {
        Self {
            kind: ExtensionKind::CommandPulse,
            effect: None,
            command_pulse,
        }
    }
}

pub fn build_extension(
    config: &ExtensionConfig,
    fallback_effect: EffectKind,
) -> Result<Box<dyn KeyboardExtension>, Box<dyn std::error::Error>> {
    match config.kind {
        ExtensionKind::StaticEffect => Ok(Box::new(StaticEffectExtension::new(
            config.effect.unwrap_or(fallback_effect),
        ))),
        ExtensionKind::CommandPulse => Ok(Box::new(CommandPulseExtension::new(
            config.command_pulse.clone(),
        )?)),
    }
}
