use shr_fx::c_api_v2::*;
use std::{ffi::c_void, ptr};
fn defaults(a: u32) -> Settings {
    let mut s = Settings::default();
    assert_eq!(
        unsafe { shr_fx_v2_defaults(a, &mut s, 2, size_of::<Settings>() as u32) },
        0
    );
    s
}
fn status(h: *mut c_void) -> Status {
    let mut s = Status::default();
    assert_eq!(
        unsafe { shr_fx_v2_status(h, &mut s, 2, size_of::<Status>() as u32) },
        0
    );
    s
}
fn render(h: *mut c_void, input: &[f64]) -> Vec<f64> {
    let mut out = vec![99.0; input.len()];
    assert_eq!(
        unsafe {
            shr_fx_v2_process(
                h,
                input.as_ptr(),
                out.as_mut_ptr(),
                (input.len() / 2) as u32,
            )
        },
        0
    );
    out
}
#[test]
fn delay_independent_samples_partitions_precision_and_feedback() {
    unsafe {
        let mut s = defaults(1);
        s.time_ms = 10.0;
        s.damping = 0.0;
        s.gain = 1.0;
        s.amount = 0.5;
        let h = shr_fx_v2_create(8000, 8192, &s);
        let split = shr_fx_v2_create(8000, 8192, &s);
        let mut input = vec![0.0; 1600];
        input[0] = 0.5 + 2f64.powi(-40);
        input[3] = -0.25;
        let out = render(h, &input);
        for n in 0..800 {
            for c in 0..2 {
                let expected = if c == 0 && n >= 80 && n % 80 == 0 {
                    input[0] * 0.5f64.powi((n / 80 - 1) as i32)
                } else if c == 1 && n >= 81 && (n - 1) % 80 == 0 {
                    -0.25 * 0.5f64.powi(((n - 1) / 80 - 1) as i32)
                } else {
                    0.0
                };
                assert_eq!(out[n * 2 + c], expected)
            }
        }
        let actual: Vec<_> = input.chunks(14).flat_map(|x| render(split, x)).collect();
        assert_eq!(out, actual);
        shr_fx_v2_destroy(h);
        shr_fx_v2_destroy(split);
    }
}
#[test]
fn requests_backpressure_cancel_transition_reset_and_fault_isolation() {
    unsafe {
        let s = defaults(1);
        let h = shr_fx_v2_create(8000, 512, &s);
        let healthy = shr_fx_v2_create(8000, 512, &s);
        let mut t = s;
        t.time_ms = 1.0;
        let p = shr_fx_v2_prepare(8000, 512, &t);
        assert_eq!(shr_fx_v2_publish(h, p, 1, 1), -4);
        assert_eq!(status(h).current, s);
        assert_eq!(shr_fx_v2_publish(h, p, 1, 0), 0);
        assert_eq!(status(h).target, t);
        assert_eq!(status(h).revision, 0);
        let other = shr_fx_v2_prepare(8000, 512, &s);
        assert_eq!(shr_fx_v2_publish(h, other, 2, 0), -2);
        render(h, &[0.0; 320]);
        assert_eq!(status(h).revision, 1);
        assert_eq!(status(h).applied_request, 1);
        assert_eq!(status(h).last_request, 2);
        assert_eq!(status(h).request_result, -2);
        assert_eq!(status(h).request_state, 5);
        assert_eq!(shr_fx_v2_publish(h, other, 2, 1), -2);
        shr_fx_v2_cancel(shr_fx_v2_retire(h));
        assert_eq!(shr_fx_v2_publish(h, other, 1, 1), -5);
        assert_eq!(shr_fx_v2_publish(h, other, 2, 1), 0);
        assert_eq!(shr_fx_v2_reset(h), 0);
        shr_fx_v2_cancel(shr_fx_v2_retire(h));
        assert_eq!(status(h).current, t);
        let mut x = [0.0; 400];
        x[0] = 1.0;
        assert!(render(h, &x).iter().any(|x| *x != 0.0));
        assert!(render(healthy, &x).iter().any(|x| *x != 0.0));
        let mut out = [9.0; 2];
        assert_eq!(
            shr_fx_v2_process(h, [f64::NAN, 0.0].as_ptr(), out.as_mut_ptr(), 1),
            -3
        );
        assert_eq!(out, [0.0; 2]);
        assert_eq!(shr_fx_v2_process(h, ptr::null(), ptr::null_mut(), 0), 0);
        assert_eq!(status(h).fault, 1);
        assert!(render(healthy, &[0.0; 400]).iter().any(|x| *x != 0.0));
        shr_fx_v2_reset(h);
        assert!(render(h, &[0.0; 400]).iter().all(|x| *x == 0.0));
        shr_fx_v2_destroy(h);
        shr_fx_v2_destroy(healthy);
    }
}
#[test]
fn room_and_chorus_native_precision_determinism_bypass_tails() {
    unsafe {
        for a in [2, 3] {
            let mut s = defaults(a);
            s.gain = 1.0;
            let h = shr_fx_v2_create(8000, 8192, &s);
            let twin = shr_fx_v2_create(8000, 8192, &s);
            let mut input = vec![0.0; 8192];
            input[0] = 0.5 + 2f64.powi(-40);
            let out = render(h, &input);
            assert_eq!(out, render(twin, &input));
            assert!(out.iter().all(|x| x.is_finite()));
            assert!(out.chunks_exact(2).all(|x| x[1] == 0.0));
            assert!(out.iter().any(|x| *x != 0.0 && *x != (*x as f32) as f64));
            shr_fx_v2_reset(h);
            shr_fx_v2_reset(twin);
            render(h, &[1.0, 0.0]);
            render(twin, &[1.0, 0.0]);
            s.bypass = 1;
            let p = shr_fx_v2_prepare(8000, 8192, &s);
            assert_eq!(shr_fx_v2_publish(h, p, 1, 0), 0);
            let tail = render(h, &[0.0; 8192]);
            assert_eq!(tail, render(twin, &[0.0; 8192]));
            assert!(tail.iter().any(|x| *x != 0.0));
            shr_fx_v2_cancel(shr_fx_v2_retire(h));
            shr_fx_v2_destroy(h);
            shr_fx_v2_destroy(twin);
        }
    }
}
#[test]
fn malformed_shapes_settings_and_in_place() {
    unsafe {
        let mut s = defaults(3);
        s.amount = 20.0;
        assert_eq!(shr_fx_v2_validate(&s), -6);
        assert!(shr_fx_v2_prepare(8000, 10, &s).is_null());
        s = defaults(1);
        let h = shr_fx_v2_create(8000, 10, &s);
        let mut out = [4.0; 22];
        assert_eq!(shr_fx_v2_process(h, ptr::null(), out.as_mut_ptr(), 11), -2);
        assert_eq!(out, [4.0; 22]);
        assert_eq!(
            shr_fx_v2_process(h, out.as_ptr(), out.as_mut_ptr().add(1), 10),
            -1
        );
        assert_eq!(out, [4.0; 22]);
        assert_eq!(
            shr_fx_v2_status(h, h.cast(), 2, size_of::<Status>() as u32),
            -1
        );
        assert_eq!(shr_fx_v2_process(h, out.as_ptr(), out.as_mut_ptr(), 10), 0);
        assert!(out[..20].iter().all(|x| *x == 0.0));
        shr_fx_v2_destroy(h);
    }
}

