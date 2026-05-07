mod config;
mod effects;
mod extensions;
mod layout;
mod render;
mod runner;
mod sdk;

use clap::{Parser, Subcommand};
use config::AppConfig;
use effects::EffectKind;
use extensions::{CommandPulseConfig, ExtensionKind, build_extension};
use layout::KeyboardLayout;
use render::{Color, PaletteName};
use runner::{ExtensionRunOptions, RunOptions, run_effect, run_extension, sleep_interruptibly};
use sdk::rgb::{DeviceInfo, WootingRgb};
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
    /// Run an extension directly.
    Extension {
        #[command(subcommand)]
        command: ExtensionCommand,
    },
    /// Run a TOML extension-host profile.
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
enum ExtensionCommand {
    /// Run a named extension. Use `--` before command-pulse commands.
    Run {
        /// Extension to run.
        #[arg(value_enum)]
        extension: ExtensionKind,
        /// Static effect used by the static-effect extension.
        #[arg(long, value_enum, default_value_t = EffectKind::Comet)]
        effect: EffectKind,
        /// Palette for extension renderers.
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
        /// Command timeout for command-pulse.
        #[arg(long, default_value_t = 600)]
        timeout_seconds: u64,
        /// Command-pulse success hold seconds.
        #[arg(long, default_value_t = 3)]
        success_hold_seconds: u64,
        /// Command-pulse failure hold seconds.
        #[arg(long, default_value_t = 6)]
        failure_hold_seconds: u64,
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
        Command::Extension { command } => match command {
            ExtensionCommand::Run {
                extension,
                effect,
                palette,
                brightness,
                fps,
                seconds,
                timeout_seconds,
                success_hold_seconds,
                failure_hold_seconds,
                command,
            } => {
                let (config, options) = match extension {
                    ExtensionKind::StaticEffect => (
                        extensions::ExtensionConfig::static_effect(effect),
                        ExtensionRunOptions {
                            palette,
                            brightness,
                            fps,
                            seconds: Some(seconds),
                            continuous: false,
                        },
                    ),
                    ExtensionKind::CommandPulse => (
                        extensions::ExtensionConfig::command_pulse(CommandPulseConfig {
                            command,
                            timeout_seconds,
                            success_hold_seconds,
                            failure_hold_seconds,
                            ..CommandPulseConfig::default()
                        }),
                        ExtensionRunOptions {
                            palette,
                            brightness,
                            fps,
                            seconds: None,
                            continuous: true,
                        },
                    ),
                };
                let mut extension = build_extension(&config, effect)?;
                run_keyboard_extension(
                    cli.sdk_path,
                    &options,
                    &mut *extension,
                    true,
                    &interrupted,
                )?;
            }
        },
        Command::Run { config, dry_run } => {
            let config = AppConfig::load(&config)?;
            print_config(&config);
            if !dry_run {
                let sdk_path = cli.sdk_path.or(config.sdk_path.clone());
                let mut extension = build_extension(&config.extension_config(), config.effect)?;
                run_keyboard_extension(
                    sdk_path,
                    &config.extension_run_options(),
                    &mut *extension,
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

fn run_keyboard_extension(
    sdk_path: Option<PathBuf>,
    options: &ExtensionRunOptions,
    extension: &mut dyn extensions::KeyboardExtension,
    warn_on_close_error: bool,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut keyboard = WootingRgb::open(sdk_path.as_deref())?;
    print_info(keyboard.info());
    run_extension(&keyboard, options, extension, interrupted)?;
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
    let extension = config.extension_config();
    println!("config:");
    println!("  sdk_path: {}", path_display(config.sdk_path.as_ref()));
    println!("  extension: {:?}", extension.kind);
    println!("  effect: {}", config.effect);
    println!("  palette: {}", config.palette);
    println!("  brightness: {}", config.brightness);
    println!("  fps: {}", config.fps);
    println!("  seconds: {:?}", config.seconds);
    println!("  continuous: {}", config.continuous);
    println!("  warn_on_close_error: {}", config.warn_on_close_error);
    if extension.kind == ExtensionKind::CommandPulse {
        println!("  command: {:?}", extension.command_pulse.command);
        println!(
            "  timeout_seconds: {}",
            extension.command_pulse.timeout_seconds
        );
    }
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
