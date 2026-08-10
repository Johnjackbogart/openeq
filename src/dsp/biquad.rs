//! Robert Bristow-Johnson Audio EQ Cookbook biquad filters.

use core::f32::consts::PI;

/// Normalized coefficients for a second-order IIR filter.
#[derive(Clone, Copy, Debug)]
pub(super) struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoefficients {
    pub(super) const IDENTITY: Self = Self {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };

    pub(super) fn peaking(sample_rate: f32, frequency_hz: f32, gain_db: f32, q: f32) -> Self {
        let (omega, a) = common_design_values(sample_rate, frequency_hz, gain_db);
        let alpha = omega.sin() / (2.0 * q.clamp(0.1, 24.0));
        let cos_omega = omega.cos();

        Self::normalized(
            1.0 + alpha * a,
            -2.0 * cos_omega,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos_omega,
            1.0 - alpha / a,
        )
    }

    pub(super) fn low_shelf(sample_rate: f32, frequency_hz: f32, gain_db: f32) -> Self {
        let (omega, alpha, a) = shelf_design_values(sample_rate, frequency_hz, gain_db);
        let cos_omega = omega.cos();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        Self::normalized(
            a * ((a + 1.0) - (a - 1.0) * cos_omega + two_sqrt_a_alpha),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega),
            a * ((a + 1.0) - (a - 1.0) * cos_omega - two_sqrt_a_alpha),
            (a + 1.0) + (a - 1.0) * cos_omega + two_sqrt_a_alpha,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega),
            (a + 1.0) + (a - 1.0) * cos_omega - two_sqrt_a_alpha,
        )
    }

    pub(super) fn high_shelf(sample_rate: f32, frequency_hz: f32, gain_db: f32) -> Self {
        let (omega, alpha, a) = shelf_design_values(sample_rate, frequency_hz, gain_db);
        let cos_omega = omega.cos();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        Self::normalized(
            a * ((a + 1.0) + (a - 1.0) * cos_omega + two_sqrt_a_alpha),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega),
            a * ((a + 1.0) + (a - 1.0) * cos_omega - two_sqrt_a_alpha),
            (a + 1.0) - (a - 1.0) * cos_omega + two_sqrt_a_alpha,
            2.0 * ((a - 1.0) - (a + 1.0) * cos_omega),
            (a + 1.0) - (a - 1.0) * cos_omega - two_sqrt_a_alpha,
        )
    }

    fn normalized(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        debug_assert!(a0.is_finite() && a0.abs() > f32::EPSILON);

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub(super) fn magnitude_at(self, sample_rate: f32, frequency_hz: f32) -> f32 {
        let omega = 2.0 * PI * frequency_hz / sample_rate;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let cos_two_omega = (2.0 * omega).cos();
        let sin_two_omega = (2.0 * omega).sin();

        let numerator_real = self.b0 + self.b1 * cos_omega + self.b2 * cos_two_omega;
        let numerator_imag = -(self.b1 * sin_omega + self.b2 * sin_two_omega);
        let denominator_real = 1.0 + self.a1 * cos_omega + self.a2 * cos_two_omega;
        let denominator_imag = -(self.a1 * sin_omega + self.a2 * sin_two_omega);

        (numerator_real.hypot(numerator_imag) / denominator_real.hypot(denominator_imag)).abs()
    }
}

/// A transposed direct-form II biquad. The two state values are intentionally retained when
/// changing coefficients to avoid a discontinuity in the audio stream.
#[derive(Clone, Copy, Debug)]
pub(super) struct Biquad {
    coefficients: BiquadCoefficients,
    z1: f32,
    z2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            coefficients: BiquadCoefficients::IDENTITY,
            z1: 0.0,
            z2: 0.0,
        }
    }
}

impl Biquad {
    pub(super) fn set_coefficients(&mut self, coefficients: BiquadCoefficients) {
        self.coefficients = coefficients;
    }

