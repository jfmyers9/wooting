use crate::layout::{KeyboardLayout, MatrixCoord};
use crate::sdk::rgb::DeviceInfo;
use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;

pub const MAX_ROWS: usize = 6;
pub const MAX_COLUMNS: usize = 21;
pub const RGB_CHANNELS: usize = 3;
pub const FRAME_BYTES: usize = MAX_ROWS * MAX_COLUMNS * RGB_CHANNELS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn scale(self, max_channel: u8) -> Self {
        let scale = |value: u8| ((u16::from(value) * u16::from(max_channel)) / 255) as u8;
        Self::new(scale(self.red), scale(self.green), scale(self.blue))
    }

    fn blend(self, other: Self, amount: u8) -> Self {
        let blend_channel = |a: u8, b: u8| {
            let a = u16::from(a) * u16::from(255 - amount);
            let b = u16::from(b) * u16::from(amount);
            ((a + b) / 255) as u8
        };
        Self::new(
            blend_channel(self.red, other.red),
            blend_channel(self.green, other.green),
            blend_channel(self.blue, other.blue),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    bytes: [u8; FRAME_BYTES],
}

impl Frame {
    pub fn black() -> Self {
        Self {
            bytes: [0; FRAME_BYTES],
        }
    }

    pub fn set(&mut self, row: usize, column: usize, color: Color) {
        if row >= MAX_ROWS || column >= MAX_COLUMNS {
            return;
        }

        let offset = ((row * MAX_COLUMNS) + column) * RGB_CHANNELS;
        self.bytes[offset] = color.red;
        self.bytes[offset + 1] = color.green;
        self.bytes[offset + 2] = color.blue;
    }

    pub fn set_coord(&mut self, coord: MatrixCoord, color: Color) {
        self.set(usize::from(coord.row), usize::from(coord.column), color);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum PaletteName {
    #[default]
    Wooting,
    Cyberpunk,
    Ocean,
    Heat,
    Terminal,
}

impl fmt::Display for PaletteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_possible_value()
                .expect("palette has value")
                .get_name()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Palette {
    colors: &'static [Color],
}

const WOOTING_PALETTE: &[Color] = &[
    Color::new(0, 180, 255),
    Color::new(255, 255, 255),
    Color::new(0, 90, 220),
];
const CYBERPUNK_PALETTE: &[Color] = &[
    Color::new(255, 0, 120),
    Color::new(0, 255, 255),
    Color::new(255, 220, 0),
];
const OCEAN_PALETTE: &[Color] = &[
    Color::new(0, 32, 96),
    Color::new(0, 160, 220),
    Color::new(120, 255, 255),
];
const HEAT_PALETTE: &[Color] = &[
    Color::new(80, 0, 0),
    Color::new(255, 64, 0),
    Color::new(255, 220, 64),
];
const TERMINAL_PALETTE: &[Color] = &[
    Color::new(0, 32, 0),
    Color::new(0, 220, 64),
    Color::new(180, 255, 180),
];

impl PaletteName {
    pub fn palette(self) -> Palette {
        let colors = match self {
            Self::Wooting => WOOTING_PALETTE,
            Self::Cyberpunk => CYBERPUNK_PALETTE,
            Self::Ocean => OCEAN_PALETTE,
            Self::Heat => HEAT_PALETTE,
            Self::Terminal => TERMINAL_PALETTE,
        };
        Palette { colors }
    }
}

impl Palette {
    pub fn sample(&self, tick: u32) -> Color {
        self.colors[usize::try_from(tick).unwrap_or(0) % self.colors.len()]
    }

    pub fn gradient(&self, position: u8) -> Color {
        if self.colors.len() == 1 {
            return self.colors[0];
        }

        if position == 255 {
            return self.colors[self.colors.len() - 1];
        }

        let segments = self.colors.len() - 1;
        let scaled = usize::from(position) * segments;
        let index = (scaled / 255).min(segments - 1);
        let amount = (scaled % 255) as u8;
        self.colors[index].blend(self.colors[index + 1], amount)
    }
}

#[derive(Clone, Debug)]
pub struct RenderContext<'a> {
    pub info: &'a DeviceInfo,
    pub layout: &'a KeyboardLayout,
    pub brightness: u8,
    pub palette: PaletteName,
    pub tick: u32,
}

pub fn pulse_wave(tick: u32, period: u32) -> u8 {
    let period = period.max(2);
    let phase = tick % period;
    let half = period / 2;
    if phase < half {
        ((phase * 255) / half) as u8
    } else {
        (((period - phase) * 255) / half) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_uses_official_full_matrix_size() {
        assert_eq!(Frame::black().as_bytes().len(), 6 * 21 * 3);
    }

    #[test]
    fn frame_ignores_out_of_bounds_writes() {
        let mut frame = Frame::black();

        frame.set(MAX_ROWS, 0, Color::new(255, 0, 0));
        frame.set(0, MAX_COLUMNS, Color::new(0, 255, 0));

        assert!(frame.as_bytes().iter().all(|channel| *channel == 0));
    }

    #[test]
    fn palette_gradient_samples_endpoints() {
        let palette = PaletteName::Heat.palette();
        assert_eq!(palette.gradient(0), Color::new(80, 0, 0));
        assert_eq!(palette.gradient(255), Color::new(255, 220, 64));
    }
}
