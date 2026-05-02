use crate::layout::{KeyboardLayout, MatrixCoord};
use crate::wooting::DeviceInfo;
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
pub enum EffectKind {
    RowTest,
    Rainbow,
    Comet,
    Matrix,
    Breath,
}

impl fmt::Display for EffectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_possible_value()
                .expect("effect has value")
                .get_name()
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PaletteName {
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
    fn sample(&self, tick: u32) -> Color {
        self.colors[usize::try_from(tick).unwrap_or(0) % self.colors.len()]
    }

    fn gradient(&self, position: u8) -> Color {
        if self.colors.len() == 1 {
            return self.colors[0];
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

impl EffectKind {
    pub fn render(self, ctx: &RenderContext<'_>) -> Frame {
        match self {
            Self::RowTest => row_test(ctx),
            Self::Rainbow => rainbow(ctx),
            Self::Comet => comet(ctx),
            Self::Matrix => matrix(ctx),
            Self::Breath => breath(ctx),
        }
    }
}

fn row_test(ctx: &RenderContext<'_>) -> Frame {
    let mut frame = Frame::black();
    let palette = [
        Color::new(255, 0, 0),
        Color::new(255, 128, 0),
        Color::new(255, 255, 0),
        Color::new(0, 255, 0),
        Color::new(0, 128, 255),
        Color::new(128, 0, 255),
    ];

    for row in 0..usize::from(ctx.info.max_rows) {
        for column in 0..usize::from(ctx.info.max_columns) {
            frame.set(
                row,
                column,
                palette[row % palette.len()].scale(ctx.brightness),
            );
        }
    }

    frame
}

fn rainbow(ctx: &RenderContext<'_>) -> Frame {
    let mut frame = Frame::black();
    let rows = usize::from(ctx.info.max_rows).max(1);
    let columns = usize::from(ctx.info.max_columns).max(1);

    for row in 0..rows {
        for column in 0..columns {
            let position = ((column * 360) / columns) as u16;
            let vertical = ((row * 90) / rows) as u16;
            let hue = (position + vertical + ((ctx.tick as u16) * 6)) % 360;
            frame.set(row, column, hsv_to_rgb(hue, 255, ctx.brightness));
        }
    }

    frame
}

fn comet(ctx: &RenderContext<'_>) -> Frame {
    let mut frame = Frame::black();
    let keys = ctx.layout.keys();
    if keys.is_empty() {
        return frame;
    }

    let palette = ctx.palette.palette();
    let head = usize::try_from(ctx.tick).unwrap_or(0) % keys.len();
    let trail = 10.min(keys.len());

    for offset in 0..trail {
        let index = (head + keys.len() - offset) % keys.len();
        let fade = 255u8.saturating_sub(((offset * 255) / trail) as u8);
        let color = palette
            .gradient(fade)
            .scale(((u16::from(ctx.brightness) * u16::from(fade)) / 255) as u8);
        frame.set_coord(keys[index].coord, color);
    }

    frame
}

fn matrix(ctx: &RenderContext<'_>) -> Frame {
    let mut frame = Frame::black();
    let palette = PaletteName::Terminal.palette();
    let rows = ctx.info.max_rows.max(1);

    for key in ctx.layout.keys() {
        let column_seed = u32::from(key.coord.column) * 3;
        let head = ((ctx.tick + column_seed) % u32::from(rows)) as i16;
        let distance = (head - i16::from(key.coord.row)).rem_euclid(i16::from(rows)) as u8;
        if distance <= 3 {
            let fade = 255u8.saturating_sub(distance * 70);
            frame.set_coord(key.coord, palette.gradient(fade).scale(ctx.brightness));
        }
    }

    frame
}

fn breath(ctx: &RenderContext<'_>) -> Frame {
    let mut frame = Frame::black();
    let palette = ctx.palette.palette();
    let phase = triangular_wave(ctx.tick, 96);
    let brightness = ((u16::from(ctx.brightness) * u16::from(phase)) / 255) as u8;
    let color = palette.sample(ctx.tick / 96).scale(brightness);

    for key in ctx.layout.keys() {
        frame.set_coord(key.coord, color);
    }

    frame
}

fn triangular_wave(tick: u32, period: u32) -> u8 {
    let phase = tick % period;
    let half = period / 2;
    if phase < half {
        ((phase * 255) / half) as u8
    } else {
        (((period - phase) * 255) / half) as u8
    }
}

fn hsv_to_rgb(hue: u16, saturation: u8, value: u8) -> Color {
    if saturation == 0 {
        return Color::new(value, value, value);
    }

    let region = hue / 60;
    let remainder = ((hue % 60) * 255 / 60) as u8;

    let p = scale_down(value, 255 - saturation);
    let q = scale_down(value, 255 - scale_down(saturation, remainder));
    let t = scale_down(value, 255 - scale_down(saturation, 255 - remainder));

    match region {
        0 => Color::new(value, t, p),
        1 => Color::new(q, value, p),
        2 => Color::new(p, value, t),
        3 => Color::new(p, q, value),
        4 => Color::new(t, p, value),
        _ => Color::new(value, p, q),
    }
}

fn scale_down(a: u8, b: u8) -> u8 {
    ((u16::from(a) * u16::from(b)) / 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::KeyboardLayout;
    use crate::wooting::{DeviceType, Layout};

    fn info(rows: u8, columns: u8) -> DeviceInfo {
        DeviceInfo {
            connected: true,
            model: "test".to_string(),
            max_rows: rows,
            max_columns: columns,
            led_index_max: 0,
            device_type: DeviceType::Keyboard60,
            layout: Layout::Ansi,
            v2_interface: true,
            uses_small_packets: false,
            uses_multi_report: false,
        }
    }

    #[test]
    fn frame_uses_official_full_matrix_size() {
        assert_eq!(Frame::black().as_bytes().len(), 6 * 21 * 3);
    }

    #[test]
    fn row_test_respects_device_bounds() {
        let info = info(1, 2);
        let layout = KeyboardLayout::for_device(&info);
        let frame = EffectKind::RowTest.render(&RenderContext {
            info: &info,
            layout: &layout,
            brightness: 10,
            palette: PaletteName::Wooting,
            tick: 0,
        });
        assert!(frame.as_bytes()[0..6].iter().any(|channel| *channel > 0));
        assert!(frame.as_bytes()[6..].iter().all(|channel| *channel == 0));
    }

    #[test]
    fn all_effects_render_full_frames() {
        let info = info(6, 17);
        let layout = KeyboardLayout::for_device(&info);
        for effect in [
            EffectKind::RowTest,
            EffectKind::Rainbow,
            EffectKind::Comet,
            EffectKind::Matrix,
            EffectKind::Breath,
        ] {
            let frame = effect.render(&RenderContext {
                info: &info,
                layout: &layout,
                brightness: 96,
                palette: PaletteName::Cyberpunk,
                tick: 5,
            });
            assert_eq!(frame.as_bytes().len(), FRAME_BYTES);
        }
    }
}
