# Listen & Learn: research basis

Why each game exists, the age it suits, and where its numbers come from.
The numbers live in `crates/listen-core/src/calib.rs` (age bands and
help) and in each game's ladder (`games/*.rs`); tests pin the limits.
Every source below was opened; "abstract" means only the abstract was
read. Anything marked **not verified** is a design inference.

## What the evidence says, in short

- **Big contrasts first.** Even 5-year-olds find pitch *direction* hard:
  with 1 s piano tones, 80 % of trials right at 4 semitones but only
  18 of 26 children reliably above chance; smaller steps fell below half
  the children [1]. Discriminating is easier than labelling: preschoolers
  told apart melodies with different contours but could not link them to
  cartoon characters (chance), while a large *timbre* change was both
  discriminated and linked to pictures excellently [2]. Preschool
  frequency thresholds can reach 3–5 % with a play task, but only 56 % of
  3–4-year-olds passed training [3]. Loudness discrimination is adult-like
  by about 5 [4], but 2-year-olds' sense of loud as "more" is still
  disorganised [5].
- **A visual model makes the mapping.** With a ball rising and falling in
  time with the pitch, 5-year-olds were 95 % right; without it, 75 % [1].
  Children of 30–36 months reliably match a higher pitch to a smaller,
  lighter ball [6]; infants already link rising pitch to rising height
  [7]. Hence: bird up/high, mouse small/quiet, and an animated answer.
- **Touch must point at the thing.** 24–36-month-olds learned from
  touchscreen video only when the touch had to land on the relevant
  object, not anywhere on screen; for older toddlers that requirement
  slightly hurt [8, 9]. Two-year-olds tap more during instructions than
  4–6-year-olds, and more tapping goes with lower self-regulation [10].
  Two-year-olds swipe, hit the screen and use several fingers far more
  than 3-year-olds, and improve fast with practice [11].
- **Errorless, then fade.** A prompt given at once and then delayed
  (constant time delay) taught preschoolers with fewer errors and trials
  than prompting only after errors [12].
- **Tempo and beat.** 3-year-olds told a "rabbit" tempo from a "tortoise"
  one at ±50 % around 600 ms (~79 % right), with thresholds around
  ±10–15 %; 4-year-olds around ±5–10 % [13]. Children's own tapping tempo
  is about 400–500 ms, and young children synchronise only near it
  [14, secondary]. With a live partner even 2.5-year-olds adjust their
  drumming, and all ages synchronise better than with a machine [15]. Of
  156 3–5-year-olds drumming with a live partner, about 40 % kept both a
  400 and a 600 ms beat and 24 % kept neither [16].
- **Pitch structure and singing.** Infants respond to pitch and timing
  patterns early; temporal learning outpaces pitch learning [17]. Key
  membership is evident by 4–5 [18]. Young children's songs start as
  short phrases with a falling contour and a narrow range; the early
  singing ranges are about D4–F4, widening to D4–A4 [19]. Happy voice
  quality, more than speech versus song, holds infants' attention [20].
- **Music-education sequences.** Kodály exercises begin with the minor
  third mi–sol, then add la, do and re (pentatonic); fa and ti come last;
  concepts go from easiest for the child to harder, sound before symbol
  [21]. The claim that the falling sol–mi is a universal "children's
  chant" comes from teaching practice and is **not verified** by
  research; the research supports a falling contour and narrow range
  [19]. Gordon's Music Learning Theory describes informal, playful,
  pattern-based guidance before formal instruction, with three types and
  seven stages of "preparatory audiation" [22]; the age boundaries of
  those stages are **not verified**. No Orff or Jessica Grahn source on
  this age group was opened; neither is relied on.
- **Screen time.** AAP (2016): avoid screens under 18 months except video
  chat; 18–24 months only high-quality media, with a parent; 2–5 years at
  most one hour a day, ideally shared [23]. Short sets follow from this.
  The set lengths themselves (6–12 rights) are **not verified**.

## Design rules that follow

1. Age 2 gets two choices at most and only maximal contrasts: an octave
   or more for pitch comparisons, a 3:1 tempo ratio, 24 dB loudness.
   Fewer than two choices would remove the listening; more would add
   chance errors (choice count: **not verified**, no study found that
   compares 1, 2 and 3 choices at this age).
2. The first answers are *shown*: the answer lights while it plays,
   glows afterwards, and the other choice is dimmed and only makes its
   sound. A touch on it is exploration, never an error. This stops random
   tapping from paying off while keeping every touch audible.
