# Listen & Learn: games

Every game follows [spec.md](spec.md). Each section gives the route, the
targets, what a question sounds like, the ladder (easiest rung first) and
any rows beyond the shared ones. Which rungs each age uses, and why, is
in [research.md](research.md) and `listen-core/src/calib.rs`.

## Do Re Mi (`#notes`)

Hear one note, touch its coloured bar. After a right answer the message
names it ("Mi!") and, with names on, the name is sung on that pitch.

| Rung | Bars |
|---|---|
| 0 | Mi Sol |
| 1 | Mi Sol La |
| 2 | Do Mi Sol La |
| 3 | Do Re Mi Sol La |
| 4 | Do to high Do |

- One bar tone of the adult note length; the bar lights while it plays
  when help allows. With three or more bars the same note never comes
  twice running; with two, runs are capped at three.
- Rows: every question is in the rung's set; question and answer sound
  identical for the right bar; the sung name is at the note's MIDI pitch.

## Right answers in the comparison games

Bird or Fish, Same or Different, Up or Down, Turtle or Rabbit, Mouse or
Elephant and Who's singing? confirm a right answer by playing the
question once more with the chosen picture lighting on each note (the
walker climbs or descends over the replay), then the chime. Onsets are
1.25× closer except in Turtle or Rabbit, whose tempo is the answer;
pitch and loudness never change. Only the tones starting in the first
1.6 s replay, so the celebration stays short.

## Bird or Fish (`#birdfish`)

High or low. The bird flies in the sky (top), the fish swims in the water
(bottom), in portrait and landscape alike.

| Rung | High notes | Low notes | Gap |
|---|---|---|---|
| 0 | A5–D6 | A2–D3 | 31 st |
| 1 | E5–G5 | E3–G3 | 21 st |
| 2 | B♭4–C5, after middle C | C3–D3, after middle C | 20 st |
| 3 | F4–G4, after middle C | F3–G3, after middle C | 10 st |

- Rungs 0–1 play the note twice; 2–3 play middle C first (never lit).
- Low notes stay at or above A2 (110 Hz) so phone speakers carry them.
- Touching the bird chirps high; touching the fish blubs low.

## Same or Different (`#twins`)

Two notes. Twin chicks for same; a chick and a ladybug for different.
Gap when different, by rung: 12, 7, 5, 3–4, 2, 1 semitones. Same and
different are equally likely.

## Up or Down (`#stairs`)

A short tune climbs or falls; a walker climbs or descends on a right
answer. Rungs: Do Mi Sol Do (octave), Do Mi Sol, five steps, three steps.
Starts on G3, A3, C4 or D4. Row: up strictly rises, down strictly falls.

## Echo (`#echo`)

Copy a short pattern on the bars, Simon-style. Beads fill in the colour
of each note copied. With help, each bar lights as it plays.

| Rung | Bars | Pattern |
|---|---|---|
| 0 | Mi Sol | 2 |
| 1 | Mi Sol La | 2 |
| 2 | Mi Sol La | 3 |
| 3 | Do Mi Sol La | 3 |
| 4 | Do Re Mi Sol La | 4 |

- Notes 0.55 s apart, never the same note twice in a row; with more than
  two bars, never the same pattern twice running.
- Each touch plays its bar. A slip wiggles that bar, clears the beads and
  replays the pattern after 0.9 s. When the answer is shown, the next bar
  to touch glows after each step.

## Turtle or Rabbit (`#tempo`)

The same five-note tune (Sol Mi Sol Mi Do), slow or fast. Seconds per
note (slow, fast) by rung: (0.9, 0.3), (0.8, 0.4), (0.72, 0.48),
(0.69, 0.51), (0.66, 0.54).

## Mouse or Elephant (`#loud`)

The same three notes (Do Mi Sol), quiet or loud. Loudness gap by rung:
24, 18, 12, 8, 6 dB. The loud version stays under the compressor
threshold, so the gap is real.

## Who's singing? (`#who`)

The same three-note tune on the same pitches (sol, mi, la), played by a
singer (sung "la"), a xylophone, a bell or a flute: only the timbre
differs. Rungs: singer or xylophone; add the bell; add the flute. Each
picture plays its own instrument when touched. Row: pitches and timing
are identical whoever plays.

## Drum along (`#drum`)

A bear drums a steady beat; the child drums on a big drum, which sounds
at contact with every finger (two hands drum freely). Listen starts the
bear, and it drums on for as long as the child does: no goal, no end of
set, no reset. After 30 s with no drum it rests until Listen.

Streak (adult toggle; off at 2–3, on at 4 and 5+): a star and the number
of drums in a row that landed on the beat. A drum between beats only
fades the number away; no sound, no wiggle, nothing wrong.

| Rung | Beat | Streak counts within | Lead-in |
|---|---|---|---|
| 0 | 0.5 s | ±30 % of a beat (±150 ms) | 4 |
| 1 | 0.6 s | ±25 % (±150 ms) | 4 |
| 2 | 0.5 s | ±18 % (±90 ms) | 4 |

- A drum is judged at the time it was touched against the beat as
  heard: the audio clock less the output latency (`getOutputTimestamp`
  where available, else `baseLatency + outputLatency`), so speaker and
  Bluetooth delay never skew it.
- Each beat counts once; the lead-in never counts; a skipped beat
  starts the streak again at 1; drums closer than 0.15 s count once.
  Every 8 in a row, a few stars.

## Free play (`#play`)

A xylophone with no questions and no wrong answers. Bars sound at contact;
a note floats up from each. Several fingers play at once, up to "notes at
once" (default 5; 1 · 2 · 3 · 5 · 10). Rungs: six bars (Do Re Mi Sol La
Do, pentatonic), eight bars.
