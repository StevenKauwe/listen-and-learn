//! Web Audio synthesis. Warm bar tone: strong fundamental, gentle partials
//! (a marimba-like fourth harmonic and a faint mallet tick), 26 ms attack,
//! exponential decay. Every voice goes through a compressor and a limiter
//! before the volume, so loudness is even and nothing clips.

use listen_core::music::{freq, Sound, Tone, Voice};
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioContextState, BiquadFilterType, GainNode};

pub struct Audio {
    pub ctx: AudioContext,
    bus: GainNode,
    master: GainNode,
    voices: Vec<(GainNode, f64)>,
    primed: bool,
    unmuted: bool,
}

/// iPhones mute Web Audio when the ring/silent switch is on, unlike media
/// playback, so a toddler's toy would be silent for most parents. Safari 17+
/// lets a page declare itself "playback" audio (`navigator.audioSession`).
/// Older iOS: a looping silent `<audio>` element started in a gesture moves
/// the page into the media category, which also carries Web Audio through.
/// Returns true once one of the two has been applied.
fn play_through_silent_switch() -> bool {
    use js_sys::Reflect;
    let Some(win) = web_sys::window() else {
        return false;
    };
    let nav: JsValue = win.navigator().into();
    if let Ok(session) = Reflect::get(&nav, &"audioSession".into()) {
        if session.is_object() {
            return Reflect::set(&session, &"type".into(), &"playback".into()).unwrap_or(false);
        }
    }
    let Ok(el) = web_sys::HtmlAudioElement::new_with_src(&silent_wav_uri()) else {
        return false;
    };
    el.set_loop(true);
    let _ = el.set_attribute("playsinline", "");
    match el.play() {
        Ok(p) => {
            let noop = wasm_bindgen::closure::Closure::once(|_: JsValue| {});
            let _ = p.catch(&noop);
            noop.forget();
            // Keep it alive (and playing) for the page's lifetime.
            std::mem::forget(el);
            true
        }
        Err(_) => false,
    }
}

/// A 0.1 s silent mono 8-bit 8 kHz WAV as a data URI (built here, no asset).
fn silent_wav_uri() -> String {
    const N: u32 = 800;
    let mut b: Vec<u8> = Vec::with_capacity(44 + N as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + N).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&8000u32.to_le_bytes());
    b.extend_from_slice(&8000u32.to_le_bytes()); // bytes per second
    b.extend_from_slice(&1u16.to_le_bytes()); // block align
    b.extend_from_slice(&8u16.to_le_bytes()); // bits
    b.extend_from_slice(b"data");
    b.extend_from_slice(&N.to_le_bytes());
    b.resize(44 + N as usize, 128); // 8-bit silence is the midpoint
    format!("data:audio/wav;base64,{}", base64(&b))
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= c.len() {
                s.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                s.push('=');
            }
        }
    }
    s
}

/// Lead time so the first partial never starts in the past.
const LEAD: f64 = 0.03;

impl Audio {
    pub fn new(volume: f32) -> Result<Audio, JsValue> {
        let ctx = AudioContext::new()?;
        let comp = ctx.create_dynamics_compressor()?;
        comp.threshold().set_value(-10.0);
        comp.knee().set_value(8.0);
        comp.ratio().set_value(5.0);
        comp.attack().set_value(0.004);
        comp.release().set_value(0.2);
        let lim = ctx.create_dynamics_compressor()?;
        lim.threshold().set_value(-2.0);
        lim.knee().set_value(0.0);
        lim.ratio().set_value(20.0);
        lim.attack().set_value(0.001);
        lim.release().set_value(0.1);
        let bus = ctx.create_gain()?;
        let master = ctx.create_gain()?;
        master.gain().set_value(volume);
        bus.connect_with_audio_node(&comp)?;
        comp.connect_with_audio_node(&master)?;
        master.connect_with_audio_node(&lim)?;
        lim.connect_with_audio_node(&ctx.destination())?;
        Ok(Audio {
            ctx,
            bus,
            master,
            voices: vec![],
            primed: false,
            unmuted: false,
        })
    }

    pub fn running(&self) -> bool {
        self.ctx.state() == AudioContextState::Running
    }

