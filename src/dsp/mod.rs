//! Allocation-free, framework-independent EQ DSP.

mod biquad;
mod eq;

pub use eq::{BandSettings, EqSettings, StereoEq, response_magnitude};
