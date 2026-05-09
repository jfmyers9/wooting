use crate::layout::Zone;
use crate::render::{Color, Frame, RenderContext, pulse_wave};
use crate::signals::SignalProgram;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct CommandPulseConfig {
    pub command: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub timeout_seconds: u64,
    pub success_hold_seconds: u64,
    pub failure_hold_seconds: u64,
    pub interrupted_hold_seconds: u64,
}

impl Default for CommandPulseConfig {
    fn default() -> Self {
        Self {
            command: Vec::new(),
            cwd: None,
            timeout_seconds: 600,
            success_hold_seconds: 3,
            failure_hold_seconds: 6,
            interrupted_hold_seconds: 2,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandPulseError {
    #[error("command-pulse requires a command")]
    EmptyCommand,
}

#[derive(Debug)]
pub struct CommandPulseSignal {
    config: CommandPulseConfig,
    child: Option<Child>,
    state: CommandPulseState,
}

#[derive(Debug)]
enum CommandPulseState {
    Pending,
    Running { started: Instant },
    Success { completed: Instant },
    Failure { completed: Instant },
    TimedOut { completed: Instant },
    Interrupted { completed: Instant },
}

impl CommandPulseSignal {
    pub fn new(config: CommandPulseConfig) -> Result<Self, CommandPulseError> {
        if config.command.is_empty() {
            return Err(CommandPulseError::EmptyCommand);
        }

        Ok(Self {
            config,
            child: None,
            state: CommandPulseState::Pending,
        })
    }

    fn start(&mut self) {
        let mut command = Command::new(&self.config.command[0]);
        command.args(&self.config.command[1..]);
        if let Some(cwd) = &self.config.cwd {
            command.current_dir(cwd);
        }
        command.stdin(Stdio::null());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        match command.spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.state = CommandPulseState::Running {
                    started: Instant::now(),
                };
            }
            Err(error) => {
                eprintln!("command-pulse failed to start: {error}");
                self.state = CommandPulseState::Failure {
                    completed: Instant::now(),
                };
            }
        }
    }

    fn poll_child(&mut self, interrupted: &AtomicBool) {
        let CommandPulseState::Running { started } = self.state else {
            return;
        };

        if interrupted.load(Ordering::SeqCst) {
            self.kill_child();
            self.state = CommandPulseState::Interrupted {
                completed: Instant::now(),
            };
            return;
        }

        if started.elapsed() > Duration::from_secs(self.config.timeout_seconds) {
            self.kill_child();
            self.state = CommandPulseState::TimedOut {
                completed: Instant::now(),
            };
            return;
        }

        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.child = None;
                    let completed = Instant::now();
                    if status.success() {
                        self.state = CommandPulseState::Success { completed };
                    } else {
                        self.state = CommandPulseState::Failure { completed };
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    eprintln!("command-pulse failed to poll child: {error}");
                    self.kill_child();
                    self.state = CommandPulseState::Failure {
                        completed: Instant::now(),
                    };
                }
            }
        }
    }

    fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn terminal_hold_elapsed(&self, completed: Instant, hold_seconds: u64) -> bool {
        completed.elapsed() >= Duration::from_secs(hold_seconds)
    }

    fn terminal_color(&self) -> Color {
        match self.state {
            CommandPulseState::Pending => Color::new(0, 80, 180),
            CommandPulseState::Running { .. } => Color::new(0, 180, 255),
            CommandPulseState::Success { .. } => Color::new(0, 220, 80),
            CommandPulseState::Failure { .. } => Color::new(255, 32, 24),
            CommandPulseState::TimedOut { .. } => Color::new(255, 128, 0),
            CommandPulseState::Interrupted { .. } => Color::new(160, 80, 255),
        }
    }
}

impl SignalProgram for CommandPulseSignal {
    fn tick(&mut self, interrupted: &AtomicBool) {
        match self.state {
            CommandPulseState::Pending => self.start(),
            CommandPulseState::Running { .. } => self.poll_child(interrupted),
            CommandPulseState::Success { .. }
            | CommandPulseState::Failure { .. }
            | CommandPulseState::TimedOut { .. }
            | CommandPulseState::Interrupted { .. } => {}
        }
    }

