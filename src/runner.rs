use crate::effects::EffectKind;
use crate::layout::KeyboardLayout;
use crate::render::{PaletteName, RenderContext};
use crate::sdk::rgb::WootingRgb;
use crate::signals::{SignalProgram, StaticEffectSignal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub effect: EffectKind,
    pub palette: PaletteName,
    pub brightness: u8,
    pub fps: u32,
    pub seconds: Option<u64>,
    pub continuous: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            effect: EffectKind::Rainbow,
            palette: PaletteName::Wooting,
            brightness: 96,
            fps: 30,
            seconds: Some(10),
            continuous: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignalRunOptions {
    pub palette: PaletteName,
    pub brightness: u8,
    pub fps: u32,
    pub seconds: Option<u64>,
    pub continuous: bool,
}

impl Default for SignalRunOptions {
    fn default() -> Self {
        let run = RunOptions::default();
        Self {
            palette: run.palette,
            brightness: run.brightness,
            fps: run.fps,
            seconds: run.seconds,
            continuous: run.continuous,
        }
    }
}

impl From<&RunOptions> for SignalRunOptions {
    fn from(options: &RunOptions) -> Self {
        Self {
            palette: options.palette,
            brightness: options.brightness,
            fps: options.fps,
            seconds: options.seconds,
            continuous: options.continuous,
        }
    }
}

pub fn run_effect(
    keyboard: &WootingRgb,
    options: &RunOptions,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut signal = StaticEffectSignal::new(options.effect);
    run_signal(
        keyboard,
        &SignalRunOptions::from(options),
        &mut signal,
        interrupted,
    )
}

pub fn run_signal(
    keyboard: &WootingRgb,
    options: &SignalRunOptions,
    signal: &mut dyn SignalProgram,
    interrupted: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error>> {
    let fps = options.fps.max(1);
    let frame_time = Duration::from_secs_f64(1.0 / f64::from(fps));
    let deadline = options
        .seconds
        .filter(|_| !options.continuous)
        .map(|seconds| Instant::now() + Duration::from_secs(seconds));
    let layout = KeyboardLayout::for_device(keyboard.info());
    let mut tick = 0;

    while !interrupted.load(Ordering::SeqCst)
        && !signal.finished()
        && deadline.is_none_or(|deadline| Instant::now() < deadline)
    {
        let started = Instant::now();
        signal.tick(interrupted);
        let frame = signal.render(&RenderContext {
            info: keyboard.info(),
            layout: &layout,
            brightness: options.brightness,
            palette: options.palette,
            tick,
        });
        keyboard.set_frame(&frame)?;
        keyboard.update()?;
        tick += 1;

        if let Some(remaining) = frame_time.checked_sub(started.elapsed()) {
            sleep_interruptibly(remaining, interrupted);
        }
    }

    signal.shutdown(interrupted.load(Ordering::SeqCst));
    Ok(())
}

pub fn sleep_interruptibly(duration: Duration, interrupted: &AtomicBool) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline && !interrupted.load(Ordering::SeqCst) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(remaining.min(Duration::from_millis(25)));
    }
}
