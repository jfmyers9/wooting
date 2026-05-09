mod config;
mod effects;
mod layout;
mod render;
mod runner;
mod sdk;
mod signals;

use clap::{Parser, Subcommand};
use config::AppConfig;
use effects::EffectKind;
use layout::KeyboardLayout;
use render::{Color, PaletteName};
use runner::{RunOptions, SignalRunOptions, run_effect, run_signal, sleep_interruptibly};
use sdk::rgb::{DeviceInfo, WootingRgb};
use signals::{CommandPulseConfig, CommandPulseOutput, SignalKind, build_signal};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path to libwooting-rgb-sdk.dylib/.so or wooting-rgb-sdk.dll.
    #[arg(long, global = true)]
    sdk_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print connected keyboard metadata.
    Info,
    /// Print the inferred keyboard layout summary.
    LayoutInfo,
    /// Paint a short row test pattern, then reset.
    Test {
        /// Maximum RGB channel value.
        #[arg(long, default_value_t = 96)]
        brightness: u8,
        /// Seconds to keep the pattern visible.
        #[arg(long, default_value_t = 3)]
        seconds: u64,
    },
    /// Try the SDK direct single-key feature call, then reset.
    Direct {
        /// Matrix row to light.
        #[arg(long, default_value_t = 0)]
        row: u8,
        /// Matrix column to light.
        #[arg(long, default_value_t = 0)]
        column: u8,
        /// Maximum RGB channel value.
        #[arg(long, default_value_t = 96)]
        brightness: u8,
        /// Seconds to keep the key visible.
        #[arg(long, default_value_t = 3)]
        seconds: u64,
    },
    /// Run a device-bounded rainbow animation, then reset.
    Rainbow {
        /// Maximum RGB channel value.
        #[arg(long, default_value_t = 128)]
        brightness: u8,
        /// Seconds to run the animation.
        #[arg(long, default_value_t = 10)]
        seconds: u64,
        /// Animation frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
    },
    /// Run any named RGB demo effect.
    Effect {
        /// Effect to run.
        #[arg(value_enum)]
        effect: EffectKind,
        /// Palette for palette-aware effects.
        #[arg(long, value_enum, default_value_t = PaletteName::Wooting)]
        palette: PaletteName,
        /// Maximum RGB channel value.
        #[arg(long, default_value_t = 128)]
        brightness: u8,
        /// Seconds to run the effect.
        #[arg(long, default_value_t = 10)]
        seconds: u64,
        /// Animation frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
    },
    /// Run a signal directly.
    #[command(alias = "extension")]
    Signal {
        #[command(subcommand)]
        command: SignalCommand,
    },
    /// Run a TOML signals profile.
    Run {
        /// Config file path.
        #[arg(long)]
        config: PathBuf,
        /// Print resolved config without touching the keyboard.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SignalCommand {
    /// Run a named signal. Use `--` before command-pulse commands.
    Run {
        /// Signal to run.
        #[arg(value_enum)]
        signal: SignalKind,
        /// Static effect used by the static-effect signal.
        #[arg(long, value_enum, default_value_t = EffectKind::Comet)]
        effect: EffectKind,
        /// Palette for signal renderers.
        #[arg(long, value_enum, default_value_t = PaletteName::Wooting)]
        palette: PaletteName,
        /// Maximum RGB channel value.
        #[arg(long, default_value_t = 128)]
        brightness: u8,
        /// Animation frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
        /// Seconds to run static-effect.
        #[arg(long, default_value_t = 10)]
        seconds: u64,
        /// Working directory for command-pulse.
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// Environment override for command-pulse, in KEY=VALUE form. Repeatable.
        #[arg(long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,
        /// Command stdout/stderr policy for command-pulse.
        #[arg(long, value_enum, default_value_t = CommandPulseOutput::Inherit)]
        output: CommandPulseOutput,
        /// Print a command-pulse status summary when the command completes.
        #[arg(long)]
        summary: bool,
        /// Command timeout for command-pulse.
        #[arg(long, default_value_t = 600)]
        timeout_seconds: u64,
        /// Command-pulse success hold seconds.
        #[arg(long, default_value_t = 3)]
        success_hold_seconds: u64,
        /// Command-pulse failure/timeout hold seconds.
        #[arg(long, default_value_t = 6)]
        failure_hold_seconds: u64,
        /// Command-pulse interrupted hold seconds.
        #[arg(long, default_value_t = 2)]
        interrupted_hold_seconds: u64,
        /// Command and args for command-pulse.
        #[arg(last = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let interrupted = install_ctrlc_handler()?;

    match cli.command {
        Command::Info => {
            let keyboard = WootingRgb::open(cli.sdk_path.as_deref())?;
            print_info(keyboard.info());
        }
        Command::LayoutInfo => {
            let keyboard = WootingRgb::open(cli.sdk_path.as_deref())?;
            print_info(keyboard.info());
            let layout = KeyboardLayout::for_device(keyboard.info());
            println!("layout: {}", layout.summary());
        }
        Command::Test {
            brightness,
            seconds,
        } => {
            let options = RunOptions {
                effect: EffectKind::RowTest,
                brightness,
                seconds: Some(seconds),
                ..RunOptions::default()
            };
            run_keyboard(cli.sdk_path, &options, &interrupted)?;
        }
        Command::Direct {
            row,
            column,
            brightness,
            seconds,
        } => {
            let mut keyboard = WootingRgb::open(cli.sdk_path.as_deref())?;
            print_info(keyboard.info());
            keyboard.direct_set_key(row, column, Color::new(brightness, brightness, brightness))?;
            sleep_interruptibly(Duration::from_secs(seconds), &interrupted);
            close_best_effort(&mut keyboard, true);
        }
        Command::Rainbow {
            brightness,
            seconds,
            fps,
        } => {
            let options = RunOptions {
                effect: EffectKind::Rainbow,
                brightness,
                seconds: Some(seconds),
                fps,
                ..RunOptions::default()
            };
            run_keyboard(cli.sdk_path, &options, &interrupted)?;
        }
        Command::Effect {
            effect,
            palette,
            brightness,
            seconds,
            fps,
        } => {
            let options = RunOptions {
                effect,
                palette,
                brightness,
                seconds: Some(seconds),
                fps,
                continuous: false,
            };
            run_keyboard(cli.sdk_path, &options, &interrupted)?;
        }
        Command::Signal { command } => match command {
            SignalCommand::Run {
                signal,
                effect,
                palette,
                brightness,
                fps,
                seconds,
                cwd,
                env,
                output,
                summary,
                timeout_seconds,
                success_hold_seconds,
                failure_hold_seconds,
                interrupted_hold_seconds,
                command,
            } => {
                let env = parse_env_vars(env)?;
                let (config, options) = match signal {
                    SignalKind::StaticEffect => (
                        signals::SignalConfig::static_effect(effect),
                        SignalRunOptions {
                            palette,
                            brightness,
                            fps,
                            seconds: Some(seconds),
                            continuous: false,
                        },
                    ),
                    SignalKind::CommandPulse => (
                        signals::SignalConfig::command_pulse(CommandPulseConfig {
                            command,
                            cwd,
                            env,
                            output,
                            summary,
                            timeout_seconds,
                            success_hold_seconds,
                            failure_hold_seconds,
                            interrupted_hold_seconds,
                            ..CommandPulseConfig::default()
                        }),
                        SignalRunOptions {
                            palette,
                            brightness,
                            fps,
                            seconds: None,
                            continuous: true,
                        },
                    ),
                };
                let mut signal = build_signal(&config, effect)?;
                run_keyboard_signal(cli.sdk_path, &options, &mut *signal, true, &interrupted)?;
            }
        },
        Command::Run { config, dry_run } => {
            let config = AppConfig::load(&config)?;
            print_config(&config);
            if !dry_run {
                let sdk_path = cli.sdk_path.or(config.sdk_path.clone());
                let mut signal = build_signal(&config.signal_config(), config.effect)?;
                run_keyboard_signal(
                    sdk_path,
                    &config.signal_run_options(),
                    &mut *signal,
                    config.warn_on_close_error,
                    &interrupted,
                )?;
            }
        }
    }

    Ok(())
}

fn run_keyboard(
    sdk_path: Option<PathBuf>,
    options: &RunOptions,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    run_keyboard_with_close_policy(sdk_path, options, true, interrupted)
}

fn run_keyboard_with_close_policy(
    sdk_path: Option<PathBuf>,
    options: &RunOptions,
    warn_on_close_error: bool,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut keyboard = WootingRgb::open(sdk_path.as_deref())?;
    print_info(keyboard.info());
    run_effect(&keyboard, options, interrupted)?;
    close_best_effort(&mut keyboard, warn_on_close_error);
    Ok(())
}

fn run_keyboard_signal(
    sdk_path: Option<PathBuf>,
    options: &SignalRunOptions,
    signal: &mut dyn signals::SignalProgram,
    warn_on_close_error: bool,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut keyboard = WootingRgb::open(sdk_path.as_deref())?;
    print_info(keyboard.info());
    run_signal(&keyboard, options, signal, interrupted)?;
    close_best_effort(&mut keyboard, warn_on_close_error);
    Ok(())
}

fn close_best_effort(keyboard: &mut WootingRgb, warn: bool) {
    if let Err(error) = keyboard.close()
        && warn
    {
        eprintln!(
            "warning: {error}; SDK close/reset was not acknowledged, but the handle was closed"
        );
    }
}

fn print_config(config: &AppConfig) {
    let signal = config.signal_config();
    println!("config:");
    println!("  sdk_path: {}", path_display(config.sdk_path.as_ref()));
    println!("  signal: {:?}", signal.kind);
    println!("  effect: {}", config.effect);
    println!("  palette: {}", config.palette);
    println!("  brightness: {}", config.brightness);
    println!("  fps: {}", config.fps);
    println!("  seconds: {:?}", config.seconds);
    println!("  continuous: {}", config.continuous);
    println!("  warn_on_close_error: {}", config.warn_on_close_error);
    if !config.sources.is_empty() {
        println!("  sources: {}", config.sources.len());
        for source in &config.sources {
            println!("    {}: {:?}", source.id, source.kind);
        }
    }
    if !config.rules.is_empty() {
        println!("  rules: {}", config.rules.len());
        for rule in &config.rules {
            println!("    priority {} -> {}", rule.priority, rule.scene);
        }
    }
    if !config.scenes.is_empty() {
        println!("  scenes: {}", config.scenes.len());
        for name in config.scenes.keys() {
            println!("    {name}");
        }
    }
    if !config.sources.is_empty() && !config.rules.is_empty() {
        println!("  selected_scenes:");
        for source in &config.sources {
            for status in ["running", "success", "failure", "timeout", "interrupted"] {
                if let Some(selected) = config.select_scene(&source.id, status) {
                    println!(
                        "    {}.{status}: {} (priority {}, effect {:?})",
                        source.id, selected.name, selected.rule.priority, selected.scene.effect
                    );
                }
            }
        }
    }
    if signal.kind == SignalKind::CommandPulse {
        println!("  command: {:?}", signal.command_pulse.command);
        println!("  cwd: {}", path_display(signal.command_pulse.cwd.as_ref()));
        println!("  env_overrides: {}", signal.command_pulse.env.len());
        println!("  output: {:?}", signal.command_pulse.output);
        println!("  summary: {}", signal.command_pulse.summary);
        println!(
            "  timeout_seconds: {}",
            signal.command_pulse.timeout_seconds
        );
        println!(
            "  hold_seconds: success={}, failure={}, interrupted={}",
            signal.command_pulse.success_hold_seconds,
            signal.command_pulse.failure_hold_seconds,
            signal.command_pulse.interrupted_hold_seconds
        );
    }
}

fn parse_env_vars(values: Vec<String>) -> Result<BTreeMap<String, String>, String> {
    let mut env = BTreeMap::new();
    for value in values {
        let (key, var_value) = value
            .split_once('=')
            .ok_or_else(|| format!("--env must use KEY=VALUE, got {value:?}"))?;
        if key.is_empty() {
            return Err(format!("--env key cannot be empty in {value:?}"));
        }
        env.insert(key.to_string(), var_value.to_string());
    }
    Ok(env)
}

fn path_display(path: Option<&PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "<auto>".to_string())
}

fn print_info(info: &DeviceInfo) {
    println!("model: {}", info.model);
    println!("connected: {}", info.connected);
    println!(
        "matrix: {} rows x {} columns",
        info.max_rows, info.max_columns
    );
    println!("led_index_max: {}", info.led_index_max);
    println!("device_type: {:?}", info.device_type);
    println!("layout: {:?}", info.layout);
    println!("v2_interface: {}", info.v2_interface);
    println!("uses_small_packets: {}", info.uses_small_packets);
    println!("uses_multi_report: {}", info.uses_multi_report);
}

fn install_ctrlc_handler() -> Result<Arc<AtomicBool>, ctrlc::Error> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    })?;
    Ok(interrupted)
}
