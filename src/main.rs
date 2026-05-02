mod config;
mod effects;
mod ffi;
mod layout;
mod runner;
mod wooting;

use clap::{Parser, Subcommand};
use config::AppConfig;
use effects::{Color, EffectKind, PaletteName};
use layout::KeyboardLayout;
use runner::{RunOptions, run_effect, sleep_interruptibly};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use wooting::{DeviceInfo, WootingRgb};

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
    /// Run any named effect.
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
    /// Run a TOML profile.
    Run {
        /// Config file path.
        #[arg(long)]
        config: PathBuf,
        /// Print resolved config without touching the keyboard.
        #[arg(long)]
        dry_run: bool,
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
        Command::Run { config, dry_run } => {
            let config = AppConfig::load(&config)?;
            print_config(&config);
            if !dry_run {
                let sdk_path = cli.sdk_path.or(config.sdk_path.clone());
                run_keyboard_with_close_policy(
                    sdk_path,
                    &config.run_options(),
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
    println!("config:");
    println!("  sdk_path: {}", path_display(config.sdk_path.as_ref()));
    println!("  effect: {}", config.effect);
    println!("  palette: {}", config.palette);
    println!("  brightness: {}", config.brightness);
    println!("  fps: {}", config.fps);
    println!("  seconds: {:?}", config.seconds);
    println!("  continuous: {}", config.continuous);
    println!("  warn_on_close_error: {}", config.warn_on_close_error);
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
