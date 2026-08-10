use super::biquad::{Biquad, BiquadCoefficients};

/// The host-supplied settings for one EQ band.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandSettings {
    pub frequency_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

/// The complete settings for OpenEQ's fixed three-band topology.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqSettings {
    pub low: BandSettings,
    pub mid: BandSettings,
    pub high: BandSettings,
}

#[derive(Debug, Default)]
struct ChannelEq {
    low: Biquad,
    mid: Biquad,
    high: Biquad,
}

impl ChannelEq {
    fn set_low(&mut self, coefficients: BiquadCoefficients) {
        self.low.set_coefficients(coefficients);
    }

    fn set_mid(&mut self, coefficients: BiquadCoefficients) {
        self.mid.set_coefficients(coefficients);
    }

    fn set_high(&mut self, coefficients: BiquadCoefficients) {
        self.high.set_coefficients(coefficients);
    }

    fn reset(&mut self) {
        self.low.reset();
        self.mid.reset();
        self.high.reset();
    }

    fn process(&mut self, sample: f32) -> f32 {
        self.high
            .process(self.mid.process(self.low.process(sample)))
    }
}

/// A matched pair of three-band EQ filter chains.
///
/// One set of coefficients is calculated for both channels. This keeps stereo imaging stable and
/// avoids duplicate work in the real-time callback.
#[derive(Debug, Default)]
pub struct StereoEq {
    left: ChannelEq,
    right: ChannelEq,
    last_settings: Option<EqSettings>,
}

impl StereoEq {
    /// Updates only the bands whose settings have changed since the previous frame.
    pub fn update(&mut self, sample_rate: f32, settings: EqSettings) {
        let previous = self.last_settings;

        if previous.is_none_or(|last| last.low != settings.low) {
            let coefficients = BiquadCoefficients::low_shelf(
                sample_rate,
                settings.low.frequency_hz,
                settings.low.gain_db,
            );
            self.left.set_low(coefficients);
            self.right.set_low(coefficients);
        }

        if previous.is_none_or(|last| last.mid != settings.mid) {
            let coefficients = BiquadCoefficients::peaking(
                sample_rate,
                settings.mid.frequency_hz,
                settings.mid.gain_db,
                settings.mid.q,
            );
            self.left.set_mid(coefficients);
            self.right.set_mid(coefficients);
        }

        if previous.is_none_or(|last| last.high != settings.high) {
            let coefficients = BiquadCoefficients::high_shelf(
                sample_rate,
                settings.high.frequency_hz,
                settings.high.gain_db,
            );
            self.left.set_high(coefficients);
            self.right.set_high(coefficients);
        }

        self.last_settings = Some(settings);
    }

    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
        self.last_settings = None;
    }

    pub fn process_left(&mut self, sample: f32) -> f32 {
        self.left.process(sample)
    }

    pub fn process_right(&mut self, sample: f32) -> f32 {
        self.right.process(sample)
    }
}

/// Returns the EQ's total linear magnitude at `frequency_hz`.
///
/// This is intended for the editor's response plot, never the audio callback. It designs the
/// three stateless filters at the requested point, so callers do not interact with live filter
/// delay state.
pub fn response_magnitude(sample_rate: f32, settings: EqSettings, frequency_hz: f32) -> f32 {
    let low =
        BiquadCoefficients::low_shelf(sample_rate, settings.low.frequency_hz, settings.low.gain_db);
    let mid = BiquadCoefficients::peaking(
        sample_rate,
        settings.mid.frequency_hz,
        settings.mid.gain_db,
        settings.mid.q,
    );
    let high = BiquadCoefficients::high_shelf(
        sample_rate,
        settings.high.frequency_hz,
        settings.high.gain_db,
    );

    low.magnitude_at(sample_rate, frequency_hz)
        * mid.magnitude_at(sample_rate, frequency_hz)
        * high.magnitude_at(sample_rate, frequency_hz)
}

#[cfg(test)]
mod tests {
    use super::{BandSettings, EqSettings, StereoEq};

    fn default_settings() -> EqSettings {
        EqSettings {
            low: BandSettings {
                frequency_hz: 120.0,
                gain_db: 0.0,
                q: 1.0,
            },
            mid: BandSettings {
                frequency_hz: 1_000.0,
                gain_db: 0.0,
                q: 1.0,
            },
            high: BandSettings {
                frequency_hz: 8_000.0,
                gain_db: 0.0,
                q: 1.0,
            },
        }
    }

    #[test]
    fn unity_settings_preserve_a_stereo_signal() {
        let mut eq = StereoEq::default();
        eq.update(48_000.0, default_settings());

        for sample in [-1.0, -0.2, 0.0, 0.4, 1.0] {
            assert!((eq.process_left(sample) - sample).abs() < 1e-6);
            assert!((eq.process_right(sample) - sample).abs() < 1e-6);
        }
    }

    #[test]
    fn equal_channels_receive_equal_processing() {
        let mut eq = StereoEq::default();
        let mut settings = default_settings();
        settings.mid.gain_db = 9.0;
        settings.mid.q = 1.5;
        eq.update(48_000.0, settings);

        for sample in [1.0, 0.0, -0.5, 0.25, 0.0] {
            assert_eq!(eq.process_left(sample), eq.process_right(sample));
        }
    }

    #[test]
    fn response_magnitude_is_unity_for_flat_eq() {
        let settings = default_settings();

        for frequency_hz in [20.0, 120.0, 1_000.0, 8_000.0, 20_000.0] {
            assert!(
                (super::response_magnitude(48_000.0, settings, frequency_hz) - 1.0).abs() < 1e-5,
                "flat EQ should be unity at {frequency_hz} Hz"
            );
        }
    }
}
