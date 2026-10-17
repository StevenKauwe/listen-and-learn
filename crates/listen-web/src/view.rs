//! HTML for each screen. Plain strings: the browser parses them once per
//! screen change, and the wasm stays small.

use crate::art;
use listen_core::music::SCALE;
use listen_core::{Game, Settings, ALL};

/// Target ids in `data-t`: answers use their game ids (0..), these are
/// the shared controls.
pub const T_LISTEN: u8 = 100;
pub const T_FULLSCREEN: u8 = 101;
pub const T_TILE: u8 = 110;

/// Bars shorten by this share of the row per step, like a real xylophone.
const BAR_STEP: f32 = 4.5;

fn knob_hold(kind: &str, label: &str, icon: &str) -> String {
    format!(
        r#"<button class="knob {kind}" data-hold="{kind}" aria-label="{label}">{}{icon}</button>"#,
        art::RING
    )
}

pub fn home(show_fs: bool, ios_hint: bool) -> String {
    let fs = if show_fs {
        format!(
            r#"<button class="knob" data-t="{T_FULLSCREEN}" aria-label="Full screen">{}</button>"#,
            art::FULLSCREEN
        )
    } else {
        r#"<span class="knob-space"></span>"#.to_string()
    };
    let mut tiles = String::new();
    for (i, g) in ALL.iter().enumerate() {
        tiles.push_str(&format!(
            r#"<button class="tile toy" data-t="{}" style="--tile:var(--t-{slug})" aria-label="{title}"><span class="pic">{}</span><span class="cap">{title}</span></button>"#,
            T_TILE as usize + i,
            art::tile(g.slug()),
            slug = g.slug(),
            title = short_title(g.slug(), g.title()),
        ));
    }
    format!(
        r#"<div class="screen home"><header class="top">{fs}<h1 class="brand">Listen <i>&amp;</i> Learn</h1>{gear}</header>
<main class="shelf">{tiles}</main>{hint}</div>"#,
        gear = knob_hold("gear", "Grown-ups: hold for settings", art::GEAR),
        hint = if ios_hint {
            r#"<p class="ios-hint">For full screen on iPhone: Share, then Add to Home Screen.</p>"#
        } else {
            ""
        },
    )
}

fn short_title(slug: &str, title: &'static str) -> &'static str {
    match slug {
        "twins" => "Same?",
        "loud" => "Quiet or Loud",
        "tempo" => "Slow or Fast",
        "play" => "Play",
        _ => title,
    }
}

pub fn game(g: &dyn Game, level: u8) -> String {
    let targets = g.targets(level);
    let stage = match g.slug() {
        "notes" | "play" => xylo(&targets),
        "echo" => format!(
            r#"<div class="echo"><div class="beads" id="beads"></div>{}</div>"#,
            xylo(&targets)
        ),
        slug => {
            let (a, b) = art::duo(slug);
            format!(
                r#"<div class="duo duo-{slug}"><button class="pick p0 toy" data-t="0" aria-label="{}">{a}</button><button class="pick p1 toy" data-t="1" aria-label="{}">{b}</button></div>"#,
                pick_label(slug, 0),
                pick_label(slug, 1)
            )
        }
    };
    let free = if g.free() { " free" } else { "" };
    format!(
        r#"<div class="screen game g-{slug}{free}"><header class="top">{home}<div class="msg" id="msg" aria-live="polite"></div>{gear}</header>
<div class="listen-wrap"><button class="listen" id="listen" data-t="{T_LISTEN}" aria-label="Listen">{spk}</button></div>
<div class="stage" id="stage">{stage}</div></div>"#,
        slug = g.slug(),
        home = knob_hold("home", "Hold to go home", art::HOUSE),
        gear = knob_hold("gear", "Grown-ups: hold for settings", art::GEAR),
        spk = art::SPEAKER,
    )
}

fn pick_label(slug: &str, t: u8) -> &'static str {
    match (slug, t) {
        ("birdfish", 0) => "Bird, high",
        ("birdfish", _) => "Fish, low",
        ("twins", 0) => "Same",
        ("twins", _) => "Different",
        ("stairs", 0) => "Up",
        ("stairs", _) => "Down",
        ("tempo", 0) => "Turtle, slow",
        ("tempo", _) => "Rabbit, fast",
        ("loud", 0) => "Mouse, quiet",
        _ => "Elephant, loud",
    }
}

