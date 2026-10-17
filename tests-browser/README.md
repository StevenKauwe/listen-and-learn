# tests-browser — the acceptance rows that need a real engine

The five spec-§10 rows a headless core crate cannot test: no-scroll, real
pointer behavior, offline, rotate, standalone layout. Everything here runs
the built app in **real WebKit** (the Safari/iPhone engine family) with
**real PointerEvent dispatch** — pointerdown/move/up sequences carrying
pointerId, pointerType "touch", and pressure — never synthetic clicks.

## Tool decision (and why not Playwright)

**WebKitGTK 4.1 from Python**, in the same style as crabscape's
`tools/test-webkit.py`.

- The assignment prefers WebKit. WebKitGTK *is* WebKit; Playwright on Linux
  ships Chromium only (its embedded WebKit is macOS/Windows). For an
  iPhone-first PWA the right engine beats the bigger tool.
- `gi.require_version("WebKit2", "4.1")` imports clean on this machine —
  zero downloads. The Playwright path needs a browser fetch and yields the
  wrong engine.
- The offline row is more honest than a browser flag: the harness **kills
  the HTTP server** after the service worker takes control and reloads.
  Only the SW cache can serve the page; a fetch that needs the network
  fails for real.

Requirements: `webkit2gtk-4.1` + its Python bindings (`python-gobject`),
Gtk 3. The harness sets `WEBKIT_DISABLE_COMPOSITING_MODE=1` itself so it
runs on machines with no OpenGL (offscreen windows, CI).

## Run

```sh
python3 tests-browser/run_tests.py                 # against the fixture
python3 tests-browser/run_tests.py --url URL       # against the real app
```

Exit 0 iff every row passes. iPhone viewport (390×844, iPhone UA); the
rotate row resizes to 844×390 live.

## The fixture is the contract

`fixture/index.html` is a reference implementation of the input contract,
not the game. The same five rows run against it today and against the real
app when its build lands, by swapping the URL. The app must expose the same
DOM contract the fixture does:

| Observable | Where | Meaning |
|---|---|---|
| `body[data-guesses]` | counter | total guesses registered (one per gesture, slop-tolerant) |
| `body[data-last]` | name | which note the last guess was for |
| `body[data-audio]` | none/init/suspended/running/error | sound initialization state |
| `body[data-errors]` | counter | page errors (the offline row asserts zero) |
| `body[data-sw]` | registered/failed/unsupported | service worker registration |

Input semantics under test (spec §10): a drag within **20 px is still a
tap** (exactly one guess); a larger drag is not a tap and **never scrolls**
the page; a second simultaneous pointer is **ignored** while one is active;
sound initializes on first touch; buttons stay visible in landscape.

## Rows

1. Vertical swipe on a button → `scrollY` stays 0 (bonus: the big drag
   doesn't count as a tap).
2. Real 20 px-drag tap → exactly one guess, for the button actually touched.
3. Two simultaneous pointers → at most one guess.
4. SW active, server dead, reload → page serves from cache, a guess
   registers, sound initializes, zero page errors (no failed fetches).
5. Rotate to landscape → every button visible in the viewport.

The 2-second-hold row is implemented in the fixture (contract-complete)
and harness-ready; it currently lives with the logic rows in the core
crate per the assignment split.