3. Help then fades: at 2 the answer still lights while it plays (the
   training condition in [1]); at 3 only for notes and echo; at 4–5 not.
   A glow comes after a delay (4 s at age 2, 6 s at 3: time delay [12])
   or after misses. Two hard questions in a row bring back one shown
   answer.
4. Within an age band the game adapts: three first-try rights step one
   rung closer; a question needing two misses steps back.
5. Questions open only after they have played (a delayed choice), and
   touches must land on the answer itself (specific contingency [8, 9]).
6. Short sets: a rest and a celebration after 6 rights at 2, 8 at 3, 10 at
   4, 12 at 5+.

## Games

| Game | Trains | Evidence | Suits | Age bands (ladder rungs) |
|---|---|---|---|---|
| Who's singing? (new) | timbre | timbre–picture links are easy for preschoolers, contour ones are not [2] | 2+ | 2: singer or xylophone · 3: + bell · 4–5+: + flute |
| Bird or Fish | high/low | pitch–height/size correspondence from infancy [6, 7]; direction is hard, so huge gaps [1] | 2+ | 2: 31 st apart · 3: 31 → 21 st · 4: 31 → 20 st (middle-C reference) · 5+: 21 → 10 st |
| Drum along (new) | steady beat | partner model [15], tempo near the child's own [14], many can't sync yet [16] | 2+ | 2–3: free drumming with the bear, 500 ms, nothing counted · 4: streak on, ±25 % of 600 ms · 5+: streak on, ±18 % of 500 ms |
| Turtle or Rabbit | tempo | rabbit/tortoise tempo task, thresholds by age [13] | 2+ (best 3+) | 2: 3:1 · 3: 3:1 → 2:1 · 4: → ±15 % · 5+: 2:1 → ±10 % |
| Mouse or Elephant | loudness | adult-like by 5 [4]; loud-as-more disorganised at 2 [5] | 3+ (playable at 2) | 2: 24 dB · 3: 24 → 18 · 4: → 8 · 5+: 18 → 6 dB |
| Do Re Mi | pitch names | Kodály order [21]; names sung on their pitch; ear-only identification of a 3-semitone step is beyond 2-year-olds [1], so age 2 keeps the light | 2+ with lights, by ear 4+ | 2: sol mi · 3: sol mi la · 4: do mi sol la · 5+: do re mi sol la |
| Echo | melodic memory | short patterns; Gordon's pattern focus [22]; sol–mi first [21] | 2+ with lights | 2: sol mi, 2 notes · 3: + la, 2–3 notes · 4: do mi sol la, 3 · 5+: up to 4 notes |
| Up or Down | contour | direction at 4 st is hard even at 5 [1]; contour labels are hard [2] | 3+ (playable at 2) | 2: octave arpeggio · 3: + do mi sol · 4: + five steps · 5+: down to three 2-st steps |
| Same or Different | change detection | detecting a change is easier than naming its direction [1] | 3+ (the idea of "same" at 2: **not verified**) | 2: octave · 3: → fifth · 4: → fourth · 5+: fifth → whole step |
| Free play | exploration | contingency: every touch sounds; pentatonic so any order is pleasant | 2+ | 2–3: six bars · 4+: eight; five notes at once at every age |

Drum along's streak is off below 4: about a quarter of 3–5-year-olds
keep neither a 400 nor a 600 ms beat [16], so a count would mostly show
younger children failing. From 4 it counts drums in a row within the
rung's window (±150 ms at 600 ms, ±90 ms at 500 ms); a miss only starts
it again. The windows are the earlier judged-rung values; they are now
measured against the beat as heard (output latency removed), which makes
them effectively fairer, not stricter.

Free play allows five notes at once at every age. Two-year-olds use
several fingers far more than 3-year-olds [11]; with one finger at a
time, a resting finger or palm silences every other touch, which reads as
"broken" rather than as a rule. A real xylophone sounds under every hand.
Whether multi-touch confuses toddlers was **not verified**.

Replaying the question with the chosen picture moving after a right
answer pairs the sound with its picture once more (the cross-modal
links in [6, 7]) and makes the touch's consequence specific to it [8,
9]. A study testing replay-on-success itself was not found: **not
verified**.

No game was removed: each has at least some support at some age, and
the bands put the weaker ones (same/different, up/down, loudness) at
their easiest for 2-year-olds, with the answer shown.

## Sung note names

Note names are sung on their own pitch (Mi on E4 = 329.63 Hz), after the
bar note of a right Do Re Mi answer, default on. Reasons: Kodály practice
always sings solfège on its pitch [21]; a spoken name at an arbitrary
pitch pairs the name with the wrong sound; the notes C4–C5 sit in the
early singing range [19], inviting the child to sing it back. Whether
singing the name speeds learning at 2 is **not verified**. Voice quality
should be warm and happy [20].

