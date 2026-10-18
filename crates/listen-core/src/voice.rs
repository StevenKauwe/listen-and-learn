//! Sung note names: a small formant singer, rendered sample by sample so
//! the browser and the native tests run the very same code.
//!
//! Source: a band-limited glottal-like pulse train (harmonics summed by
//! Chebyshev recurrence, so nothing aliases) at the note's exact pitch,
//! with gentle delayed vibrato and a breath of aspiration noise. Filter: a
//! cascade of four formant resonators (Klatt-style, unity gain at 0 Hz),
//! whose frequencies glide from the consonant's locus into the vowel.
//! Consonants: filtered noise for f, s, t and the d burst; voiced murmurs
//! and formant glides for d, r, m, l.
//!
//! Vowel formants F1–F3 are the child means (boys and girls) of
//! Hillenbrand et al. (1995), steady-state values; F4 and all bandwidths
//! are synthesis choices. See docs/research.md.

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vowel {
    O,
    E,
    I,
    A,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Onset {
    D,
    R,
    M,
    F,
    S,
    L,
    T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Onset,
    pub vowel: Vowel,
    /// "Sol" closes on a light l.
    pub coda_l: bool,
}

const fn syl(onset: Onset, vowel: Vowel, coda_l: bool) -> Syllable {
    Syllable {
        onset,
        vowel,
        coda_l,
    }
}

/// Do Re Mi Fa Sol La Ti, indexed like the scale degrees (high Do = 0).
pub const SYLLABLES: [Syllable; 7] = [
    syl(Onset::D, Vowel::O, false),
    syl(Onset::R, Vowel::E, false),
    syl(Onset::M, Vowel::I, false),
    syl(Onset::F, Vowel::A, false),
    syl(Onset::S, Vowel::O, true),
    syl(Onset::L, Vowel::A, false),
    syl(Onset::T, Vowel::I, false),
];

/// F1..F4 in Hz. F1–F3: Hillenbrand et al. 1995, children (mean of the
/// boys' and girls' steady-state values for /o e i ɑ/). F4: chosen.
pub fn formants(v: Vowel) -> [f32; 4] {
    match v {
        Vowel::O => [598.0, 1141.0, 3021.0, 4100.0],
        Vowel::E => [565.0, 2658.0, 3335.0, 4400.0],
        Vowel::I => [454.0, 3094.0, 3704.0, 4600.0],
        Vowel::A => [999.0, 1694.0, 2949.0, 4100.0],
    }
}

/// Formant bandwidths in Hz (synthesis choice; a little wide = soft).
pub const BANDWIDTHS: [f32; 4] = [90.0, 130.0, 190.0, 260.0];

/// How a consonant starts the syllable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnsetSpec {
    /// Seconds of noise (frication, burst, aspiration) before voicing.
    pub noise_s: f32,
    /// Centre and Q of the noise band, and its level.
    pub noise_hz: f32,
    pub noise_q: f32,
    pub noise_gain: f32,
    /// Seconds the voiced consonant holds its locus (murmur) once voicing
    /// starts.
    pub hold_s: f32,
    /// Formants F1..F3 at the consonant.
    pub locus: [f32; 3],
    /// Seconds of the formant glide into the vowel.
    pub glide_s: f32,
    /// Voicing level during the hold (vowel = 1).
    pub amp: f32,
}