#[test]
fn modulation_matches_independent_parabolic_fractional_tap_reference() {
    unsafe {
        let s = defaults(3);
        let h = shr_fx_v2_create(8000, 8192, &s);
        let mut input = vec![0.0; 4000];
        input[0] = 0.5 + 2f64.powi(-40);
        let out = render(h, &input);
        let mut phase = s.seed as f64 / 4294967296.0;
        for n in 0..2000 {
            phase += s.rate_hz / 8000.0;
            if phase >= 1.0 {
                phase -= 1.0
            }
            let x = phase * 2.0 - 1.0;
            let osc = 4.0 * x * (1.0 - x.abs());
            let delay = (s.time_ms + s.amount * osc) * 8.0;
            let integral = delay as usize;
            let tap = |d: usize| {
                if d > 0 && d <= n {
                    input[(n - d) * 2]
                } else {
                    0.0
                }
            };
            let expected = (tap(integral)
                + (tap(integral + 1) - tap(integral)) * (delay - integral as f64))
                * s.gain;
            assert!((out[n * 2] - expected).abs() < 1e-17);
        }
        shr_fx_v2_destroy(h);
    }
}

#[test]
fn transitions_use_source_frames_and_interruption_reports_current_exactly() {
    unsafe {
        let s = defaults(1);
        let h = shr_fx_v2_create(8000, 512, &s);
        let mut target = s;
        target.time_ms = 1.0;
        let p = shr_fx_v2_prepare(8000, 512, &target);
        assert_eq!(shr_fx_v2_publish(h, p, 10, 0), 0);
        render(h, &[0.0; 158]);
        assert_eq!(status(h).current, s);
        assert_eq!(status(h).request_state, 1);
        render(h, &[0.0; 2]);
        assert_eq!(status(h).current, target);
        assert_eq!(status(h).request_state, 2);
        assert_eq!(status(h).applied_request, 0);
        assert_eq!(shr_fx_v2_reset(h), 0);
        assert_eq!(status(h).request_state, 4);
        assert_eq!(status(h).current, target);
        shr_fx_v2_cancel(shr_fx_v2_retire(h));
        let p = shr_fx_v2_prepare(44100, 512, &s);
        assert_eq!(shr_fx_v2_publish(h, p, 11, 10), -6);
        assert_eq!(status(h).last_request, 11);
        assert_eq!(status(h).request_state, 5);
        shr_fx_v2_cancel(p);
        shr_fx_v2_destroy(h);
    }
}

