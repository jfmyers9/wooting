mod effects;
mod ffi;
mod wooting;

use clap::{Parser, Subcommand};
use effects::{Color, rainbow, row_test};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
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
    /// Paint a short, low-brightness row test pattern, then reset.
    Test {
        /// Maximum RGB channel value. Keep low while experimenting.
        #[arg(long, default_value_t = 16)]
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
        /// Maximum RGB channel value. Keep low while experimenting.
        #[arg(long, default_value_t = 16)]
        brightness: u8,
        /// Seconds to keep the key visible.
        #[arg(long, default_value_t = 3)]
        seconds: u64,
    },
    /// Run a device-bounded rainbow animation, then reset.
    Rainbow {
        /// Maximum RGB channel value. Keep low while experimenting.
        #[arg(long, default_value_t = 24)]
        brightness: u8,
        /// Seconds to run the animation.
        #[arg(long, default_value_t = 10)]
        seconds: u64,
        /// Animation frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
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
        Command::Test {
            brightness,
            seconds,
        } => {
            let mut keyboard = WootingRgb::open(cli.sdk_path.as_deref())?;
            print_info(keyboard.info());
            let frame = row_test(keyboard.info(), brightness);
            keyboard.set_frame(&frame)?;
            keyboard.set_cell(0, 0, Color::new(brightness, brightness, brightness))?;
            keyboard.update()?;
            sleep_interruptibly(Duration::from_secs(seconds), &interrupted);
            close_best_effort(&mut keyboard);
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
            close_best_effort(&mut keyboard);
        }
        Command::Rainbow {
            brightness,
            seconds,
            fps,
        } => {
            let mut keyboard = WootingRgb::open(cli.sdk_path.as_deref())?;
            print_info(keyboard.info());
            run_rainbow(&keyboard, brightness, seconds, fps.max(1), &interrupted)?;
            close_best_effort(&mut keyboard);
        }
    }

    Ok(())
}

fn close_best_effort(keyboard: &mut WootingRgb) {
    if let Err(error) = keyboard.close() {
        eprintln!(
            "warning: {error}; SDK close/reset was not acknowledged, but the handle was closed"
        );
    }
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

fn run_rainbow(
    keyboard: &WootingRgb,
    brightness: u8,
    seconds: u64,
    fps: u32,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame_time = Duration::from_secs_f64(1.0 / f64::from(fps));
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut tick = 0;

    while Instant::now() < deadline && !interrupted.load(Ordering::SeqCst) {
        let started = Instant::now();
        let frame = rainbow(keyboard.info(), brightness, tick);
        keyboard.set_frame(&frame)?;
        keyboard.update()?;
        tick += 1;

        if let Some(remaining) = frame_time.checked_sub(started.elapsed()) {
            sleep_interruptibly(remaining, interrupted);
        }
    }

    Ok(())
}

fn sleep_interruptibly(duration: Duration, interrupted: &AtomicBool) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline && !interrupted.load(Ordering::SeqCst) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(remaining.min(Duration::from_millis(25)));
    }
}

fn install_ctrlc_handler() -> Result<Arc<AtomicBool>, ctrlc::Error> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    })?;
    Ok(interrupted)
}
