pub mod command_pulse;
pub mod static_effect;

use crate::effects::EffectKind;
use crate::render::{Frame, RenderContext};
use clap::ValueEnum;
pub use command_pulse::{CommandPulseConfig, CommandPulseSignal};
use serde::Deserialize;
pub use static_effect::StaticEffectSignal;
use std::sync::atomic::AtomicBool;

pub trait SignalProgram {
    fn tick(&mut self, interrupted: &AtomicBool);
    fn render(&self, ctx: &RenderContext<'_>) -> Frame;
    fn finished(&self) -> bool;
    fn shutdown(&mut self, interrupted: bool);
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum SignalKind {
    #[default]
    StaticEffect,
    CommandPulse,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SignalConfig {
    pub kind: SignalKind,
    pub effect: Option<EffectKind>,
    #[serde(flatten)]
    pub command_pulse: CommandPulseConfig,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            kind: SignalKind::StaticEffect,
            effect: Some(EffectKind::default()),
            command_pulse: CommandPulseConfig::default(),
        }
    }
}

impl SignalConfig {
    pub fn static_effect(effect: EffectKind) -> Self {
        Self {
            kind: SignalKind::StaticEffect,
            effect: Some(effect),
            command_pulse: CommandPulseConfig::default(),
        }
    }

    pub fn command_pulse(command_pulse: CommandPulseConfig) -> Self {
        Self {
            kind: SignalKind::CommandPulse,
            effect: None,
            command_pulse,
        }
    }
}

pub fn build_signal(
    config: &SignalConfig,
    fallback_effect: EffectKind,
) -> Result<Box<dyn SignalProgram>, Box<dyn std::error::Error>> {
    match config.kind {
        SignalKind::StaticEffect => Ok(Box::new(StaticEffectSignal::new(
            config.effect.unwrap_or(fallback_effect),
        ))),
        SignalKind::CommandPulse => Ok(Box::new(CommandPulseSignal::new(
            config.command_pulse.clone(),
        )?)),
    }
}
