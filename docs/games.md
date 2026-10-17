# Listen & Learn: games

Every game follows [spec.md](spec.md). Each section gives the route, the
targets, what a question sounds like, the levels (easiest first; the
default is marked) and any rows beyond the shared ones.

## Do Re Mi (`#notes`)

Hear one note, touch its coloured bar. The note is never shown before a
right answer; after one, the message names it ("Mi!").

| Level | Bars | |
|---|---|---|
| Beginner | Do, Mi, Sol | widely separated |
| Standard | Do, Re, Mi | **default** |
| Intermediate | Do to Sol | |
| Advanced | Do to Ti | |

- One bar tone of the adult note length. The same note never comes twice
  in a row. The answer plays its own note at the same length and level.
- Rows: every question is in the level's set; never the same note twice
  running; question and answer sound identical for the right bar.

## Bird or Fish (`#birdfish`)

High or low. The bird flies in the sky (top), the fish swims in the water
(bottom), in portrait and landscape alike.

| Level | High notes | Low notes |
|---|---|---|
| Very far apart (**default**) | A5–D6 | A2–D3 |
| Two octaves | E5–G5 | E3–G3 |
| One octave | G4–A4, after a middle-C reference | G3–A3, after a middle-C reference |

- Levels 1–2 play the note twice; level 3 plays middle C first.
- Low notes stay at or above A2 (110 Hz) so phone speakers carry them.
- Touching the bird chirps high; touching the fish blubs low.
- Row: at every level the lowest high note is ≥ 10 semitones above the
  highest low note.

## Same or Different (`#twins`)

Two notes. Twin chicks for same; a chick and a ladybug for different.

| Level | Gap when different |
|---|---|
| Very different (**default**) | 12 semitones |
| Different | 5 or 7 semitones |
| Close | 2 semitones |

- Same and different are equally likely. Row: same means identical
  pitches; different means exactly the level's gap.

## Up or Down (`#stairs`)

A short tune climbs or falls; a little walker climbs or descends the
stairs on a right answer. Up is the top picture in portrait.

| Level | Tune |
|---|---|
| Big jumps (**default**) | Do Mi Sol Do (arpeggio) |
| Five steps | Do Re Mi Fa Sol |
| Three steps | Do Re Mi |

- Starts on G3, A3, C4 or D4. Row: up tunes strictly rise, down tunes
  strictly fall.

## Echo (`#echo`)

Copy a short pattern on the bars, Simon-style. Beads above the bars fill
in the colour of each note copied.

| Level | Bars | Pattern | Bars light while playing |
|---|---|---|---|
| Two notes, lights (**default**) | Do Mi Sol | 2 | yes |
| Three notes, lights | Do Mi Sol | 3 | yes |
| Three notes, by ear | Do Mi Sol | 3 | no |
| Four notes, by ear | Do to Sol | 4 | no |

- Notes 0.55 s apart, never the same note twice in a row, never the same
  pattern twice running.
- Each touch plays its bar. A slip wiggles that bar, clears the beads and
  replays the pattern after 0.9 s. The whole pattern copied is a right
  answer.
- Lighting bars is the one place a game shows the notes before the answer,
  and only on the easy levels.

## Turtle or Rabbit (`#tempo`)

The same five-note tune (Sol Mi Sol Mi Do), slow or fast.

| Level | Slow (s per note) | Fast (s per note) |
|---|---|---|
| Very different (**default**) | 0.8 | 0.2 |
| Different | 0.62 | 0.3 |
| Close | 0.5 | 0.36 |

- Row: slow is at least 1.35× the fast note spacing at every level.

## Mouse or Elephant (`#loud`)

The same three notes (Do Mi Sol), quiet or loud. The mouse is drawn small,
the elephant large.

| Level | Loudness gap |
|---|---|
| Very different (**default**) | 24 dB |
| Different | 14 dB |
| Close | 8 dB |

- The loud version stays under the compressor threshold, so the gap is
  real. The phone's own volume shifts both equally; the gap is what the
  child compares.

## Free play (`#play`)

A xylophone with no questions and no wrong answers. Bars sound at contact
(the one exception to judging on release); a note floats up from each.

| Level | Bars |
|---|---|
| Six bars (**default**) | Do Re Mi Sol La Do (pentatonic) |
| Eight bars | Do to high Do |

- The pentatonic set has no semitone steps, so any order sounds
  consonant.
- Row: one sound per touch; a second finger is ignored.