/// Converging rails: each bar end sits on them, as on a real xylophone.
fn rails(n: usize) -> String {
    let n = n as f32;
    let x = |y: f32| 5.5 + BAR_STEP / 2.0 * (y * n / 100.0 - 0.5);
    let (a, b) = (x(0.0), x(100.0));
    let line = |x1: f32, y1: f32, x2: f32, y2: f32| {
        format!(r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}"/>"#)
    };
    format!(
        r#"<svg class="rails rp" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{}{}</svg><svg class="rails rl" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{}{}</svg>"#,
        line(a, 0.0, b, 100.0),
        line(100.0 - a, 0.0, 100.0 - b, 100.0),
        line(0.0, a, 100.0, b),
        line(0.0, 100.0 - a, 100.0, 100.0 - b),
    )
}

pub fn xylo(targets: &[u8]) -> String {
    let mut rows = String::new();
    for (i, &d) in targets.iter().enumerate() {
        let deg = SCALE[d as usize];
        rows.push_str(&format!(
            r#"<div class="row" data-t="{d}" style="--c:var(--{key});--i:{i}" role="button" tabindex="0" aria-label="{name}"><div class="bar"><i class="peg"></i><b>{name}</b><i class="peg"></i></div></div>"#,
            key = deg.key,
            name = deg.name,
        ));
    }
    let n = targets.len();
    format!(
        r#"<div class="xylo"><div class="frame" style="--n:{n}">{}{rows}</div></div>"#,
        rails(n)
    )
}

pub fn beads(answer: &[u8], done: usize) -> String {
    (0..answer.len())
        .map(|i| {
            if i < done {
                format!(
                    r#"<i class="bead on" style="--cb:var(--{})"></i>"#,
                    SCALE[answer[i] as usize].key
                )
            } else {
                r#"<i class="bead"></i>"#.to_string()
            }
        })
        .collect()
}

pub fn sheet(s: &Settings, current: Option<usize>) -> String {
    let mut games = String::new();
    let mut order: Vec<usize> = (0..ALL.len()).collect();
    if let Some(c) = current {
        order.retain(|&i| i != c);
        order.insert(0, c);
    }
    for i in order {
        let g = ALL[i];
        let chips: String = g
            .levels()
            .iter()
            .enumerate()
            .map(|(l, name)| {
                let on = s.levels[i] as usize == l;
                format!(r#"<button class="chip" data-act="lv:{i}:{l}" aria-pressed="{on}">{name}</button>"#)
            })
            .collect();
        let here = if Some(i) == current { " here" } else { "" };
        games.push_str(&format!(
            r#"<div class="lv{here}"><h4>{}</h4><div class="chips">{chips}</div></div>"#,
            g.title()
        ));
    }
    let tog = |key: &str, label: &str, on: bool| {
        format!(
            r#"<button class="tog" data-act="tog:{key}" role="switch" aria-checked="{on}"><span>{label}</span><i></i></button>"#
        )
    };
    format!(
        r#"<div class="sheet-head"><h2>Grown-ups</h2><button class="done" data-act="close">Done</button></div>
<div class="sheet-scroll">
<section><h3>Levels</h3>{games}</section>
<section><h3>Sound</h3>
<label class="slider" for="s-volume"><span>Volume</span><input id="s-volume" type="range" min="0" max="100" step="5" value="{vol}"><output id="o-volume">{vol}%</output></label>
<label class="slider" for="s-len"><span>Note length</span><input id="s-len" type="range" min="0.8" max="2" step="0.1" value="{len:.1}"><output id="o-len">{len:.1} s</output></label>
{speak}{cheer}</section>
<section><h3>Play</h3>{auto}{assist}</section>
<section class="fine"><p>Hold the house for 1.5 seconds to go home. Hold the gear for 2 seconds to open this sheet. Settings stay on this device.</p>
<button class="reset" data-act="reset" id="reset">Reset settings</button></section>
</div>"#,
        vol = (s.volume * 100.0).round() as u32,
        len = s.note_len,
        speak = tog("speak", "Say note names", s.speak),
        cheer = tog("cheer", "Reward chime", s.cheer),
        auto = tog("auto_next", "Next round starts by itself", s.auto_next),
        assist = tog("assist", "Glow the answer after 3 tries", s.assist),
    )
}
