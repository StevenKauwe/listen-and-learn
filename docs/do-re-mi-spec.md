# Do Re Mi — Toddler Ear-Training Game (Product Specification)

> Source of truth for behavior. Stored verbatim-in-intent from the owner's
> brief (2026-10-17); all thresholds and tables below are normative.

## 1. Overview
Fullscreen, touch-first ear-training game for ages 2–4, optimized for
iPhones. The app plays a note; the child taps the colored button for the
note they heard. Feel: a physical musical toy, not a conventional app.
**Primary design principle: the interface adapts to an imprecise toddler,
never the reverse.** Resting fingers, slight drags, long presses, multiple
fingers, inadvertent swipes — none may interrupt gameplay, scroll the
page, or cause accidental navigation.

## 2. Core gameplay & state machine
Flow: READY → (big "Listen!" pressed, note choices visible-but-inactive) →
PLAYING_MYSTERY → (note finishes) → AWAITING_GUESS → (toddler touches a
button; its note plays) → PLAYING_GUESS → CORRECT_FEEDBACK (celebration,
identify the note, auto-next ≈1.5 s) or INCORRECT_FEEDBACK (gentle, same
mystery note stays, unlimited retries, never penalize) → TRANSITIONING.

Explicit FSM phases: READY, PLAYING_MYSTERY, AWAITING_GUESS,
PLAYING_GUESS, CORRECT_FEEDBACK, INCORRECT_FEEDBACK, TRANSITIONING.
Only AWAITING_GUESS accepts answers. Touches during mystery playback are
ignored entirely — never queued. A finger already resting on a button
when guessing opens must lift and touch again. No overlapping
transitions, double scoring, or double-scheduled rounds. The app must
never accept an answer before the mystery note fully played once.

## 3. Toddler-proof touch (highest-priority engineering requirement)
**3.1 No accidental scrolling:** fits the viewport; `100dvh` + safe-area
insets; `overflow:hidden`; `overscroll-behavior:none`; `touch-action:none`
on the surface and buttons; `user-select:none` (+ webkit variants);
`-webkit-touch-callout:none`; proper viewport meta. Prevent pull-to-
refresh, rubber-banding, double-tap zoom, selection, context menus — as
far as the browser permits, without disabling OS accessibility.

**3.2 Imperfect taps (defaults in `doremi-core::pointer`):** Pointer
Events; judge the button at **initial contact**, never under the finger
at release; accept drift ≤ 35 CSS px; accept quick taps AND long presses;
cancel clearly-intentional swipes (> 70 px, or 35–70 px fast); never
activate a neighbor on slide-over; dedupe synthetic clicks; exactly one
guess per intentional interaction.

**3.3 Multi-touch:** one active pointer may initiate a guess; extra
fingers ignored until the interaction ends; never multi-answer; reset
reliably on pointercancel/suspension/transitions; no stuck pointers.

**3.4 Targets:** use most of the screen; three large colored rounded
rectangles by default; targets far beyond Apple's minimum; generous
spacing.

## 4. Interface
Fullscreen PWA on the Home Screen (standalone, no Safari chrome; document
that hosted pages cannot fully remove browser chrome). Portrait layout:
top ~15% feedback (friendly visual + two-word message), middle ~15–20%
one huge speaker/play button (animates gently; replay allowed after
playback), bottom ~70% the note buttons (large solfège text, no
decorative symbols, stable colors).

Default mapping (colors never change between rounds):
| Note | Color | Pitch |
|---|---|---|
| Do | Red | C4 — 261.63 Hz |
| Re | Orange | D4 — 293.66 Hz |
| Mi | Yellow | E4 — 329.63 Hz |
| Fa | Green | F4 — 349.23 Hz |
| Sol | Blue | G4 — 392.00 Hz |
| La | Purple | A4 — 440.00 Hz |
| Ti | Pink | B4 — 493.88 Hz |

Feedback: depress/highlight on touch; correct = star + positive sound;
incorrect = gentle wiggle, never an alarming flash; no harsh buzzers; no
rapid flashing; text secondary to buttons and sound.