#[test]
fn finite_above_unity_input_and_extreme_settings_stay_supported() {
    unsafe {
        for algorithm in 1..=10 {
            let mut s = defaults(algorithm);
            s.gain = 1.0;
            if matches!(algorithm, DELAY | TAPE) {
                s.amount = 0.9;
                s.time_ms = 1.0;
            }
            if matches!(algorithm, ROOM | SMALL_ROOM | CHAMBER | PLATE | HALL) {
                s.amount = 0.9;
                s.damping = 1.0;
            }
            if algorithm == 3 {
                s.time_ms = 30.0;
                s.amount = 9.0;
                s.rate_hz = 10.0;
            }
            if matches!(algorithm, EXCITER_WARM | EXCITER_BRIGHT) {
                s.amount = 1.0;
                s.damping = 1.0;
                s.rate_hz = 6000.0;
            }
            for rate in [8000, 48000, 192000] {
                let h = shr_fx_v2_create(rate, 8192, &s);
                assert!(!h.is_null());
                let input = [16.0; 16384];
                let out = render(h, &input);
                assert!(out.iter().all(|x| x.is_finite() && x.abs() <= 256.0));
                assert_eq!(status(h).fault, 0);
                shr_fx_v2_destroy(h);
            }
        }
    }
}

#[test]
fn palette_descriptors_defaults_bounds_and_unavailable_fields_agree() {
    unsafe {
        let mut caps = Capabilities::default();
        assert_eq!(
            shr_fx_v2_capabilities(&mut caps, 2, size_of::<Capabilities>() as u32),
            0
        );
        assert_eq!(caps.algorithm_mask, ALGORITHM_MASK);
        for algorithm in 1..=10 {
            let s = defaults(algorithm);
            assert_eq!(shr_fx_v2_validate(&s), 0);
            for id in 1..=6 {
                let mut p = Parameter::default();
                assert_eq!(
                    shr_fx_v2_parameter(algorithm, id, &mut p, 2, size_of::<Parameter>() as u32),
                    0
                );
                let value = [
                    s.time_ms,
                    s.amount,
                    s.damping,
                    s.rate_hz,
                    s.gain,
                    s.bypass as f64,
                ][id as usize - 1];
                assert_eq!(value, p.default_value);
                assert!((p.minimum..=p.maximum).contains(&value));
                if id <= 5 {
                    for invalid in [f64::NAN, p.minimum - 1.0, p.maximum + 1.0] {
                        let mut bad = s;
                        match id {
                            1 => bad.time_ms = invalid,
                            2 => bad.amount = invalid,
                            3 => bad.damping = invalid,
                            4 => bad.rate_hz = invalid,
                            _ => bad.gain = invalid,
                        }
                        assert_eq!(shr_fx_v2_validate(&bad), -6);
                        assert!(shr_fx_v2_prepare(48000, 48, &bad).is_null());
                    }
                }
            }
        }
        let mut p = Parameter::default();
        assert_eq!(
            shr_fx_v2_parameter(11, 1, &mut p, 2, size_of::<Parameter>() as u32),
            -6
        );
    }
}

