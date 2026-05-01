use crate::wooting::DeviceInfo;

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

    fn scale(self, max_channel: u8) -> Self {
        let scale = |value: u8| ((u16::from(value) * u16::from(max_channel)) / 255) as u8;
        Self::new(scale(self.red), scale(self.green), scale(self.blue))
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

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub fn row_test(info: &DeviceInfo, brightness: u8) -> Frame {
    let mut frame = Frame::black();
    let palette = [
        Color::new(255, 0, 0),
        Color::new(255, 128, 0),
        Color::new(255, 255, 0),
        Color::new(0, 255, 0),
        Color::new(0, 128, 255),
        Color::new(128, 0, 255),
    ];

    for row in 0..usize::from(info.max_rows) {
        for column in 0..usize::from(info.max_columns) {
            frame.set(row, column, palette[row % palette.len()].scale(brightness));
        }
    }

    frame
}

pub fn rainbow(info: &DeviceInfo, brightness: u8, tick: u32) -> Frame {
    let mut frame = Frame::black();
    let rows = usize::from(info.max_rows).max(1);
    let columns = usize::from(info.max_columns).max(1);

    for row in 0..rows {
        for column in 0..columns {
            let position = ((column * 360) / columns) as u16;
            let vertical = ((row * 90) / rows) as u16;
            let hue = (position + vertical + ((tick as u16) * 6)) % 360;
            frame.set(row, column, hsv_to_rgb(hue, 255, brightness));
        }
    }

    frame
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
        let frame = row_test(&info(1, 2), 10);
        assert!(frame.as_bytes()[0..6].iter().any(|channel| *channel > 0));
        assert!(frame.as_bytes()[6..].iter().all(|channel| *channel == 0));
    }
}