pub fn onset(o: Onset) -> OnsetSpec {
    let s = |noise_s, noise_hz, noise_q, noise_gain, hold_s, locus, glide_s, amp| OnsetSpec {
        noise_s,
        noise_hz,
        noise_q,
        noise_gain,
        hold_s,
        locus,
        glide_s,
        amp,
    };
    match o {
        // Voiced stop: short voice bar, then a burst rides on the release.
        Onset::D => s(
            0.0,
            4200.0,
            1.2,
            0.5,
            0.035,
            [280.0, 2200.0, 3100.0],
            0.05,
            0.12,
        ),
        // Approximant: low F3 is what makes it an r.
        Onset::R => s(
            0.0,
            0.0,
            1.0,
            0.0,
            0.05,
            [380.0, 1250.0, 1900.0],
            0.07,
            0.55,
        ),
        // Nasal murmur: low F1, weak upper formants.
        Onset::M => s(
            0.0,
            0.0,
            1.0,
            0.0,
            0.07,
            [280.0, 1200.0, 2700.0],
            0.04,
            0.35,
        ),
        // Fricatives: noise, then voicing with a labial / alveolar glide.
        Onset::F => s(
            0.09,
            4500.0,
            0.7,
            0.22,
            0.0,
            [450.0, 1200.0, 2700.0],
            0.05,
            0.6,
        ),
        Onset::S => s(
            0.11,
            7000.0,
            1.6,
            0.4,
            0.0,
            [450.0, 2000.0, 3100.0],
            0.05,
            0.6,
        ),
        Onset::L => s(0.0, 0.0, 1.0, 0.0, 0.05, [380.0, 1400.0, 3000.0], 0.06, 0.6),
        // Voiceless stop: burst plus aspiration, then the alveolar glide.
        Onset::T => s(
            0.05,
            5000.0,
            1.2,
            0.35,
            0.0,
            [450.0, 2300.0, 3200.0],
            0.05,
            0.6,
        ),
    }
}

/// The l at the end of "Sol".
const CODA_L: [f32; 3] = [380.0, 1100.0, 2900.0];
const CODA_S: f32 = 0.12;
/// Release fade at the end of every syllable.
pub const RELEASE_S: f32 = 0.16;
/// Vibrato: rate, depth (cents, peak), delay and ramp. Gentle and late,
/// like a child singing a held note.
pub const VIBRATO_HZ: f32 = 5.4;
pub const VIBRATO_CENTS: f32 = 22.0;
pub const VIBRATO_DELAY_S: f32 = 0.22;
pub const VIBRATO_RAMP_S: f32 = 0.3;
/// Breath noise mixed into the voice source.
const BREATH: f32 = 0.025;

/// When things happen inside one syllable of length `total` seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    /// Voicing starts (0 for voiced consonants).
    pub voice_on: f32,
    /// The vowel's target formants are reached.
    pub vowel_on: f32,
    /// The release fade starts.
    pub release_on: f32,
    pub total: f32,
}

/// Shortest syllable that still fits a consonant, a vowel and a release.
pub const MIN_DUR: f32 = 0.45;

pub fn plan(s: Syllable, dur: f32) -> Plan {
    let o = onset(s.onset);
    let total = dur.max(MIN_DUR);
    let voice_on = o.noise_s;
    let vowel_on = voice_on + o.hold_s + o.glide_s;
    Plan {
        voice_on,
        vowel_on,
        release_on: total - RELEASE_S,
        total,
    }
}

fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a + (b - a) * x.clamp(0.0, 1.0)
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Formant frequencies at time `t` (a chosen F1 never sits below the
/// fundamental: singers raise F1 to the pitch on high notes).
pub fn formants_at(s: Syllable, p: &Plan, f0: f32, t: f32) -> [f32; 4] {
    let o = onset(s.onset);
    let v = formants(s.vowel);
    let glide_start = p.voice_on + o.hold_s;
    let x = smooth((t - glide_start) / o.glide_s.max(1e-3));
    let mut f = [
        lerp(o.locus[0], v[0], x),
        lerp(o.locus[1], v[1], x),
        lerp(o.locus[2], v[2], x),
        v[3],
    ];
    if s.coda_l {
        let c = smooth((t - (p.total - CODA_S - 0.04)) / CODA_S);
        for i in 0..3 {
            f[i] = lerp(f[i], CODA_L[i], c);
        }
    }
    f[0] = f[0].max(f0 * 1.08);
    f
}

/// Voicing level at time `t` (0..1).
pub fn voice_amp(s: Syllable, p: &Plan, t: f32) -> f32 {
    let o = onset(s.onset);
    if t < p.voice_on || t >= p.total {
        return 0.0;
    }
    let since = t - p.voice_on;
    // Soft onset of the voice itself (no click).
    let on = smooth(since / 0.025);
    let glide_start = p.voice_on + o.hold_s;
    let level = lerp(o.amp, 1.0, smooth((t - glide_start) / o.glide_s.max(1e-3)));
    let coda = if s.coda_l {
        lerp(1.0, 0.65, smooth((t - (p.total - CODA_S - 0.04)) / CODA_S))
    } else {
        1.0
    };
    let release = 1.0 - smooth((t - p.release_on) / RELEASE_S);
    // A small swell through the vowel, like a sung note.
    let swell = 0.92 + 0.08 * smooth((t - p.vowel_on) / 0.25);
    on * level * coda * release * swell
}

/// Noise (consonant) level at time `t`.
pub fn noise_amp(s: Syllable, p: &Plan, t: f32) -> f32 {
    let o = onset(s.onset);
    match s.onset {
        // The d burst sits at the end of its voice bar.
        Onset::D => {
            let at = o.hold_s;
            let x = t - at;
            if (0.0..0.012).contains(&x) {
                o.noise_gain * (1.0 - x / 0.012)
            } else {
                0.0
            }
        }
        Onset::T => {
            // 10 ms burst, then softer aspiration up to voicing.
            if t < 0.01 {
                o.noise_gain
            } else if t < p.voice_on {
                o.noise_gain * 0.35
            } else {
                0.0
            }
        }
        Onset::F | Onset::S => {
            if t >= p.voice_on + 0.02 {
                return 0.0;
            }
            let rise = smooth(t / 0.03);
            let fall = 1.0 - smooth((t - (p.voice_on - 0.015)) / 0.035);
            o.noise_gain * rise * fall
        }
        _ => 0.0,
    }
}

/// Instantaneous pitch at `t`: the exact note, with gentle vibrato that
/// is symmetric in cents around it.
pub fn f0_at(f0: f32, t: f32) -> f32 {
    let depth = VIBRATO_CENTS * smooth((t - VIBRATO_DELAY_S) / VIBRATO_RAMP_S);
    let cents = depth * (std::f32::consts::TAU * VIBRATO_HZ * t).sin();
    f0 * (cents / 1200.0).exp2()
}

/// Two-pole resonator, unity gain at 0 Hz (Klatt 1980 form).
#[derive(Clone, Copy, Default)]
struct Reso {
    a: f32,
    b: f32,
    c: f32,
    y1: f32,
    y2: f32,
}

impl Reso {
    fn set(&mut self, f: f32, bw: f32, sr: f32) {
        let t = 1.0 / sr;
        let c = -(-2.0 * std::f32::consts::PI * bw * t).exp();
        let b = 2.0
            * (-std::f32::consts::PI * bw * t).exp()
            * (2.0 * std::f32::consts::PI * f * t).cos();
        self.a = 1.0 - b - c;
        self.b = b;
        self.c = c;
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.a * x + self.b * self.y1 + self.c * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// RBJ band-pass (0 dB peak) for consonant noise.
#[derive(Clone, Copy, Default)]
struct Band {
    b0: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Band {
    fn new(f: f32, q: f32, sr: f32) -> Band {
        let w = std::f32::consts::TAU * f.min(sr * 0.45) / sr;
        let alpha = w.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        Band {
            b0: alpha / a0,
            a1: -2.0 * w.cos() / a0,
            a2: (1.0 - alpha) / a0,
            ..Band::default()
        }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * (x - self.x2) - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Source spectral tilt: harmonic k has amplitude k^-TILT.
const TILT: f32 = 1.15;

/// Render one sung syllable at MIDI note `midi`, `dur` seconds long, as
/// mono samples peaking at 0.9.
pub fn render(s: Syllable, midi: f32, dur: f32, sr: f32) -> Vec<f32> {
    let f0 = crate::music::freq(midi);
    let p = plan(s, dur);
    let n = (p.total * sr) as usize;
    let o = onset(s.onset);
    let mut out = Vec::with_capacity(n);
    let mut rng = Rng::new(0x5EED ^ midi as u32);
    let mut noise = || (rng.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0;
    let mut res = [Reso::default(); 4];
    let mut band = Band::new(o.noise_hz.max(100.0), o.noise_q, sr);
    // Harmonics below 0.45 sr at the highest vibrato excursion.
    let top = f0 * (VIBRATO_CENTS / 1200.0).exp2();
    let k_max = ((0.45 * sr / top) as usize).clamp(1, 60);
    let amps: Vec<f32> = (1..=k_max)
        .map(|k| {
            // Fade the last few harmonics so the top edge is soft.
            let edge = ((k_max + 1 - k) as f32 / 4.0).min(1.0);
            (k as f32).powf(-TILT) * edge
        })
        .collect();
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / sr;
        if i % 16 == 0 {
            let f = formants_at(s, &p, f0, t);
            for (j, r) in res.iter_mut().enumerate() {
                r.set(f[j], BANDWIDTHS[j], sr);
            }
        }
        let va = voice_amp(s, &p, t);
        // Harmonic sum by Chebyshev recurrence: sin(kφ) from sin((k-1)φ).
        let w = std::f32::consts::TAU * phase;
        let (s1, c1) = w.sin_cos();
        let (mut sk1, mut sk2) = (s1, 0.0f32);
        let mut src = amps[0] * s1;
        for a in &amps[1..] {
            let sk = 2.0 * c1 * sk1 - sk2;
            src += a * sk;
            sk2 = sk1;
            sk1 = sk;
        }
        phase += f0_at(f0, t) / sr;
        phase -= phase.floor();
        let nz = noise();
        // Aspiration for t rides through the vocal tract like a whisper.
        let asp = if s.onset == Onset::T && t < p.voice_on && t >= 0.01 {
            0.6
        } else {
            0.0
        };
        let mut x = va * (src + BREATH * nz) + asp * nz;
        for r in res.iter_mut() {
            x = r.run(x);
        }
        let fric = band.run(nz) * noise_amp(s, &p, t);
        out.push(x + fric);
    }
    // Normalise on the voiced part, so consonants keep their balance.
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6);
    let g = 0.9 / peak;
    for v in out.iter_mut() {
        *v *= g;
    }
    // A few ms of fade at both ends: no clicks.
    let edge = ((0.004 * sr) as usize).min(n / 2);
    for i in 0..edge {
        let w = i as f32 / edge as f32;
        out[i] *= w;
        out[n - 1 - i] *= w;
    }
    out
}

/// Mean pitch over `samples` by autocorrelation with parabolic peak
/// interpolation, searched within ±1 semitone of `near` Hz.
pub fn measure_f0(samples: &[f32], sr: f32, near: f32) -> f32 {
    let lo = (sr / (near * 1.06)) as usize;
    let hi = (sr / (near / 1.06)) as usize + 1;
    let n = samples.len().saturating_sub(hi);
    let ac = |lag: usize| -> f32 {
        (0..n)
            .map(|i| samples[i] as f64 * samples[i + lag] as f64)
            .sum::<f64>() as f32
    };
    let mut best = (lo, f32::MIN);
    for lag in lo..=hi {
        let v = ac(lag);
        if v > best.1 {
            best = (lag, v);
        }
    }
    let (l, c) = (ac(best.0 - 1), best.1);
    let r = ac(best.0 + 1);
    let shift = 0.5 * (l - r) / (l - 2.0 * c + r);
    sr / (best.0 as f32 + shift)
}

/// 16-bit mono PCM WAV bytes (for listening to the singer natively).
pub fn wav(samples: &[f32], sr: u32) -> Vec<u8> {
    let n = samples.len() as u32;
    let mut b = Vec::with_capacity(44 + 2 * n as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + 2 * n).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(2 * n).to_le_bytes());
    for v in samples {
        b.extend_from_slice(&((v.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{freq, SCALE};

    const SR: f32 = 48000.0;

    #[test]
    fn every_syllable_fits_its_note_and_reaches_the_vowel_early() {
        for s in SYLLABLES {
            for dur in [MIN_DUR, 0.8, 1.2, 2.0] {
                let p = plan(s, dur);
                assert_eq!(p.total, dur);
                assert!(p.voice_on <= p.vowel_on && p.vowel_on < p.release_on);
                assert!(p.vowel_on <= 0.17, "{s:?}: vowel at {}", p.vowel_on);
                // Most of a sung name is vowel, as in singing.
                if dur >= 0.8 {
                    assert!((p.release_on - p.vowel_on) / dur >= 0.55);
                }
            }
        }
    }

    #[test]
    fn consonant_timings_follow_their_kind() {
        let voiced = [Onset::D, Onset::R, Onset::M, Onset::L];
        for o in voiced {
            assert_eq!(onset(o).noise_s, 0.0, "{o:?} voices from the start");
        }
        assert!(onset(Onset::S).noise_s >= 0.08, "s needs a real hiss");
        assert!(onset(Onset::F).noise_s >= 0.07);
        assert!((0.03..=0.07).contains(&onset(Onset::T).noise_s));
        assert!(onset(Onset::M).hold_s >= 0.05, "m needs a hum");
        assert!(onset(Onset::S).noise_hz > onset(Onset::F).noise_hz);
        assert!(onset(Onset::R).locus[2] < 2100.0, "r lowers F3");
    }

    #[test]
    fn formant_table_is_the_child_data_and_ordered() {
        assert_eq!(formants(Vowel::I)[..3], [454.0, 3094.0, 3704.0]);
        for v in [Vowel::O, Vowel::E, Vowel::I, Vowel::A] {
            let f = formants(v);
            assert!(f.windows(2).all(|w| w[0] < w[1]), "{v:?}");
        }
        // Front vowels have a high F2, back /o/ a low one.
        assert!(formants(Vowel::I)[1] > 2.5 * formants(Vowel::O)[1]);
    }

    #[test]
    fn f1_never_sits_below_the_pitch() {
        let s = SYLLABLES[6]; // Ti on B4
        let f0 = freq(71.0);
        let p = plan(s, 1.0);
        assert!(formants_at(s, &p, f0, 0.5)[0] >= f0);
    }

    #[test]
    fn vibrato_is_symmetric_so_the_mean_pitch_is_the_note() {
        let f0 = freq(64.0);
        // Five whole vibrato cycles once the vibrato is fully in.
        let n = (5.0 / VIBRATO_HZ * 48000.0) as usize;
        let mean_cents: f32 = (0..n)
            .map(|i| 1200.0 * (f0_at(f0, 0.6 + i as f32 / 48000.0) / f0).log2())
            .sum::<f32>()
            / n as f32;
        assert!(mean_cents.abs() < 0.1, "{mean_cents} cents");
        assert_eq!(f0_at(f0, 0.05), f0, "no vibrato at the start");
    }

    #[test]
    fn mi_is_sung_on_e4_and_every_name_on_its_own_note() {
        for (i, d) in SCALE.iter().enumerate() {
            let s = SYLLABLES[i % 7];
            let out = render(s, d.midi as f32, 1.2, SR);
            assert!(out.iter().all(|v| v.is_finite() && v.abs() <= 0.91));
            let p = plan(s, 1.2);
            let a = ((p.vowel_on + 0.05) * SR) as usize;
            let b = (p.release_on * SR) as usize;
            let want = freq(d.midi as f32);
            let got = measure_f0(&out[a..b], SR, want);
            let cents = 1200.0 * (got / want).log2();
            assert!(
                cents.abs() < 3.0,
                "{} {want} Hz: got {got} ({cents} c)",
                d.name
            );
        }
    }

    #[test]
    fn wav_header() {
        let w = wav(&[0.0, 0.5, -0.5], 48000);
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(w.len(), 44 + 6);
    }
}
