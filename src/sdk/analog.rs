#![allow(dead_code)]

//! Research notes and safe boundary for future Wooting analog input.
//!
//! Wooting Signals should use `wooting-analog-sdk_dist` as an application
//! dependency when Analog Lava Lab moves beyond planning. It should not become
//! an Analog SDK plugin unless the project starts supporting new hardware.
//!
//! Open implementation questions:
//! - how the distributable SDK is located on macOS, Linux, and Windows;
//! - what permissions are required for concurrent RGB output and analog input;
//! - how HID key codes map to the RGB matrix for 60/80/full-size layouts;
//! - how to sample pressure without adding distracting CPU overhead.

use crate::layout::MatrixCoord;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalogKeyPressure {
    pub key_code: u16,
    pub pressure: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalogCapability {
    Unavailable,
    RequiresSdkResearch,
}

pub fn analog_capability() -> AnalogCapability {
    AnalogCapability::RequiresSdkResearch
}

pub fn pressure_to_heat(pressure: f32) -> u8 {
    (pressure.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn unresolved_matrix_mapping(_key_code: u16) -> Option<MatrixCoord> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_heat_is_clamped() {
        assert_eq!(pressure_to_heat(-1.0), 0);
        assert_eq!(pressure_to_heat(0.5), 128);
        assert_eq!(pressure_to_heat(2.0), 255);
    }

    #[test]
    fn matrix_mapping_is_explicitly_unresolved() {
        assert_eq!(unresolved_matrix_mapping(30), None);
    }
}
