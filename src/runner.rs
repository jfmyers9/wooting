use crate::effects::{EffectKind, PaletteName, RenderContext};
use crate::layout::KeyboardLayout;
use crate::wooting::WootingRgb;
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

pub fn run_effect(
    keyboard: &WootingRgb,
    options: &RunOptions,
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
        && deadline.is_none_or(|deadline| Instant::now() < deadline)
    {
        let started = Instant::now();
        let frame = options.effect.render(&RenderContext {
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

    Ok(())
}

pub fn sleep_interruptibly(duration: Duration, interrupted: &AtomicBool) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline && !interrupted.load(Ordering::SeqCst) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(remaining.min(Duration::from_millis(25)));
    }
}