## 5. Musical audio
Web Audio, equal-tempered. Defaults: C-major from C4; duration 1.2 s;
attack 20–40 ms; natural decay/release; warm piano/marimba-like timbre
(strong fundamental, gentle upper harmonics); consistent perceived
loudness; no clipping (compressor/limiter as needed); audible on iPhone
speakers; adult-accessible volume control. A mystery note always finishes
before guessing enables; a guess plays its note fully; feedback never
obscures audio; no abrupt cutoffs. iOS audio unlock on first Start tap;
handle interruptions/suspension; never open guessing if audio failed to
start — show an obvious retry. Same timbre/loudness for mystery and
buttons; never reveal the mystery note visually before a correct answer.

## 6. Difficulty & progression
Default Standard = Do, Re, Mi. Presets: Beginner Do/Mi/Sol (widely
separated) · Standard Do/Re/Mi · Intermediate Do–Sol · Advanced Do–Ti.
Adult chooses; no automatic progression by default (optional adult
toggle). Optional adaptives: repeat challenging notes; avoid immediate
repeats (core enforces); gradual note-count growth; toddler's main screen
unchanged by difficulty changes.

## 7. Feedback & rewards
Recognition, exploration, repetition — not competition. Correct: brief
celebration + flourish (non-interfering) + optional spoken name; auto-next.
Incorrect: no points lost, no game-over, no failure screen, no harsh
sounds; same note stays; unlimited retries; auto-replay after 2–3 misses
(core default 2); optional assist mode replays + gently highlights.
Rewards: small celebrations, optional streak stars; NO timers, lives, or
leaderboards; celebrations must not delay the next round.

## 8. Adult settings (hidden from toddlers)
Small gear icon; long-press ≈ 2 s to open; never opens accidentally.
Settings: difficulty/notes · duration 0.8–2.0 s · volume · timbre ·
spoken names · auto-next · celebration sounds · assist mode · reset
progress. Persist locally.

## 9. Technical
Lightweight self-contained app; iPhone Safari first; installable Home
Screen PWA; offline after install; no accounts/backend/analytics/ads/
external runtime deps; fast start, low memory. PWA manifest + icons +
service worker; single-file standalone HTML also produced (document that
opening a local file from iOS Files ≠ installing a PWA). Responsive:
portrait primary, landscape graceful, resize not scroll. Robust against
rapid touches, transitions-spanning touches, multi-finger, cancels,
rotation, Safari/PWA lifecycle; never lose the ability to start another
round; never double-register.

## 10. Acceptance tests
| Scenario | Expected |
|---|---|
| Touch Do, drag 20 px | One Do guess |
| Hold a button 2 s | One guess |
| Two fingers | At most one guess |
| Vertical swipe on a button | No scroll |
| Touch before mystery playback | No guess |
| Hold an answer through playback end | No guess |
| 10 rapid taps | No duplicate scoring/transitions |
| Correct guess | Pitch plays → feedback → next round |
| Incorrect guess | Gentle; same note remains |
| Background mid-note | Clean recovery |
| Rotate | Buttons visible/operable |
| Home Screen launch | Standalone layout |
| Offline | Gameplay + sound work |

(Logic rows: `crates/doremi-core/tests/acceptance.rs`. Browser rows:
the Playwright/WebKit harness.)

## 11. Priorities
P0: no-scroll · imprecise-touch handling · mystery-before-guessing ·
pitch/timing correctness · huge buttons · stable FSM · standalone
presentation. P1: warm tones · auto-next · gentle feedback · beginner
mode · PWA+offline · protected settings. P2: instruments · adaptive
learning · spoken names · progress · extended scales/octaves · extra
celebrations.

## 12. Deliverables
Standalone HTML game · installable offline PWA · full source · brief
iPhone install instructions · pointer/FSM test coverage.

**Final principle:** a two-year-old can play while holding the phone
imperfectly. Accidental scrolling, premature guesses and double
activations never interfere. The only required actions: listen, and
touch a big colored note button.