#[test]
fn musical_divisions_are_exact_or_explicitly_out_of_capacity() {
    unsafe {
        for bpm in [30.0, 120.0, 300.0] {
            for denominator in [1, 2, 4, 8, 16, 32, 64] {
                for (modifier, factor) in [(0, 1.0), (1, 1.5), (2, 2.0 / 3.0)] {
                    let mut ms = -99.0;
                    let expected = 240_000.0 / bpm / denominator as f64 * factor;
                    let result = shr_fx_v2_delay_time(bpm, denominator, modifier, &mut ms);
                    if expected > 2000.0 {
                        assert_eq!(result, -2);
                        assert_eq!(ms, -99.0);
                    } else {
                        assert_eq!(result, 0);
                        assert_eq!(ms, expected);
                    }
                }
            }
        }
        for (bpm, denominator, modifier) in [
            (f64::NAN, 4, 0),
            (29.0, 4, 0),
            (301.0, 4, 0),
            (120.0, 3, 0),
            (120.0, 4, 3),
        ] {
            let mut ms = 99.0;
            assert_eq!(
                shr_fx_v2_delay_time(bpm, denominator, modifier, &mut ms),
                -6
            );
            assert_eq!(ms, 99.0);
        }
        assert_eq!(shr_fx_v2_delay_time(120.0, 4, 0, ptr::null_mut()), -1);
        // Render a dotted eighth at 120 BPM: 375 ms, 3000 frames at 8 kHz.
        let mut s = defaults(DELAY);
        assert_eq!(shr_fx_v2_delay_time(120.0, 8, 1, &mut s.time_ms), 0);
        s.amount = 0.0;
        s.gain = 1.0;
        let h = shr_fx_v2_create(8000, 8192, &s);
        let mut input = vec![0.0; 8192];
        input[0] = 1.0;
        let out = render(h, &input);
        assert!(out[..6000].iter().all(|x| *x == 0.0));
        assert_eq!(out[6000], 1.0);
        shr_fx_v2_destroy(h);
    }
}

#[test]
fn tape_echo_matches_independent_saturated_modulated_first_tap() {
    unsafe {
        let mut s = defaults(TAPE);
        s.time_ms = 10.0;
        s.amount = 0.0;
        s.gain = 1.0;
        let h = shr_fx_v2_create(8000, 512, &s);
        let mut input = vec![0.0; 1024];
        input[0] = 0.5 + 2f64.powi(-40);
        let out = render(h, &input);
        let seed = s.seed as f64 / 4294967296.0;
        let mut phase = [seed, (seed + 0.31).fract()];
        let stored = input[0] / (1.0 + 0.12 * input[0].abs());
        for n in 0..512 {
            let mut oscillator = [0.0; 2];
            for i in 0..2 {
                phase[i] += [0.37, 6.1][i] / 8000.0;
                if phase[i] >= 1.0 {
                    phase[i] -= 1.0;
                }
                let x = phase[i] * 2.0 - 1.0;
                oscillator[i] = 4.0 * x * (1.0 - x.abs());
            }
            let delay = 80.0 + (oscillator[0] * 0.35 + oscillator[1] * 0.08) * 8.0;
            let integral = delay as usize;
            let a = if integral == n { stored } else { 0.0 };
            let b = if integral + 1 == n { stored } else { 0.0 };
            let expected = a + (b - a) * (delay - integral as f64);
            assert!((out[n * 2] - expected).abs() < 1e-14);
            assert_eq!(out[n * 2 + 1], 0.0);
        }
        assert!(out.iter().any(|x| *x != 0.0));
        shr_fx_v2_reset(h);
        assert_eq!(out, render(h, &input));
        shr_fx_v2_destroy(h);
    }
}

#[test]
fn reverb_variants_have_independent_onsets_stereo_spread_and_darkening() {
    unsafe {
        // Shortest comb at 8kHz plus the minimum one-frame predelay.
        for (algorithm, onset, spread) in [
            (ROOM, 239, false),
            (SMALL_ROOM, 91, false),
            (CHAMBER, 250, true),
            (PLATE, 139, true),
            (HALL, 543, true),
        ] {
            let mut s = defaults(algorithm);
            s.gain = 1.0;
            s.damping = 0.15;
            let h = shr_fx_v2_create(8000, 8192, &s);
            let mut input = vec![0.0; 16384];
            input[0] = 0.5 + 2f64.powi(-40);
            let bright = render(h, &input);
            assert!(bright[..onset * 2].iter().all(|x| *x == 0.0));
            let feedback = match algorithm {
                ROOM => 0.7,
                SMALL_ROOM => 0.57,
                CHAMBER => 0.73,
                PLATE => 0.76,
                _ => 0.77,
            };
            let input_diffusion = if spread { 0.85 * 0.25 } else { 1.0 };
            let output_diffusion = if matches!(algorithm, PLATE | HALL) {
                0.0625 * 0.65
            } else {
                0.25
            };
            let expected = input[0] * input_diffusion * (1.0 - feedback) * 0.25 * output_diffusion;
            assert!(
                (bright[onset * 2] - expected).abs() < 1e-16,
                "algorithm {algorithm}"
            );
            assert_eq!(bright.chunks_exact(2).any(|x| x[1] != 0.0), spread);
            assert!(bright.iter().any(|x| *x != (*x as f32) as f64));
            s.damping = 0.85;
            let dark = shr_fx_v2_create(8000, 8192, &s);
            let darker = render(dark, &input);
            // Normalized first-difference energy in the feedback tail measures
            // high-frequency content without treating output gain as darkness.
            let roughness = |v: &[f64]| {
                let v = &v[4000..];
                v.windows(3)
                    .step_by(2)
                    .map(|w| (w[2] - w[0]).powi(2))
                    .sum::<f64>()
                    / v.iter().step_by(2).map(|x| x * x).sum::<f64>()
            };
            assert!(
                roughness(&darker) < roughness(&bright),
                "algorithm {algorithm}"
            );
            shr_fx_v2_destroy(h);
            shr_fx_v2_destroy(dark);
        }
    }
}

