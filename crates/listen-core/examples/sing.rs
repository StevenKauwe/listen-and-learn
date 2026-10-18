//! Render every sung note name to WAV and report its measured pitch.
//!
//!     cargo run -p listen-core --example sing -- <out-dir>

use listen_core::music::{freq, SCALE};
use listen_core::voice::{measure_f0, plan, render, wav, SYLLABLES};

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    std::fs::create_dir_all(&dir).expect("out dir");
    let sr = 48000.0;
    let mut scale = Vec::new();
    for (i, d) in SCALE.iter().enumerate() {
        let s = SYLLABLES[i % 7];
        let out = render(s, d.midi as f32, 1.2, sr);
        let p = plan(s, 1.2);
        let (a, b) = (
            ((p.vowel_on + 0.05) * sr) as usize,
            (p.release_on * sr) as usize,
        );
        let want = freq(d.midi as f32);
        let got = measure_f0(&out[a..b], sr, want);
        println!(
            "{:<4} midi {} want {want:7.2} Hz  got {got:7.2} Hz  ({:+.2} cents)",
            d.name,
            d.midi,
            1200.0 * (got / want).log2()
        );
        let name = format!("{dir}/{}-{}-{}.wav", i + 1, d.name.to_lowercase(), d.midi);
        std::fs::write(&name, wav(&out, sr as u32)).expect("write");
        scale.extend_from_slice(&out[..(0.75 * sr) as usize]);
    }
    std::fs::write(format!("{dir}/scale-do-to-do.wav"), wav(&scale, sr as u32)).expect("write");
}
