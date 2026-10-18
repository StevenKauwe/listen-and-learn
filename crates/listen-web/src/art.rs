//! Hand-drawn-feeling SVG art. `.k` is the ink outline (CSS); `#wob` is a
//! faint displacement filter that wobbles edges like a brush line. Every
//! character is drawn in a 120×120 box.

const INK: &str = "#2E2630";

fn svg(body: &str) -> String {
    svg_moving(body, "")
}

/// Parts that move all the time (a wing, the bear's paws) are each
/// wobbled on their own: moving anything inside the big filtered group
/// would redraw the whole picture's filter on every frame, which is slow
/// on phones and delays touches.
fn svg_moving(body: &str, moving: &str) -> String {
    format!(
        r#"<svg class="art" viewBox="0 0 120 120" aria-hidden="true"><g filter="url(#wob)">{body}</g>{moving}</svg>"#
    )
}

fn eye(x: f32, y: f32, r: f32) -> String {
    format!(
        r##"<circle fill="{INK}" cx="{x}" cy="{y}" r="{r}"/><circle fill="#fff" cx="{}" cy="{}" r="{}"/>"##,
        x + r * 0.35,
        y - r * 0.35,
        r * 0.32
    )
}

fn cheek(x: f32, y: f32) -> String {
    format!(r##"<circle fill="#F49C9C" opacity=".75" cx="{x}" cy="{y}" r="4"/>"##)
}

pub fn bird() -> String {
    bird_with(WING)
}

const WING: &str =
    r##"<path class="k wing" fill="#E2603F" d="M38 63 q14 -21 33 -4 q-14 16 -33 4z"/>"##;

fn bird_with(wing: &str) -> String {
    format!(
        r##"<g class="bird"><path class="k" fill="#E2603F" d="M34 64 L11 53 L16 67 L10 80 L35 73Z"/>
<ellipse class="k" fill="#F2845C" cx="58" cy="68" rx="30" ry="22"/>
<path fill="#FCD9C2" d="M42 78 q18 12 38 -2 q-6 13 -21 13 q-12 0 -17 -11z"/>
<circle class="k" fill="#F2845C" cx="80" cy="49" r="16"/>
{wing}
<path class="k" fill="#F6B93B" d="M95 45 l13 5 -13 6z"/>{}{}
<path class="k" fill="none" d="M52 89 l-3 10 M66 89 l3 10"/></g>"##,
        eye(84.0, 45.0, 3.4),
        cheek(87.0, 55.0)
    )
}

pub fn fish() -> String {
    format!(
        r##"<g class="fish"><path class="k tail" fill="#E89A2E" d="M31 62 L10 45 L15 62 L10 80Z"/>
<path class="k" fill="#F6C343" d="M28 62 C42 35 86 35 103 60 C86 87 42 87 28 62Z"/>
<path fill="none" stroke="#E89A2E" stroke-width="5" stroke-linecap="round" d="M52 47 q-7 15 0 30 M64 44 q-7 18 0 36"/>
<path class="k" fill="#E89A2E" d="M57 41 q11 -15 25 -1z"/>{}
<path class="k" fill="none" d="M100 66 q-5 3 -9 0"/>
<g class="bub" fill="none" stroke="#fff" stroke-width="2.5"><circle cx="110" cy="40" r="4"/><circle cx="114" cy="26" r="3"/></g></g>"##,
        eye(86.0, 56.0, 3.8)
    )
}

pub fn chick() -> String {
    format!(
        r##"<g class="chick"><path fill="none" stroke="#E8892B" stroke-width="3.5" stroke-linecap="round" d="M52 94 v9 l-5 3 M52 103 l5 3 M68 94 v9 l-5 3 M68 103 l5 3"/>
<circle class="k" fill="#F8D34A" cx="60" cy="66" r="30"/>
<path class="k" fill="#F8D34A" d="M55 37 q-3 -11 7 -11 q-6 4 -1 11"/>
<path class="k" fill="#EDB92F" d="M31 72 q-8 -5 -3 -17 q9 3 10 13z"/>
<path class="k" fill="#EDB92F" d="M89 72 q8 -5 3 -17 q-9 3 -10 13z"/>{}{}
<path class="k" fill="#F28A2E" d="M54 68 h12 l-6 8z"/>{}{}</g>"##,
        eye(50.0, 60.0, 3.6),
        eye(70.0, 60.0, 3.6),
        cheek(43.0, 70.0),
        cheek(77.0, 70.0)
    )
}

pub fn ladybug() -> String {
    format!(
        r##"<g class="bug"><path class="k" fill="none" d="M36 70 l-11 -5 M35 84 l-11 5 M84 70 l11 -5 M85 84 l11 5 M54 36 q-6 -12 -15 -12 M66 36 q6 -12 15 -12"/>
<circle class="k" fill="{INK}" cx="60" cy="46" r="13"/>
<ellipse class="k" fill="#E5493A" cx="60" cy="76" rx="29" ry="26"/>
<path class="k" fill="none" d="M60 51 V101"/>
<g fill="{INK}"><circle cx="47" cy="66" r="5"/><circle cx="73" cy="66" r="5"/><circle cx="46" cy="86" r="4.5"/><circle cx="74" cy="86" r="4.5"/></g>
<circle fill="#fff" cx="55" cy="43" r="3"/><circle fill="#fff" cx="65" cy="43" r="3"/>
<circle fill="{INK}" cx="55.6" cy="43.5" r="1.4"/><circle fill="{INK}" cx="65.6" cy="43.5" r="1.4"/></g>"##
    )
}

fn bean() -> String {
    format!(
        r##"<ellipse class="k" fill="#E5493A" cx="0" cy="0" rx="11" ry="12"/>{}{}<path class="k" fill="none" d="M-4 11 l-2 6 M4 11 l2 6"/>"##,
        eye(2.0, -3.0, 2.2),
        eye(7.0, -3.0, 2.2)
    )
}

/// Stairs going up (left low → right high) or down, with a walker.
pub fn stairs(up: bool) -> String {
    let (path, edge, start, cls) = if up {
        (
            "M10 108 V90 H35 V72 H60 V54 H85 V36 H110 V108Z",
            "M13 92.5 H33 M38 74.5 H58 M63 56.5 H83 M88 38.5 H107",
            (22.5, 78.0),
            "up",
        )
    } else {
        (
            "M10 108 V36 H35 V54 H60 V72 H85 V90 H110 V108Z",
            "M13 38.5 H33 M38 56.5 H58 M63 74.5 H83 M88 92.5 H107",
            (22.5, 24.0),
            "down",
        )
    };
    svg(&format!(
        r##"<path class="k" fill="#D9A46A" d="{path}"/><path fill="none" stroke="#F0CB9A" stroke-width="3" stroke-linecap="round" d="{edge}"/>
<g transform="translate({} {})"><g class="walk {cls}">{}</g></g>"##,
        start.0,
        start.1,
        bean()
    ))
}

pub fn turtle() -> String {
    format!(
        r##"<g class="turtle"><path class="k" fill="#A9D18E" d="M30 88 q-3 10 4 11 h9 q4 -2 2 -11z M72 88 q-2 10 4 11 h9 q5 -2 2 -11z"/>
<path class="k" fill="#A9D18E" d="M90 76 q7 -17 20 -11 q7 10 -6 17 q-8 2 -14 -6z"/>
<path class="k" fill="#6DB255" d="M20 88 a40 36 0 0 1 80 0z"/>
<path class="k" fill="#88C46C" d="M47 74 l6 -13 h14 l6 13 -6 12 h-14z"/>
<path class="k" fill="none" d="M47 74 h-18 M73 74 h18 M53 61 l-6 -12 M67 61 l6 -12"/>
<rect class="k" fill="#4E9442" x="16" y="85" width="88" height="8" rx="4"/>{}
<path class="k" fill="none" d="M108 76 q-3 2 -7 1"/></g>"##,
        eye(103.0, 70.0, 2.8)
    )
}

pub fn rabbit() -> String {
    format!(
        r##"<g class="rabbit"><ellipse class="k" fill="#F4EFE8" cx="73" cy="30" rx="7" ry="22" transform="rotate(-12 73 30)"/>
<ellipse class="k" fill="#F4EFE8" cx="89" cy="30" rx="7" ry="22" transform="rotate(12 89 30)"/>
<ellipse fill="#F6B6C4" cx="89" cy="31" rx="3" ry="14" transform="rotate(12 89 31)"/>
<circle class="k" fill="#fff" cx="30" cy="80" r="8"/>
<ellipse class="k" fill="#F4EFE8" cx="56" cy="80" rx="28" ry="21"/>
<path class="k" fill="#F4EFE8" d="M38 96 q-3 7 6 7 h12 M66 99 h13 q7 0 4 -7"/>
<circle class="k" fill="#F4EFE8" cx="82" cy="62" r="17"/>{}
<circle fill="#EE8DA3" cx="98" cy="64" r="2.8"/>{}</g>"##,
        eye(88.0, 58.0, 3.0),
        cheek(86.0, 69.0)
    )
}

pub fn mouse() -> String {
    format!(
        r##"<g class="mouse"><path class="k" fill="none" d="M34 86 q-19 5 -21 -10 q-1 -8 8 -10"/>
<ellipse class="k" fill="#B8B1C6" cx="58" cy="80" rx="26" ry="18"/>
<circle class="k" fill="#B8B1C6" cx="74" cy="55" r="12"/><circle fill="#F2B5C3" cx="74" cy="55" r="6.5"/>
<path class="k" fill="#B8B1C6" d="M66 70 q16 -12 34 6 q-14 10 -30 5z"/>{}
<circle class="k" fill="#EE8DA3" cx="101" cy="76" r="3"/>
<path fill="none" stroke="{INK}" stroke-width="1.6" stroke-linecap="round" d="M95 81 l10 4 M95 79 h11"/></g>"##,
        eye(86.0, 71.0, 2.6)
    )
}

pub fn elephant() -> String {
    format!(
        r##"<g class="eleph"><g class="k" fill="#8EA2B9"><rect x="28" y="80" width="14" height="23" rx="5"/><rect x="46" y="82" width="14" height="22" rx="5"/><rect x="64" y="82" width="14" height="22" rx="5"/><rect x="78" y="78" width="13" height="24" rx="5"/></g>
<path class="k" fill="none" d="M20 66 q-9 4 -7 14"/>
<ellipse class="k" fill="#8EA2B9" cx="55" cy="68" rx="36" ry="26"/>
<circle class="k" fill="#8EA2B9" cx="88" cy="55" r="21"/>
<path class="trunk" fill="none" stroke="{INK}" stroke-width="14" stroke-linecap="round" d="M104 62 q9 18 -2 33"/>
<path class="trunk" fill="none" stroke="#8EA2B9" stroke-width="8.5" stroke-linecap="round" d="M104 62 q9 18 -2 33"/>
<path class="k" fill="#7489A3" d="M79 42 q-19 -4 -21 14 q2 19 21 14 q6 -14 0 -28z"/>{}
<path class="k" fill="#FFF8EC" d="M97 70 q7 4 5 11 q-7 -3 -5 -11z"/></g>"##,
        eye(96.0, 49.0, 2.8)
    )
}

fn parrot() -> String {
    format!(
        r##"<path class="k" fill="#3E7FD6" d="M40 92 l-12 22 20 -12z"/>
<ellipse class="k" fill="#55AD4F" cx="50" cy="70" rx="20" ry="28"/>
<path class="k" fill="#3E9A45" d="M35 62 q11 -6 19 6 q-4 18 -16 21 q-7 -12 -3 -27z"/>
<circle class="k" fill="#55AD4F" cx="56" cy="40" r="16"/>
<path class="k" fill="#F4C430" d="M70 34 q13 2 11 17 q-6 -6 -13 -4z"/>
<circle class="k" fill="#fff" cx="60" cy="36" r="5.5"/><circle fill="{INK}" cx="61.5" cy="36" r="2.6"/>
<g class="k"><circle fill="#E5493A" cx="92" cy="30" r="6"/><circle fill="#F4C430" cx="104" cy="54" r="6"/><circle fill="#3E7FD6" cx="92" cy="78" r="6"/></g>
<path class="k" fill="none" d="M98 30 V12 M110 54 V36 M98 78 V60"/>"##
    )
}

/// A chick singing, with notes: the singer in Who's singing?
pub fn singer() -> String {
    format!(
        r##"<g transform="translate(-6 8) scale(.9)">{}</g><path class="k" fill="#F28A2E" d="M48 70 h14 l-7 9z"/>
<g class="notes"><path class="k" fill="#8C5BC8" d="M92 40 a5 4 0 1 1 -1 -1 V18 l12 -3 v7 l-11 3"/><path class="k" fill="#3E7FD6" d="M104 70 a4 3.4 0 1 1 -1 -1 V52 l9 -2 v6 l-8 2"/></g>"##,
        chick()
    )
}

/// A little xylophone with a mallet.
pub fn xylophone() -> String {
    let cols = ["#E5493A", "#F28A2E", "#F4C430", "#55AD4F", "#3E7FD6"];
    let mut b = String::from(
        r##"<path class="k" fill="#B98654" d="M10 50 L110 58 L110 64 L10 58Z M10 92 L110 86 L110 92 L10 98Z"/>"##,
    );
    for (i, c) in cols.iter().enumerate() {
        let x = 16.0 + i as f32 * 19.0;
        let h = 62.0 - i as f32 * 6.0;
        let y = 75.0 - h / 2.0;
        b.push_str(&format!(
            r#"<rect class="k" fill="{c}" x="{x}" y="{y}" width="15" height="{h}" rx="6"/>"#
        ));
    }
    b.push_str(
        r##"<path class="k" fill="none" stroke-width="5" d="M70 30 L100 6"/><path fill="none" stroke="#C99460" stroke-width="2.5" stroke-linecap="round" d="M70 30 L100 6"/><circle class="k" fill="#F6B6C4" cx="68" cy="33" r="8"/>"##,
    );
    b
}

/// A golden hand bell.
pub fn bell() -> String {
    r##"<g class="bell"><rect class="k" fill="#B98654" x="53" y="10" width="14" height="22" rx="6"/>
<path class="k" fill="#F4C430" d="M60 30 C38 30 34 52 32 78 L24 92 H96 L88 78 C86 52 82 30 60 30Z"/>
<path fill="none" stroke="#FFF1B8" stroke-width="5" stroke-linecap="round" d="M44 50 q-4 12 -5 26"/>
<circle class="k" fill="#E8A92E" cx="60" cy="100" r="8"/>
<path class="k" fill="none" d="M100 50 q8 8 0 18 M108 42 q14 16 0 34"/></g>"##
        .to_string()
}

/// A wooden flute with a few notes floating up.
pub fn flute() -> String {
    r##"<g class="flute"><rect class="k" fill="#D9A46A" x="8" y="64" width="104" height="16" rx="8" transform="rotate(-18 60 72)"/>
<g fill="#7A4A26" transform="rotate(-18 60 72)"><ellipse cx="28" cy="72" rx="4" ry="3"/><circle cx="52" cy="72" r="3.4"/><circle cx="66" cy="72" r="3.4"/><circle cx="80" cy="72" r="3.4"/><circle cx="94" cy="72" r="3.4"/></g>
<path class="k" fill="#55AD4F" d="M34 42 a5 4 0 1 1 -1 -1 V20 l12 -3 v7 l-11 3"/></g>"##
        .to_string()
}

/// The bear who drums the beat.
pub fn bear() -> String {
    format!(
        r##"<g class="bear"><circle class="k" fill="#B07A4A" cx="34" cy="26" r="11"/><circle class="k" fill="#B07A4A" cx="86" cy="26" r="11"/>
<circle class="k" fill="#C48B57" cx="60" cy="48" r="32"/>
<ellipse class="k" fill="#EBC79C" cx="60" cy="60" rx="15" ry="12"/>
<path class="k" fill="{INK}" d="M54 54 h12 l-6 6z"/><path class="k" fill="none" d="M60 60 v5 q-5 4 -9 0 M60 65 q5 4 9 0"/>{}{}{}{}</g>"##,
        eye(49.0, 42.0, 3.6),
        eye(71.0, 42.0, 3.6),
        cheek(41.0, 56.0),
        cheek(79.0, 56.0)
    )
}

/// The child's drum.
pub fn drum() -> String {
    r##"<g class="drumkit"><path class="k" fill="#E5493A" d="M14 44 V88 Q60 112 106 88 V44Z"/>
<path class="k" fill="none" stroke="#FFF8EC" stroke-width="4" d="M20 50 L40 96 L60 54 L80 96 L100 50"/>
<ellipse class="k" fill="#FFF3DE" cx="60" cy="44" rx="46" ry="16"/>
<ellipse fill="none" stroke="#E8D6BA" stroke-width="3" cx="60" cy="44" rx="30" ry="9"/></g>"##
        .to_string()
}

/// The question button's speaker.
pub const SPEAKER: &str = r##"<svg class="spk" viewBox="0 0 100 100" aria-hidden="true"><path class="k" fill="#FFF8EC" d="M18 40 h15 l21 -17 v54 l-21 -17 h-15z"/><path class="k w w1" fill="none" d="M65 38 q8 12 0 24"/><path class="k w w2" fill="none" d="M76 29 q14 21 0 42"/></svg>"##;

pub const HOUSE: &str = r##"<svg class="ico" viewBox="0 0 60 60" aria-hidden="true"><path class="k" fill="#FFF8EC" d="M12 30 L30 14 L48 30 V48 H12Z"/><path class="k" fill="#E5493A" d="M25 48 V37 h10 v11"/></svg>"##;

/// A gear: eight square teeth and a centre hole (the wood shows through),
/// never a sun or a sound.
pub const GEAR: &str = r##"<svg class="ico" viewBox="0 0 60 60" aria-hidden="true"><path class="k" fill="#FFF8EC" fill-rule="evenodd" stroke-width="2.5" d="M44.8 25.4 L50.6 25.8 L50.6 34.2 L44.8 34.6 L43.7 37.2 L47.5 41.6 L41.6 47.5 L37.2 43.7 L34.6 44.8 L34.2 50.6 L25.8 50.6 L25.4 44.8 L22.8 43.7 L18.4 47.5 L12.5 41.6 L16.3 37.2 L15.2 34.6 L9.4 34.2 L9.4 25.8 L15.2 25.4 L16.3 22.8 L12.5 18.4 L18.4 12.5 L22.8 16.3 L25.4 15.2 L25.8 9.4 L34.2 9.4 L34.6 15.2 L37.2 16.3 L41.6 12.5 L47.5 18.4 L43.7 22.8Z M37 30 A7 7 0 1 0 23 30 A7 7 0 1 0 37 30Z"/></svg>"##;

pub const FULLSCREEN: &str = r##"<svg class="ico" viewBox="0 0 60 60" aria-hidden="true"><path class="k" fill="none" stroke-width="5" d="M14 24 V14 H24 M36 14 H46 V24 M46 36 V46 H36 M24 46 H14 V36"/></svg>"##;

pub const RING: &str = r#"<svg class="ring" viewBox="0 0 64 64" aria-hidden="true"><circle cx="32" cy="32" r="29"/></svg>"#;

pub const STAR: &str = r##"<svg class="star" viewBox="0 0 40 40" aria-hidden="true"><path class="k" fill="#F4C430" d="M20 3 l5 11 12 1 -9 8 3 12 -11 -6 -11 6 3 -12 -9 -8 12 -1z"/></svg>"##;

pub const NOTE: &str = r##"<svg viewBox="0 0 30 40" aria-hidden="true"><path class="k" fill="currentColor" d="M10 30 a6 5 0 1 1 -1 -1 V6 l14 -3 v8 l-13 3"/></svg>"##;

/// The two pictures of a two-choice game.
pub fn duo(slug: &str) -> (String, String) {
    match slug {
        "birdfish" => (
            svg_moving(
                &bird_with(""),
                &format!(r#"<g filter="url(#wob)">{WING}</g>"#),
            ),
            svg(&fish()),
        ),
        "twins" => (
            svg(&format!(
                r#"<g transform="translate(-4 14) scale(.62)">{0}</g><g transform="translate(50 14) scale(.62)">{0}</g>"#,
                chick()
            )),
            svg(&format!(
                r#"<g transform="translate(-4 14) scale(.62)">{}</g><g transform="translate(50 14) scale(.62)">{}</g>"#,
                chick(),
                ladybug()
            )),
        ),
        "stairs" => (stairs(true), stairs(false)),
        "tempo" => (svg(&turtle()), svg(&rabbit())),
        "loud" => (
            svg(&format!(
                r#"<g transform="translate(30 40) scale(.55)">{}</g>"#,
                mouse()
            )),
            svg(&elephant()),
        ),
        _ => (String::new(), String::new()),
    }
}

/// The bear's paws drum on every beat, so they move outside the face's
/// filter (see [`svg_moving`]).
const PAWS: &str = r##"<g class="paw"><g filter="url(#wob)"><path class="k" fill="#C48B57" d="M28 78 q-14 10 -6 26 l10 -2 q-2 -10 6 -16z"/><path class="k stick" fill="none" stroke-width="4" d="M24 100 L6 116"/></g></g>
<g class="paw p2"><g filter="url(#wob)"><path class="k" fill="#C48B57" d="M92 78 q14 10 6 26 l-10 -2 q2 -10 -6 -16z"/><path class="k stick" fill="none" stroke-width="4" d="M96 100 L114 116"/></g></g>"##;

pub fn bear_svg() -> String {
    svg_moving(&bear(), PAWS)
}

pub fn drum_svg() -> String {
    svg(&drum())
}

/// The pictures of Who's singing?, by target id.
pub fn who(target: u8) -> String {
    svg(&match target {
        0 => singer(),
        1 => xylophone(),
        2 => bell(),
        _ => flute(),
    })
}

/// Home-tile picture for a game.
pub fn tile(slug: &str) -> String {
    match slug {
        "notes" => svg(
            r##"<path class="k" fill="#B98654" d="M20 18 L28 18 L34 104 L26 104Z M100 18 L92 18 L86 104 L94 104Z"/>
<rect class="k" fill="#E5493A" x="10" y="22" width="100" height="22" rx="10"/>
<rect class="k" fill="#F28A2E" x="16" y="50" width="88" height="22" rx="10"/>
<rect class="k" fill="#F4C430" x="22" y="78" width="76" height="22" rx="10"/>
<g fill="#FFF8EC" class="k" stroke-width="2"><circle cx="24" cy="33" r="3.5"/><circle cx="96" cy="33" r="3.5"/><circle cx="30" cy="61" r="3.5"/><circle cx="90" cy="61" r="3.5"/><circle cx="36" cy="89" r="3.5"/><circle cx="84" cy="89" r="3.5"/></g>"##,
        ),
        "birdfish" => svg(&format!(
            r##"<rect fill="#A9DAF1" x="6" y="8" width="108" height="54" rx="14"/><path fill="#3F8FC9" d="M6 62 q9 -6 18 0 t18 0 18 0 18 0 18 0 18 0 V100 q0 14 -14 14 H20 q-14 0 -14 -14Z"/>
<g transform="translate(26 0) scale(.55)">{}</g><g transform="translate(30 56) scale(.5)">{}</g>"##,
            bird(),
            fish()
        )),
        "twins" => svg(&format!(
            r#"<g transform="translate(-6 18) scale(.64)">{0}</g><g transform="translate(48 18) scale(.64)">{0}</g>"#,
            chick()
        )),
        "stairs" => stairs(true),
        "echo" => svg(&parrot()),
        "who" => svg(&format!(
            r#"<g transform="translate(-4 4) scale(.62)">{}</g><g transform="translate(46 44) scale(.6)">{}</g>"#,
            singer(),
            bell()
        )),
        "drum" => svg(&format!(
            r#"<g transform="translate(28 -2) scale(.53)">{}{}</g><g transform="translate(18 38) scale(.7)">{}</g>"#,
            bear(),
            PAWS.replace(r#"<g filter="url(#wob)">"#, "<g>"),
            drum()
        )),
        "tempo" => svg(&format!(
            r#"<g transform="translate(-6 40) scale(.6)">{}</g><g transform="translate(50 6) scale(.6)">{}</g>"#,
            turtle(),
            rabbit()
        )),
        "loud" => svg(&format!(
            r#"<g transform="translate(-2 60) scale(.42)">{}</g><g transform="translate(38 20) scale(.7)">{}</g>"#,
            mouse(),
            elephant()
        )),
        "play" => {
            let cols = [
                "#E5493A", "#F28A2E", "#F4C430", "#55AD4F", "#3E7FD6", "#8C5BC8",
            ];
            let mut b = String::from(
                r##"<path class="k" fill="#B98654" d="M8 34 L112 44 L112 50 L8 42Z M8 84 L112 76 L112 82 L8 92Z"/>"##,
            );
            for (i, c) in cols.iter().enumerate() {
                let x = 12.0 + i as f32 * 16.5;
                let h = 76.0 - i as f32 * 6.0;
                let y = 62.0 - h / 2.0;
                b.push_str(&format!(
                    r#"<rect class="k" fill="{c}" x="{x}" y="{y}" width="13" height="{h}" rx="6"/>"#
                ));
            }
            b.push_str(
                r##"<path class="k" fill="none" stroke-width="5" d="M68 104 L98 70"/><path fill="none" stroke="#C99460" stroke-width="2.5" stroke-linecap="round" d="M68 104 L98 70"/><circle class="k" fill="#F6B6C4" cx="100" cy="66" r="9"/>"##,
            );
            svg(&b)
        }
        _ => String::new(),
    }
}
