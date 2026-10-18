# Listen & Learn: suite specification

Source of truth for behaviour. Every game follows this file; per-game
rules, ladders and extra acceptance rows are in [games.md](games.md);
the evidence behind games, ages and numbers is in
[research.md](research.md). Thresholds and tables are normative.

## 1. Scope

Ear-training games for children aged 2 to 5, played on a phone held
imperfectly, iPhone Safari first. Each game is a toy: listen, then touch a
big picture or bar. The interface adapts to an imprecise child, never the
reverse. Resting fingers, slight drags, long presses, extra fingers and
stray swipes never scroll the page, navigate away, or count twice.

Recognition, exploration and repetition, not competition: no timers,
lives, scores, streak pressure or leaderboards in any game.

## 2. Shared requirements (every game)

### 2.1 Touch gate

One gate in front of every touch target, home tiles included
(`listen-core::gate`). Pointer Events only; no click handlers in play.

| Rule | Value |
|---|---|
| Target judged at | initial contact (never under the finger at release) |
| Drift that still counts as a tap | ≤ 35 CSS px |
| Swipe, always cancelled | > 70 px |
| Between 35 and 70 px | a tap if held ≥ 300 ms, otherwise cancelled |
| Long press | counts, once, on release |
| Answers per interaction | exactly one |
| Extra fingers | ignored until the tracked finger lifts |
| Finger already down when answers open | cannot answer; must lift and touch again |
| Touch while answers are closed | ignored, never queued |
| `pointercancel`, page hidden, screen change | gate resets, nothing stuck |

Free play is the one exception to "counts on release": a bar sounds at
contact, as a real xylophone does. One finger at a time still applies.

The page never scrolls, zooms, selects text or shows a callout:
`touch-action: none`, `overflow: hidden`, `overscroll-behavior: none`,
`user-select: none`, `-webkit-touch-callout: none`; `touchmove`,
`contextmenu`, `dblclick`, `selectstart` and `gesturestart` are cancelled
outside the adult sheet. Layout is one fixed screen (`position: fixed;
inset: 0`, equivalent to `100dvh`) padded by the safe-area insets. Portrait
first; landscape fits without scrolling.

### 2.2 Round machine

Every question game runs the same machine (`listen-core::round`).

| Phase | Answers | Leaves when |
|---|---|---|
| Ready | closed | Listen pressed |
| Question | closed | the question has fully played, measured on the audio clock |
| Awaiting | **open** | an answer is accepted (Listen here replays the question) |
| Right | closed | ≥ 1.5 s and the answer sound have finished; next question |
| Wrong | closed | the answer sound has finished; same question |

- Answers are accepted only in Awaiting. No answer is ever accepted before
  the question has played once in full.
- Wrong: a gentle wiggle, the same question stays, unlimited tries. The
  question replays by itself after the tune's "replay after" misses.
- Right: a short celebration that never delays the next round; with
  auto-next on, the next question plays by itself. After the set length
  of rights (6 at age 2) the round rests with "All done!" and stars
  until Listen is pressed.
