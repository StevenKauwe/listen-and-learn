//! HTML for each screen. Plain strings: the browser parses them once per
//! screen change, and the wasm stays small.

use crate::art;
use listen_core::music::SCALE;
use listen_core::settings::NOTES_AT_ONCE;
use listen_core::{Age, Game, Settings, ALL};

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
        "who" => "Who's singing?",
        "drum" => "Drum along",
        "play" => "Play",
        _ => title,
    }
}

pub fn game(g: &dyn Game, targets: &[u8]) -> String {
    let stage = match g.slug() {
        "notes" | "play" => xylo(targets),
        "echo" => format!(
            r#"<div class="echo"><div class="beads" id="beads"></div>{}</div>"#,
            xylo(targets)
        ),
        "who" => {
            let picks: String = targets
                .iter()
                .map(|&t| {
                    format!(
                        r#"<button class="pick toy w{t}" data-t="{t}" aria-label="{}">{}</button>"#,
                        who_label(t),
                        art::who(t)
                    )
                })
                .collect();
            format!(r#"<div class="duo who n{}">{picks}</div>"#, targets.len())
        }
        "drum" => format!(
            r#"<div class="drumming"><div class="streak" id="streak" aria-live="off"></div><div class="bear-wrap" data-t="1" aria-hidden="true">{}</div><button class="pick pad toy" data-t="0" aria-label="Drum">{}</button></div>"#,
            art::bear_svg(),
            art::drum_svg()
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

fn who_label(t: u8) -> &'static str {
    match t {
        0 => "Singer",
        1 => "Xylophone",
        2 => "Bell",
        _ => "Flute",
    }
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

/// Drum along's streak: a star and the number of on-beat drums in a row.
pub fn streak(n: u32) -> String {
    format!(r#"{}<b>{n}</b>"#, art::STAR)
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

/// What the settings sheet shows besides the settings themselves.
#[derive(Clone, Copy, Default)]
pub struct SheetView {
    /// The "More" panel instead of the simple one.
    pub more: bool,
    /// The game whose details are open in "More".
    pub open: Option<usize>,
}

fn chips(act: &str, items: &[(String, bool)]) -> String {
    let mut out = String::new();
    for (i, (label, on)) in items.iter().enumerate() {
        out.push_str(&format!(
            r#"<button class="chip" data-act="{act}:{i}" aria-pressed="{on}">{label}</button>"#
        ));
    }
    format!(r#"<div class="chips">{out}</div>"#)
}

fn tog(act: &str, label: &str, on: bool) -> String {
    format!(
        r#"<button class="tog" data-act="{act}" role="switch" aria-checked="{on}"><span>{label}</span><i></i></button>"#
    )
}

/// A labelled chip row whose values are `values` (index = action value).
fn row(label: &str, act: &str, values: &[&str], current: usize) -> String {
    let items: Vec<(String, bool)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (v.to_string(), i == current))
        .collect();
    format!(
        r#"<div class="row-set"><h5>{label}</h5>{}</div>"#,
        chips(act, &items)
    )
}

/// Chip values for "after N" settings and the index of the current one.
const COUNTS: [u8; 4] = [0, 1, 2, 3];
const WAITS: [u8; 4] = [0, 3, 5, 8];
const SHOWS: [u8; 5] = [0, 1, 2, 3, 5];
pub const SET_LENS: [u8; 6] = [0, 6, 8, 10, 12, 15];

fn idx(values: &[u8], v: u8) -> usize {
    values.iter().position(|&x| x == v).unwrap_or(usize::MAX)
}

fn game_panel(s: &Settings, i: usize) -> String {
    let g = ALL[i];
    let t = s.tunes[i];
    let steps = g.steps();
    let mut body = String::new();
    if steps.len() > 1 {
        let easy = if g.free() || g.beat() {
            "Sounds"
        } else {
            "Starts at"
        };
        body.push_str(&row(easy, &format!("t:{i}:lo"), steps, t.lo as usize));
        if !g.free() && !g.beat() {
            body.push_str(&row(
                "Can get up to",
                &format!("t:{i}:hi"),
                steps,
                t.hi as usize,
            ));
        }
    }
    if !g.free() && !g.beat() {
        body.push_str(&row(
            "Show the answer for the first",
            &format!("t:{i}:show"),
            &["Off", "1 round", "2 rounds", "3 rounds", "5 rounds"],
            idx(&SHOWS, t.show),
        ));
        body.push_str(&tog(
            &format!("t:{i}:light"),
            "Then light the answer while it plays",
            t.light,
        ));
        body.push_str(&row(
            "Glow the answer after",
            &format!("t:{i}:glow_miss"),
            &["Never", "1 miss", "2 misses", "3 misses"],
            idx(&COUNTS, t.glow_miss),
        ));
        body.push_str(&row(
            "Glow after waiting",
            &format!("t:{i}:glow_wait"),
            &["Never", "3 s", "5 s", "8 s"],
            idx(&WAITS, t.glow_wait),
        ));
        body.push_str(&row(
            "Play it again after",
            &format!("t:{i}:replay"),
            &["Never", "1 miss", "2 misses", "3 misses"],
            idx(&COUNTS, t.replay),
        ));
    }
    if g.slug() == "play" {
        let labels: Vec<String> = NOTES_AT_ONCE.iter().map(|n| n.to_string()).collect();
        let labels: Vec<&str> = labels.iter().map(|l| l.as_str()).collect();
        body.push_str(&row(
            "Notes at once",
            "chord",
            &labels,
            idx(&NOTES_AT_ONCE, s.notes_at_once),
        ));
    }
    if g.slug() == "drum" {
        body.push_str(&tog(
            "tog:streak",
            "Count beats in a row (a miss just starts again)",
            s.streak,
        ));
    }
    if s.custom(i) {
        body.push_str(&format!(
            r#"<button class="reset" data-act="t:{i}:preset">Back to the age {} setting</button>"#,
            s.age.label()
        ));
    }
    body
}

pub fn sheet(s: &Settings, current: Option<usize>, v: SheetView) -> String {
    let done = r#"<button class="done" data-act="close">Done</button>"#;
    if !v.more {
        let ages: Vec<(String, bool)> = Age::ALL
            .iter()
            .map(|a| (a.label().to_string(), *a == s.age))
            .collect();
        let custom = (0..ALL.len()).filter(|&i| s.custom(i)).count();
        let note = if custom > 0 {
            format!(
                r#"<span class="hint-line">{custom} game{} changed by hand</span>"#,
                if custom == 1 { "" } else { "s" }
            )
        } else {
            String::new()
        };
        return format!(
            r#"<div class="sheet-head"><h2>Grown-ups</h2>{done}</div>
<div class="sheet-scroll">
<section><h3>Child's age</h3><div class="ages">{}</div><p class="blurb">{}</p></section>
<section><h3>Sound</h3>
<label class="slider" for="s-volume"><span>Volume</span><input id="s-volume" type="range" min="0" max="100" step="5" value="{vol}"><output id="o-volume">{vol}%</output></label>
{sing}{cheer}</section>
<button class="more" data-act="more"><span>More settings<small>Every game, hints, sounds, sets</small>{note}</span><i>›</i></button>
<section class="fine"><p>Hold the house for 1.5 seconds to go home. Hold the gear for 2 seconds to open this sheet. Settings stay on this device.</p>
<button class="reset" data-act="reset" id="reset">Reset settings</button></section>
</div>"#,
            chips("age", &ages).replace("class=\"chip\"", "class=\"chip age\""),
            s.age.blurb(),
            vol = (s.volume * 100.0).round() as u32,
            sing = tog("tog:sing", "Sing note names", s.sing),
            cheer = tog("tog:cheer", "Reward chime", s.cheer),
        );
    }
    let mut order: Vec<usize> = (0..ALL.len()).collect();
    if let Some(c) = current {
        order.retain(|&i| i != c);
        order.insert(0, c);
    }
    let mut games = String::new();
    for i in order {
        let g = ALL[i];
        let open = v.open == Some(i);
        let badge = if s.custom(i) {
            r#"<b class="badge">changed</b>"#.to_string()
        } else {
            format!(r#"<b class="badge soft">age {}</b>"#, s.age.label())
        };
        let here = if Some(i) == current {
            r#"<em>now playing</em>"#
        } else {
            ""
        };
        games.push_str(&format!(
            r#"<div class="gm{}"><button class="gm-head" data-act="open:{i}" aria-expanded="{open}"><span>{}{here}</span>{badge}<i>›</i></button>{}</div>"#,
            if open { " open" } else { "" },
            g.title(),
            if open {
                format!(r#"<div class="gm-body">{}</div>"#, game_panel(s, i))
            } else {
                String::new()
            }
        ));
    }
    format!(
        r#"<div class="sheet-head"><button class="back" data-act="more">‹ Back</button><h2>More</h2>{done}</div>
<div class="sheet-scroll">
<section><h3>Every game</h3>
<label class="slider" for="s-len"><span>Note length</span><input id="s-len" type="range" min="0.8" max="2" step="0.1" value="{len:.1}"><output id="o-len">{len:.1} s</output></label>
{auto}
{sets}
</section>
<section><h3>Each game</h3><p class="blurb">Choosing an age resets these. Games move between "starts at" and "can get up to": closer after three right first time, back after two misses.</p>{games}</section>
</div>"#,
        len = s.note_len,
        auto = tog("tog:auto_next", "Next round starts by itself", s.auto_next),
        sets = row(
            "Rest after this many right answers",
            "set",
            &["Never", "6", "8", "10", "12", "15"],
            idx(&SET_LENS, s.set_len),
        ),
    )
}
