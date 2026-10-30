//! Original aux harmonic generator: filtered nonlinear residual, no dry sum.
//! Two cascaded 2x polyphase half-band stages surround the nonlinearity (4x).
//! FIRs are 17-tap Hamming-windowed sinc, normalized to unity DC gain.
use crate::{dsp::DelaySample, model::ExciterConfig};

const ODD: [f64; 8] = [
    -0.005248316,
    0.023251573,
    -0.07623855,
    0.3082353,
    0.3082353,
    -0.07623855,
    0.023251573,
    -0.005248316,
];

#[derive(Clone, Copy)]
pub(crate) struct Parameters<S: DelaySample> {
    pub tune_hz: S,
    pub drive: S,
    pub tone: S,
    pub bright: bool,
}
impl From<ExciterConfig> for Parameters<f32> {
    fn from(c: ExciterConfig) -> Self {
        Self {
            tune_hz: c.tune_hz,
            drive: c.drive,
            tone: c.tone,
            bright: c.bright,
        }
    }
}

#[derive(Default)]
struct Up<S: DelaySample = f32> {
    history: [S; 8],
    pos: usize,
}
impl<S: DelaySample> Up<S> {
    fn tick(&mut self, input: S) -> [S; 2] {
        self.history[self.pos] = input;
        let even = self.history[(self.pos + 4) & 7];
        let odd = ODD
            .iter()
            .enumerate()
            .map(|(i, c)| S::from_f64(*c) * self.history[(self.pos + 8 - i) & 7])
            .sum::<S>()
            * S::from_f64(2.0);
        self.pos = (self.pos + 1) & 7;
        [even, odd]
    }
}
#[derive(Default)]
struct Down<S: DelaySample = f32> {
    even: [S; 8],
    odd: [S; 8],
    pos: usize,
}
impl<S: DelaySample> Down<S> {
    fn tick(&mut self, input: [S; 2]) -> S {
        self.even[self.pos] = input[0];
        self.odd[self.pos] = input[1];
        let out = self.odd[(self.pos + 4) & 7] * S::from_f64(0.5)
            + ODD
                .iter()
                .enumerate()
                .map(|(i, c)| S::from_f64(*c) * self.even[(self.pos + 8 - i) & 7])
                .sum::<S>();
        self.pos = (self.pos + 1) & 7;
        out
    }
}
#[derive(Default)]
struct Channel<S: DelaySample = f32> {
    up: [Up<S>; 2],
    down: [Down<S>; 2],
    input_low: [S; 2],
    output_low: [S; 2],
    dc: S,
}
fn lowpass<S: DelaySample>(state: &mut S, input: S, coefficient: S) -> S {
    *state += coefficient * (input - *state);
    if state.abs() < S::from_f64(1e-20) {
        *state = S::from_f64(0.0);
    }
    *state
}
fn residual<S: DelaySample>(input: S, gain: S, bright: bool) -> S {
    let x = input * gain;
    let square = x * x;
    let bend = square / (S::from_f64(1.0) + square);
    // No linear term. Warm adds even harmonics with some odd colour; Bright
    // emphasizes odd harmonics. This is a residual, not a full distorted copy.
    if bright {
        bend * (S::from_f64(0.1) - S::from_f64(0.7) * x) / gain
    } else {
        bend * (S::from_f64(0.8) - S::from_f64(0.15) * x) / gain
    }
}
pub(crate) struct Exciter<S: DelaySample = f32> {
    channels: [Channel<S>; 2],
    rate: S,
    coefficient: [S; 3],
    target: [S; 3],
    control_phase: u8,
}
impl<S: DelaySample> Exciter<S> {
    pub(crate) fn new(rate: S, config: impl Into<Parameters<S>>) -> Self {
        let config = config.into();
        let coefficient = Self::coefficients(rate, config);
        Self {
            channels: std::array::from_fn(|_| Channel::default()),
            rate,
            coefficient,
            target: coefficient,
            control_phase: 0,
        }
    }
    fn coefficients(rate: S, config: Parameters<S>) -> [S; 3] {
        [
            config.tune_hz.min(rate * S::from_f64(0.2)),
            (S::from_f64(2000.0) + config.tone * S::from_f64(16000.0)).min(rate * S::from_f64(0.4)),
            S::from_f64(20.0),
        ]
        .map(|hz| {
            S::from_f64(1.0) - ((S::ZERO - S::from_f64(std::f64::consts::TAU)) * hz / rate).exp()
        })
    }
    pub(crate) fn clear(&mut self) {
        self.channels = std::array::from_fn(|_| Channel::default());
        self.control_phase = 0;
    }
    pub(crate) fn tick(&mut self, input: [S; 2], config: impl Into<Parameters<S>>) -> [S; 2] {
        let config = config.into();
        // Expensive coefficient preparation is bounded to once per 32 samples;
        // interpolation plus the owning slot's parameter slew prevents steps.
        if self.control_phase == 0 {
            self.target = Self::coefficients(self.rate, config);
        }
        self.control_phase = (self.control_phase + 1) & 31;
        for (c, target) in self.coefficient.iter_mut().zip(self.target) {
            *c += (target - *c) / S::from_f64(32.0);
        }
        let gain = S::from_f64(1.0) + config.drive * S::from_f64(15.0);
        std::array::from_fn(|c| {
            let channel = &mut self.channels[c];
            let mut band = input[c];
            for low in &mut channel.input_low {
                band -= lowpass(low, band, self.coefficient[0]);
            }
            let high_rate = channel.up[0].tick(band).map(|sample| {
                let excited = channel.up[1]
                    .tick(sample)
                    .map(|s| residual(s, gain, config.bright));
                channel.down[1].tick(excited)
            });
            let mut wet = channel.down[0].tick(high_rate);
            wet -= lowpass(&mut channel.dc, wet, self.coefficient[2]);
            for low in &mut channel.output_low {
                wet = lowpass(low, wet, self.coefficient[1]);
            }
            // Exact silence at zero Drive, even while filter history drains.
            wet * config.drive
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_f64_oversampling_reduces_folded_harmonics() {
        let mut up: [Up<f64>; 2] = std::array::from_fn(|_| Up::default());
        let mut down: [Down<f64>; 2] = std::array::from_fn(|_| Down::default());
        let mut direct = Vec::new();
        let mut filtered = Vec::new();
        for n in 0..8192 {
            let x = (std::f64::consts::TAU * 853.0 * n as f64 / 4096.0).sin() * 0.7;
            let at_two = up[0]
                .tick(x)
                .map(|s| down[1].tick(up[1].tick(s).map(|v| residual(v, 5.0, true))));
            let y = down[0].tick(at_two);
            if n >= 4096 {
                direct.push(residual(x, 5.0, true));
                filtered.push(y);
            }
        }
        let bin = |samples: &[f64], k: usize| {
            let (mut re, mut im) = (0.0, 0.0);
            for (n, x) in samples.iter().enumerate() {
                let phase = std::f64::consts::TAU * (k * n) as f64 / samples.len() as f64;
                re += x * phase.cos();
                im += x * phase.sin();
            }
            re.hypot(im)
        };
        for alias in [1537, 169] {
            assert!(bin(&filtered, alias) < bin(&direct, alias) * 0.15);
        }
    }
    fn bin(samples: &[f32], k: usize) -> f32 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, x) in samples.iter().enumerate() {
            let phase = std::f32::consts::TAU * (k * n) as f32 / samples.len() as f32;
            re += x * phase.cos();
            im += x * phase.sin();
        }
        (re * re + im * im).sqrt() / samples.len() as f32
    }
    #[test]
    fn oversampling_reduces_folded_harmonics() {
        let mut up: [Up; 2] = std::array::from_fn(|_| Up::default());
        let mut down: [Down; 2] = std::array::from_fn(|_| Down::default());
        let mut direct = Vec::new();
        let mut filtered = Vec::new();
        for n in 0..8192 {
            let x = (std::f32::consts::TAU * 853.0 * n as f32 / 4096.0).sin() * 0.7;
            let at_two = up[0]
                .tick(x)
                .map(|s| down[1].tick(up[1].tick(s).map(|v| residual(v, 5.0, true))));
            let y = down[0].tick(at_two);
            if n >= 4096 {
                direct.push(residual(x, 5.0, true));
                filtered.push(y);
            }
        }
        for alias in [1537, 169] {
            // folds of the third and fifth harmonics
            let raw = bin(&direct, alias);
            let antialiased = bin(&filtered, alias);
            assert!(
                antialiased < raw * 0.15,
                "bin {alias}: {antialiased} vs {raw}"
            );
        }
    }
    fn render(amplitude: f32, bright: bool) -> Vec<f32> {
        let config = ExciterConfig {
            tune_hz: 600.0,
            drive: 0.6,
            tone: 1.0,
            bright,
        };
        let mut effect = Exciter::new(48000.0, config);
        (0..8192)
            .filter_map(|n| {
                let x = (std::f32::consts::TAU * 128.0 * n as f32 / 4096.0).sin() * amplitude;
                let out = effect.tick([x, 0.0], config);
                assert_eq!(out[1], 0.0, "stereo channel leak");
                (n >= 4096).then_some(out[0])
            })
            .collect()
    }
    #[test]
    fn exciter_adds_distinct_harmonics_without_a_linear_dry_branch() {
        let warm = render(0.2, false);
        let bright = render(0.2, true);
        assert!(bin(&warm, 256) > bin(&warm, 384) * 2.0);
        assert!(bin(&bright, 384) > bin(&bright, 256) * 2.0);
        let small = render(0.002, false);
        let half = render(0.001, false);
        let energy = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>();
        // Quadratic enhancement falls by ~12 dB when input halves. A leaked
        // linear dry copy would only fall by 6 dB and dominate at small levels.
        assert!(energy(&half) < energy(&small) * 0.08);
    }
}