    fn render(&self, ctx: &RenderContext<'_>) -> Frame {
        let mut frame = Frame::black();
        let wave = pulse_wave(ctx.tick, 24);
        let base = self.terminal_color();
        let scale = match self.state {
            CommandPulseState::Pending => 48,
            CommandPulseState::Running { .. } => 72 + (wave / 4),
            CommandPulseState::Success { .. } => 160 + (wave / 4),
            CommandPulseState::Failure { .. }
            | CommandPulseState::TimedOut { .. }
            | CommandPulseState::Interrupted { .. } => 128 + (wave / 2),
        };
        let color = base.scale(((u16::from(ctx.brightness) * u16::from(scale)) / 255) as u8);
        let dim = base.scale(ctx.brightness / 8);

        for key in ctx.layout.keys() {
            frame.set_coord(key.coord, dim);
        }

        let mut status_keys = ctx
            .layout
            .keys()
            .iter()
            .filter(|key| key.zone == Zone::Function)
            .collect::<Vec<_>>();
        if status_keys.is_empty() {
            status_keys = ctx.layout.keys().iter().collect();
        }
        status_keys.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));

        match self.state {
            CommandPulseState::Running { .. } => {
                if !status_keys.is_empty() {
                    let head = usize::try_from(ctx.tick).unwrap_or(0) % status_keys.len();
                    for offset in 0..6.min(status_keys.len()) {
                        let index = (head + status_keys.len() - offset) % status_keys.len();
                        let fade = 255u8.saturating_sub((offset * 40) as u8);
                        frame.set_coord(
                            status_keys[index].coord,
                            base.scale(fade).scale(ctx.brightness),
                        );
                    }
                }
            }
            _ => {
                for key in status_keys {
                    frame.set_coord(key.coord, color);
                }
            }
        }

        frame
    }

    fn finished(&self) -> bool {
        match self.state {
            CommandPulseState::Pending | CommandPulseState::Running { .. } => false,
            CommandPulseState::Success { completed } => {
                self.terminal_hold_elapsed(completed, self.config.success_hold_seconds)
            }
            CommandPulseState::Failure { completed }
            | CommandPulseState::TimedOut { completed } => {
                self.terminal_hold_elapsed(completed, self.config.failure_hold_seconds)
            }
            CommandPulseState::Interrupted { completed } => {
                self.terminal_hold_elapsed(completed, self.config.interrupted_hold_seconds)
            }
        }
    }

    fn shutdown(&mut self, interrupted: bool) {
        if self.child.is_some() {
            self.kill_child();
            if interrupted {
                self.state = CommandPulseState::Interrupted {
                    completed: Instant::now(),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::KeyboardLayout;
    use crate::render::{PaletteName, RenderContext};
    use crate::sdk::rgb::{DeviceInfo, DeviceType, Layout};
    use std::sync::atomic::AtomicBool;
    use std::thread;

    fn info() -> DeviceInfo {
        DeviceInfo {
            connected: true,
            model: "test".to_string(),
            max_rows: 6,
            max_columns: 17,
            led_index_max: 0,
            device_type: DeviceType::Keyboard80,
            layout: Layout::Ansi,
            v2_interface: true,
            uses_small_packets: false,
            uses_multi_report: false,
        }
    }

    fn config(command: &str) -> CommandPulseConfig {
        CommandPulseConfig {
            command: vec![command.to_string()],
            success_hold_seconds: 0,
            failure_hold_seconds: 0,
            interrupted_hold_seconds: 0,
            timeout_seconds: 5,
            ..CommandPulseConfig::default()
        }
    }

    #[test]
    fn command_pulse_rejects_empty_command() {
        assert!(CommandPulseSignal::new(CommandPulseConfig::default()).is_err());
    }

    #[test]
    fn command_pulse_finishes_success() {
        let interrupted = AtomicBool::new(false);
        let mut signal = CommandPulseSignal::new(config("true")).unwrap();
        for _ in 0..100 {
            signal.tick(&interrupted);
            if signal.finished() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("command did not finish");
    }

    #[test]
    fn command_pulse_renders_full_frame() {
        let info = info();
        let layout = KeyboardLayout::for_device(&info);
        let signal = CommandPulseSignal::new(config("true")).unwrap();
        let frame = signal.render(&RenderContext {
            info: &info,
            layout: &layout,
            brightness: 96,
            palette: PaletteName::Wooting,
            tick: 0,
        });
        assert_eq!(frame.as_bytes().len(), crate::render::FRAME_BYTES);
    }
}