The singer (`listen-core::voice`) is rendered in Rust: a band-limited
pulse at the exact pitch with gentle late vibrato (±22 cents, symmetric,
so the mean pitch is the note), four cascaded formant resonators, and
consonants from band-passed noise (f, s, t, the d burst) or formant
glides (d, r, m, l). Vowel formants F1–F3 are the child means of
Hillenbrand et al. (1995) [24]: /o/ 598, 1141, 3021 Hz; /e/ 565, 2658,
3335; /i/ 454, 3094, 3704; /ɑ/ 999, 1694, 2949. F4, bandwidths and
consonant loci are synthesis choices (**not verified** against
recordings). Measured: every rendered name's spectral peak is within
0.1 cent of its note.

## Sources

1. Stalinski, Schellenberg & Trehub (2008). Developmental changes in the
   perception of pitch contour: distinguishing up from down. *JASA* 124,
   1759–1763. Full text.
2. Creel (2016). Ups and downs in auditory development: preschoolers'
   sensitivity to pitch contour and timbre. *Cognitive Science* 40,
   373–403. Abstract.
3. Rose, Flaherty, Browning, Leibold & Buss (2018). Pure-tone frequency
   discrimination in preschoolers, young school-age children, and
   adults. *JSLHR* 61, 2440–2445. Full text.
4. Jensen & Neff (1993). *Psychological Science* 4, 104–107. Abstract.
5. Smith & Sera (1992). A developmental analysis of the polar structure
   of dimensions. *Cognitive Psychology* 24. Abstract.
6. Mondloch & Maurer (2004). Do small white balls squeak? Pitch–object
   correspondences in young children. *CABN* 4, 133–136. Abstract.
7. Walker et al. (2010). Preverbal infants' sensitivity to synaesthetic
   cross-modality correspondences. *Psychological Science* 21. Abstract.
8. Kirkorian, Choi & Pempek (2016). Toddlers' word learning from
   contingent and noncontingent video on touch screens. *Child
   Development* 87, 405–413. Abstract.
9. Choi & Kirkorian (2016). Touch or watch to learn? *Psychological
   Science* 27. Abstract.
10. Russo-Johnson, Troseth, Duncan & Mesghina (2017). All tapped out.
    *Frontiers in Psychology* 8:578. Abstract.
11. Courage, Frizzell, Walsh & Smith (2021). Toddlers using tablets.
    *Frontiers in Psychology* 12:564479. Full text.
12. Doyle, Wolery, Gast, Ault & Wiley (1990). Comparison of constant
    time delay and the system of least prompts in teaching preschoolers
    with developmental delays. *Research in Developmental Disabilities*
    11, 1–22. Abstract.
13. Bobin-Bègue & Provasi (2005). Tempo discrimination in 3- and
    4-year-old children. *Current Psychology Letters* 16(2). Full text.
14. Bobin-Bègue, Droit-Volet & Provasi (2014). *Frontiers in Psychology*
    5:1346, reporting Provasi & Bobin-Bègue (2003), *IJBD* 27, 220–231
    (the 2003 paper itself was not opened).
15. Kirschner & Tomasello (2009). Joint drumming. *J Exp Child Psychol*
    102, 299–314. Abstract.
16. Bonacina et al. (2021). Rhythm, reading, and sound processing in the
    brain in preschool children. *npj Science of Learning* 6:20. Full
    text.
17. Trehub (2023). Musicality in infancy. *Psychological Topics* 32,
    1–11. Abstract.
18. Corrigall & Trainor (2010). Musical enculturation in preschool
    children. *Music Perception* 28, 195–200. Abstract.
19. Welch (2009). Evidence of the development of vocal pitch matching
    ability in children. *Japanese Journal of Music Education Research*.
    Full text (cites Davidson 1994; Rutkowski 1997).
20. Corbeil, Trehub & Peretz (2013). Speech vs. singing: infants choose
    happier sounds. *Frontiers in Psychology* 4:372. Abstract.
21. Wikipedia, "Kodály method" (sequence and principles).
22. Alliance for Active Music Making, "Edwin Gordon" resource page.
23. AAP Council on Communications and Media (2016). Media and young
    minds. *Pediatrics* 138(5) e20162591, as reported by Healio
    (2016-10-21); the journal page refused access.
24. Hillenbrand, Getty, Clark & Wheeler (1995). *JASA* 97, 3099–3111;
    steady-state data as distributed in the R package phonTools (`h95`),
    child means computed here.