    pub(super) fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub(super) fn process(&mut self, input: f32) -> f32 {
        let output = self.coefficients.b0.mul_add(input, self.z1);
        self.z1 = self
            .coefficients
            .b1
            .mul_add(input, self.z2 - self.coefficients.a1 * output);
        self.z2 = self
            .coefficients
            .b2
            .mul_add(input, -self.coefficients.a2 * output);
        output
    }
}

fn common_design_values(sample_rate: f32, frequency_hz: f32, gain_db: f32) -> (f32, f32) {
    // Parameter ranges enforce these constraints at the host boundary. Clamping here preserves
    // finite coefficients if DSP callers provide malformed values in tests or future integrations.
    // Keep the upper frequency bound above the one-hertz lower bound even when an invalid sample
    // rate reaches this defensive layer.
    let sample_rate = sample_rate.max(4.0);
    let frequency_hz = frequency_hz.clamp(1.0, sample_rate * 0.499);
    let gain_db = gain_db.clamp(-48.0, 48.0);
    let omega = 2.0 * PI * frequency_hz / sample_rate;
    let a = 10.0_f32.powf(gain_db / 40.0);

    (omega, a)
}

fn shelf_design_values(sample_rate: f32, frequency_hz: f32, gain_db: f32) -> (f32, f32, f32) {
    let (omega, a) = common_design_values(sample_rate, frequency_hz, gain_db);
    // RBJ's shelf-slope parameter. A value of one gives the steepest monotonic transition and
    // keeps the shelf gain intuitive at its low/high-frequency asymptote.
    let slope = 1.0;
    let alpha = (omega.sin() / 2.0) * ((a + 1.0 / a) * (1.0 / slope - 1.0) + 2.0).sqrt();

    (omega, alpha, a)
}

#[cfg(test)]
mod tests {
    use super::{Biquad, BiquadCoefficients};

    const SAMPLE_RATE: f32 = 48_000.0;

    #[test]
    fn zero_gain_filters_are_unity() {
        let filters = [
            BiquadCoefficients::low_shelf(SAMPLE_RATE, 120.0, 0.0),
            BiquadCoefficients::peaking(SAMPLE_RATE, 1_000.0, 0.0, 1.0),
            BiquadCoefficients::high_shelf(SAMPLE_RATE, 8_000.0, 0.0),
        ];

        for coefficients in filters {
            let mut filter = Biquad::default();
            filter.set_coefficients(coefficients);
            let impulse = filter.process(1.0);
            assert!((impulse - 1.0).abs() < 1e-6);
            assert!(filter.process(0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn peaking_filter_has_expected_center_gain() {
        let coefficients = BiquadCoefficients::peaking(SAMPLE_RATE, 1_000.0, 12.0, 1.0);
        let gain_db = 20.0 * coefficients.magnitude_at(SAMPLE_RATE, 1_000.0).log10();

        assert!((gain_db - 12.0).abs() < 0.01, "got {gain_db} dB");
    }

    #[test]
    fn shelves_have_expected_asymptotic_gain() {
        let low = BiquadCoefficients::low_shelf(SAMPLE_RATE, 120.0, 12.0);
        let high = BiquadCoefficients::high_shelf(96_000.0, 8_000.0, 12.0);
        let low_gain_db = 20.0 * low.magnitude_at(SAMPLE_RATE, 10.0).log10();
        let high_gain_db = 20.0 * high.magnitude_at(96_000.0, 40_000.0).log10();

        assert!((low_gain_db - 12.0).abs() < 0.1, "got {low_gain_db} dB");
        assert!((high_gain_db - 12.0).abs() < 0.1, "got {high_gain_db} dB");
    }

    #[test]
    fn filters_remain_finite_at_parameter_boundaries() {
        let filters = [
            BiquadCoefficients::low_shelf(1.0, 0.0, -100.0),
            BiquadCoefficients::peaking(1.0, 10_000.0, 100.0, 0.0),
            BiquadCoefficients::high_shelf(1.0, f32::INFINITY, 100.0),
        ];

        for coefficients in filters {
            let mut filter = Biquad::default();
            filter.set_coefficients(coefficients);
            let mut sample = 1.0;
            for _ in 0..128 {
                sample = filter.process(sample);
                assert!(sample.is_finite());
            }
        }
    }
}
