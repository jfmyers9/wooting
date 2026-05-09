pub mod command_pulse;
pub mod external;
pub mod focus;
pub mod github;
pub mod market;
pub mod sports;
pub mod static_effect;

use crate::effects::EffectKind;
use crate::render::{Frame, RenderContext};
use clap::ValueEnum;
pub use command_pulse::{CommandPulseConfig, CommandPulseOutput, CommandPulseSignal};
pub use focus::{FocusConfig, FocusSignal};
pub use github::{GitHubCiConfig, GitHubCiSignal};
pub use market::{MarketConfig, MarketSignal};
use serde::Deserialize;
pub use sports::{SportsConfig, SportsSignal};
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
    #[serde(rename = "github-ci", alias = "git-hub-ci")]
    #[value(name = "github-ci", alias = "git-hub-ci")]
    GitHubCi,
    FocusCockpit,
    MarketPulse,
    SportsAlerts,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SignalConfig {
    pub kind: SignalKind,
    pub effect: Option<EffectKind>,
    #[serde(flatten)]
    pub command_pulse: CommandPulseConfig,
    #[serde(flatten)]
    pub github_ci: GitHubCiConfig,
    #[serde(flatten)]
    pub focus: FocusConfig,
    #[serde(flatten)]
    pub market: MarketConfig,
    #[serde(flatten)]
    pub sports: SportsConfig,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            kind: SignalKind::StaticEffect,
            effect: Some(EffectKind::default()),
            command_pulse: CommandPulseConfig::default(),
            github_ci: GitHubCiConfig::default(),
            focus: FocusConfig::default(),
            market: MarketConfig::default(),
            sports: SportsConfig::default(),
        }
    }
}

impl SignalConfig {
    pub fn static_effect(effect: EffectKind) -> Self {
        Self {
            kind: SignalKind::StaticEffect,
            effect: Some(effect),
            command_pulse: CommandPulseConfig::default(),
            github_ci: GitHubCiConfig::default(),
            focus: FocusConfig::default(),
            market: MarketConfig::default(),
            sports: SportsConfig::default(),
        }
    }

    pub fn command_pulse(command_pulse: CommandPulseConfig) -> Self {
        Self {
            kind: SignalKind::CommandPulse,
            effect: None,
            command_pulse,
            github_ci: GitHubCiConfig::default(),
            focus: FocusConfig::default(),
            market: MarketConfig::default(),
            sports: SportsConfig::default(),
        }
    }

    pub fn github_ci(github_ci: GitHubCiConfig) -> Self {
        Self {
            kind: SignalKind::GitHubCi,
            effect: None,
            command_pulse: CommandPulseConfig::default(),
            github_ci,
            focus: FocusConfig::default(),
            market: MarketConfig::default(),
            sports: SportsConfig::default(),
        }
    }

    pub fn focus_cockpit(focus: FocusConfig) -> Self {
        Self {
            kind: SignalKind::FocusCockpit,
            effect: None,
            command_pulse: CommandPulseConfig::default(),
            github_ci: GitHubCiConfig::default(),
            focus,
            market: MarketConfig::default(),
            sports: SportsConfig::default(),
        }
    }

    pub fn market_pulse(market: MarketConfig) -> Self {
        Self {
            kind: SignalKind::MarketPulse,
            effect: None,
            command_pulse: CommandPulseConfig::default(),
            github_ci: GitHubCiConfig::default(),
            focus: FocusConfig::default(),
            market,
            sports: SportsConfig::default(),
        }
    }

    pub fn sports_alerts(sports: SportsConfig) -> Self {
        Self {
            kind: SignalKind::SportsAlerts,
            effect: None,
            command_pulse: CommandPulseConfig::default(),
            github_ci: GitHubCiConfig::default(),
            focus: FocusConfig::default(),
            market: MarketConfig::default(),
            sports,
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
        SignalKind::GitHubCi => Ok(Box::new(GitHubCiSignal::new(config.github_ci.clone())?)),
        SignalKind::FocusCockpit => Ok(Box::new(FocusSignal::new(config.focus.clone()))),
        SignalKind::MarketPulse => Ok(Box::new(MarketSignal::new(config.market.clone()))),
        SignalKind::SportsAlerts => Ok(Box::new(SportsSignal::new(config.sports.clone()))),
    }
}
