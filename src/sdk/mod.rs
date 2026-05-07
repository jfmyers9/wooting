//! Wooting SDK backends used by the extension host.
//!
//! RGB output is implemented today through the Wooting RGB SDK. Future analog
//! input should live in an `analog` backend that loads `wooting-analog-sdk_dist`
//! as an application dependency, not as an Analog SDK plugin.

pub mod rgb;
mod rgb_ffi;