- Help, per question (`listen-core::calib`):
  - **Shown** (the tune's first N right answers): the answer lights while
    the question plays and glows when answers open; every other choice is
    dimmed and, if touched, only makes its own sound. Nothing is wrong.
  - **Lit**: the answer lights while the question plays.
  - **By ear**: no light.
  - A glow on the answer after the tune's misses, or after its wait in
    seconds with no touch (delayed prompt). Two questions in a row that
    each needed two misses bring back one shown answer.
- Adaptive rung: three first-try rights step one rung harder, a question
  needing two misses one rung easier, always within the tune's band and
  never changing the targets on screen.
- Waits carry a sequence number; a stale timer never advances a round.
- Audio that fails to start or stops returns to Ready with a visible retry
  ("Tap again"); answers never open after an audio failure.
- Page hidden or interrupted mid-round: audio stops, the round returns to
  Ready with the same question (or the next one, if it was just answered).
- Single-answer games never give the same answer four times running.

### 2.3 Audio

Web Audio synthesis only; no sample files.

| Property | Value |
|---|---|
| Tuning | equal temperament, A4 = 440 Hz; scale degrees from C4 |
| Bar tone | sine fundamental plus gentle partials (2×, 3×, a marimba-like 3.98× and a faint 9.8× mallet tick), 26 ms attack, exponential decay over the note length |
| Note length (Do Re Mi, Free play) | default 1.2 s, adult range 0.8–2.0 s |
| Loudness | same voice and level for questions and answers; low notes lifted slightly and given more harmonics for phone speakers |
| Bus | compressor (−10 dB, 5:1) then limiter (−2 dB, 20:1) then volume |
| Reward chime | three soft high notes after the answer sound starts (adult toggle) |
| Sung note names | after a right Do Re Mi answer, sung on the note's own pitch by the Rust formant singer (`listen-core::voice`), 0.45 s after the bar; adult toggle, default on |
| Other voices | sung "la", bell (chime), flute, hand drum |

The audio context is created and resumed inside the first touch (iOS
unlocks only from a gesture) and resumed again on every touch. A refused
resume is the retry path, not an error.

### 2.4 Look and feedback

A painted wooden toy on a nursery wall, in light and dark themes. Inline
SVG characters with an ink outline and a faint hand-drawn wobble. Text is
secondary: at most two words on a play screen.

Solfège colours never change, in any game:

| Degree | Colour | Pitch |
|---|---|---|
| Do | red | C4, 261.63 Hz |
| Re | orange | D4, 293.66 Hz |
| Mi | yellow | E4, 329.63 Hz |
| Fa | green | F4, 349.23 Hz |
| Sol | blue | G4, 392.00 Hz |
| La | purple | A4, 440.00 Hz |
| Ti | pink | B4, 493.88 Hz |
| Do (high) | red | C5, 523.25 Hz |

Bars shorten from Do upward and rest on converging rails, like a real
xylophone. Touch depresses a target; right answers hop and scatter a few
stars; wrong answers wiggle. No flashing, no buzzers, no failure screens.
`prefers-reduced-motion` turns animation off.

### 2.5 Screens and navigation

- Home: one big picture tile per game, recognisable by picture and colour
  without reading. A tap plays the game's hint sound and opens it.
- Each game is its own route: `#notes`, `#birdfish`, `#twins`, `#stairs`,
  `#echo`, `#tempo`, `#loud`, `#who`, `#drum`, `#play` (bare hash tokens).
  No hash = home.
- Back to home: hold the house button 1.5 s. A short tap shows "Hold".
- Adult settings: hold the gear 2 s (home and every game). A short tap
  shows "Hold".
- Both holds show a filling ring and cancel if the finger drifts > 35 px.

### 2.6 Adult settings

A bottom sheet, opened only by the gear hold; it pauses the round. Closing
it starts the current game fresh with the new settings. Stored in
`localStorage`; unreadable or blocked storage falls back to defaults, and
every value is clamped.

Simple panel:

| Setting | Default | Range |
|---|---|---|
| Child's age | 2 | 2 · 3 · 4 · 5+; sets every game's tune and the set length |
| Volume | 80 % | 0–100 % |
| Sing note names | on | on/off |
| Reward chime | on | on/off |
| More settings | | opens the panel below; notes how many games were changed by hand |
| Reset settings | | two taps to confirm |

More panel: note length (1.2 s, 0.8–2.0 s); next round starts by itself
(on); rest after this many rights (age preset; never, 6, 8, 10, 12, 15).
Then one row per game (the current one first and open), marked "changed"
when it differs from the age preset, with: starts at and can get up to
(ladder rungs), show the answer for the first 0/1/2/3/5 rounds, then
light it while it plays, glow after 0–3 misses, glow after 0/3/5/8 s,
replay after 0–3 misses, and "back to the age setting". Free play and
Drum along show only their rung.

### 2.7 Platform

- Full screen: a button on the home screen uses the Fullscreen API and
  says so when it is refused. iPhone Safari has no Fullscreen API for
  pages, so on iOS (outside a Home Screen app) the home screen instead
  shows "For full screen on iPhone: Share, then Add to Home Screen."
- A screen wake lock is requested while a game is open and re-requested
  when the page becomes visible; refusal is ignored.
- Installable PWA (manifest, icons, `standalone`). A service worker serves
  the last good copy offline (network first). A hosted page in Safari
  cannot remove the browser chrome; only the Home Screen app can.
- The claude.ai artifact build has no service worker (the frame forbids
  it) and no PWA install; everything else is identical.

## 3. Architecture

Pure Rust compiled to WebAssembly.

- `crates/listen-core`: every decision, headless, no browser types. The
  touch gate, the round machine, settings, pitch tables and every game.
  All behavioural acceptance rows are unit tests here.
- `crates/listen-web`: the thin browser layer (`wasm-bindgen`, `web-sys`).
  Builds each screen's DOM and SVG, plays the core's sounds through Web
  Audio, forwards raw pointer events to the gate, runs timers. It makes no
  game decisions.
- `web/app.html`: the page body and all CSS. JavaScript is limited to the
  generated `wasm-bindgen` glue, a three-line module loader and the
  service worker.

Adding a game: one module in `listen-core/src/games/` implementing `Game`
(a two-picture game is a single `Duo` constant), one entry in `ALL`, and
its pictures in `listen-web/src/art.rs`. The gate, round machine, audio,
settings and layout are shared, so a game adds a few kilobytes. Current
size: about 240 KB of wasm (100 KB gzipped) for all ten games and the
singer, plus 39 KB of glue.

Build: `tools/build.sh` (`cargo build --release --target
wasm32-unknown-unknown`, `wasm-bindgen --target web`, optional `wasm-opt
-Oz`). The output must validate without relaxed SIMD, which iPhone WebKit
lacks: `wasm-tools validate --features=-relaxed-simd`.

## 4. Testing

- `cargo test --workspace`: the gate, the round machine for every game,
  help and adaptation, the age calibration limits, the singer (syllable
  timings, formant table, pitch of every rendered name), settings and
  ladders (`crates/listen-core/tests/acceptance.rs` plus
  unit tests).
- `python3 tests-browser/suite.py`: the built site in real WebKit
  (WebKitGTK 4.1 from Python, the Safari engine family; not Playwright,
  whose Linux build has no WebKit) with real `PointerEvent` sequences, at
  390×844 and 844×390. On machines without a GStreamer audio sink the
  audio context never runs: the harness first checks that the game shows
  retry and keeps answers closed, then shims only the audio clock to play
  every game through.
- `cargo run -p listen-core --example sing -- <dir>` renders every sung
  name to WAV and prints its measured pitch.
- Not covered automatically: sound quality and loudness on an iPhone
  speaker, Home Screen launch, offline on a device. Check these by hand.

## 5. Acceptance rows (every game)

| Scenario | Expected | Where |
|---|---|---|
| Touch a target, drift 20 px | one answer | core, browser |
| Hold a target 2 s | one answer | core |
| Two fingers | at most one answer | core, browser |
| 100 px swipe on a target | no answer, no scroll | core, browser |
| Touch before or during the question | no answer | core, browser |
| Hold through the end of the question | no answer | core |
| 10 rapid taps | one answer, one transition | core |
| Wrong answer | gentle; same question stays | core, browser |
| Age 2, first rounds | two choices, answer glows, other dimmed and never wrong | core, browser |
| Right answer | sound, celebration, next round | core, browser |
| Audio cannot start | retry shown, answers stay closed | core, browser |
| Page hidden mid-question | clean return to Ready | core |
| Stale timer after an interruption | ignored | core |
| Portrait 390×844 and landscape 844×390 | every target visible, no scroll | browser |
| Short tap on house or gear | nothing but a "Hold" hint | browser |
| Hold house 1.5 s / gear 2 s | home / settings | browser |
| Page errors during a full run | none | browser |
| Home Screen launch | standalone layout | by hand |
| Offline after one visit | games and sound work | by hand |

## 6. Priorities

P0: no scroll; imprecise-touch handling; no answer before the question has
played; correct pitch and timing; huge targets; a stable round machine.
P1: warm tone; auto-next; gentle feedback; easy first levels; offline PWA;
protected settings. P2: more games, instruments, adaptive levels, progress
views.