#[test]
fn expanded_palette_partitions_reset_faults_and_rate_bounds() {
    unsafe {
        for algorithm in 4..=10 {
            for rate in [8000, 48000, 192000] {
                let s = defaults(algorithm);
                let h = shr_fx_v2_create(rate, 8192, &s);
                let split = shr_fx_v2_create(rate, 8192, &s);
                let mut input = vec![0.0; 16384];
                input[0] = 0.5 + 2f64.powi(-40);
                let out = render(h, &input);
                let actual: Vec<_> = input.chunks(94).flat_map(|x| render(split, x)).collect();
                assert_eq!(out, actual, "algorithm {algorithm}, rate {rate}");
                shr_fx_v2_reset(h);
                assert_eq!(out, render(h, &input));
                let mut fault = [99.0; 2];
                assert_eq!(
                    shr_fx_v2_process(h, [f64::NAN, 0.0].as_ptr(), fault.as_mut_ptr(), 1),
                    -3
                );
                assert_eq!(fault, [0.0; 2]);
                assert_eq!(status(h).fault, 1);
                shr_fx_v2_reset(h);
                assert_eq!(out, render(h, &input));
                shr_fx_v2_destroy(h);
                shr_fx_v2_destroy(split);
            }
        }
    }
}

fn exciter_tone(algorithm: u32, amplitude: f64, drive: f64) -> Vec<f64> {
    unsafe {
        let mut s = defaults(algorithm);
        s.rate_hz = 600.0;
        s.amount = drive;
        s.damping = 1.0;
        s.gain = 1.0;
        let h = shr_fx_v2_create(48000, 8192, &s);
        let input: Vec<_> = (0..8192)
            .flat_map(|n| {
                [
                    (std::f64::consts::TAU * 128.0 * n as f64 / 4096.0).sin() * amplitude,
                    0.0,
                ]
            })
            .collect();
        let out = render(h, &input);
        assert!(out.chunks_exact(2).all(|x| x[1] == 0.0));
        shr_fx_v2_destroy(h);
        out[8192..].iter().step_by(2).copied().collect()
    }
}
#[test]
fn native_exciter_adds_harmonics_without_linear_dry_or_zero_drive_output() {
    let bin = |v: &[f64], k: usize| {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, x) in v.iter().enumerate() {
            let phase = std::f64::consts::TAU * (k * n) as f64 / v.len() as f64;
            re += x * phase.cos();
            im += x * phase.sin();
        }
        re.hypot(im) / v.len() as f64
    };
    let warm = exciter_tone(EXCITER_WARM, 0.2, 0.6);
    let bright = exciter_tone(EXCITER_BRIGHT, 0.2, 0.6);
    assert!(bin(&warm, 256) > 2.0 * bin(&warm, 384));
    assert!(bin(&bright, 384) > 2.0 * bin(&bright, 256));
    for algorithm in [EXCITER_WARM, EXCITER_BRIGHT] {
        let small = exciter_tone(algorithm, 0.002, 0.6);
        let half = exciter_tone(algorithm, 0.001, 0.6);
        let energy = |v: &[f64]| v.iter().map(|x| x * x).sum::<f64>();
        assert!(energy(&half) < energy(&small) * 0.08);
        assert!(small.iter().any(|x| *x != (*x as f32) as f64));
        assert!(exciter_tone(algorithm, 0.5, 0.0).iter().all(|x| *x == 0.0));
    }
}