    pub fn state_name(&self) -> &'static str {
        match self.ctx.state() {
            AudioContextState::Running => "running",
            AudioContextState::Suspended => "suspended",
            AudioContextState::Closed => "closed",
            _ => "other",
        }
    }

    /// Call inside every user gesture (iOS only unlocks from one, and only
    /// from `touchend`/`pointerup`/`click`, not from a touch `pointerdown`).
    pub fn unlock(&mut self) {
        if !self.running() {
            if let Ok(p) = self.ctx.resume() {
                // A refused resume is handled by the retry path, not an error.
                let noop = wasm_bindgen::closure::Closure::once(|_: JsValue| {});
                let _ = p.catch(&noop);
                noop.forget();
            }
            // Re-prime until the context actually runs: a prime from a
            // non-activating event does nothing on iOS.
            self.primed = false;
        }
        if !self.unmuted {
            self.unmuted = play_through_silent_switch();
        }
        if !self.primed {
            if let (Ok(buf), Ok(src)) = (
                self.ctx.create_buffer(1, 1, 22050.0),
                self.ctx.create_buffer_source(),
            ) {
                src.set_buffer(Some(&buf));
                let _ = src.connect_with_audio_node(&self.ctx.destination());
                #[allow(deprecated)]
                let _ = src.start();
                self.primed = true;
            }
        }
    }

    pub fn now(&self) -> f64 {
        self.ctx.current_time()
    }

    pub fn set_volume(&self, v: f32) {
        self.master.gain().set_value(v);
    }

    /// Schedule a sound; returns its start time on the audio clock.
    pub fn play(&mut self, s: &Sound) -> f64 {
        let now = self.now();
        self.voices.retain(|(_, end)| *end > now);
        let t0 = now + LEAD;
        for t in &s.tones {
            let _ = self.tone(t, t0 + t.at as f64);
        }
        t0
    }

    fn tone(&mut self, t: &Tone, at: f64) -> Result<(), JsValue> {
        let ctx = &self.ctx;
        let f = freq(t.midi);
        // Small speakers carry low notes poorly: lift them a little and
        // give them more harmonics; tame the highest ones.
        let tilt = (262.0 / f).powf(0.22).clamp(0.7, 1.35);
        let peak = 0.3 * t.gain * tilt;
        let low = if f < 200.0 { 1.8 } else { 1.0 };
        let (partials, atk): (&[(f32, f32, f32)], f64) = match t.voice {
            Voice::Bar => (
                &[
                    (1.0, 1.0, 1.0),
                    (2.0, 0.2 * low, 0.55),
                    (3.0, 0.06 * low, 0.35),
                    (3.98, 0.11, 0.22),
                    (9.8, 0.035, 0.06),
                ],
                0.026,
            ),
            Voice::Chime => (
                &[
                    (1.0, 1.0, 1.0),
                    (2.0, 0.28, 0.5),
                    (3.0, 0.1, 0.3),
                    (6.3, 0.05, 0.12),
                ],
                0.008,
            ),
        };
        let dur = t.dur as f64;
        let end = at + dur + 0.35;
        let voice = ctx.create_gain()?;
        voice.gain().set_value_at_time(peak, at)?;
        voice.gain().set_value_at_time(peak, at + dur)?;
        voice.gain().linear_ramp_to_value_at_time(0.0, end)?;
        let lp = ctx.create_biquad_filter()?;
        lp.set_type(BiquadFilterType::Lowpass);
        lp.frequency().set_value((f * 9.0).min(12000.0));
        lp.q().set_value(0.3);
        voice.connect_with_audio_node(&lp)?;
        lp.connect_with_audio_node(&self.bus)?;
        for &(ratio, amp, decay) in partials {
            let pf = f * ratio;
            if pf > 15000.0 {
                continue;
            }
            let osc = ctx.create_oscillator()?;
            osc.frequency().set_value(pf);
            let env = ctx.create_gain()?;
            let g = env.gain();
            g.set_value_at_time(0.0, at)?;
            let a = if ratio > 3.0 { atk * 0.6 } else { atk };
            g.linear_ramp_to_value_at_time(amp, at + a)?;
            g.set_target_at_time(0.0, at + a, (dur * decay as f64 / 3.2).max(0.02))?;
            osc.connect_with_audio_node(&env)?;
            env.connect_with_audio_node(&voice)?;
            osc.start_with_when(at)?;
            osc.stop_with_when(end)?;
        }
        self.voices.push((voice, end));
        Ok(())
    }

    /// Fade every sounding voice out quickly (no clicks).
    pub fn stop_all(&mut self) {
        let now = self.now();
        for (v, end) in self.voices.drain(..) {
            if end > now {
                let g = v.gain();
                let _ = g.cancel_scheduled_values(now);
                let _ = g.set_target_at_time(0.0, now, 0.03);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn silent_wav_is_a_valid_header() {
        let uri = silent_wav_uri();
        assert!(uri.starts_with("data:audio/wav;base64,UklGR")); // "RIFF"
        assert_eq!(
            uri.len() - "data:audio/wav;base64,".len(),
            (44 + 800usize).div_ceil(3) * 4
        );
    }
}
