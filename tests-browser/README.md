# tests-browser

`suite.py` runs the built site (`dist/`, from `tools/build.sh`) in real
WebKit: WebKitGTK 4.1 driven from Python, the Safari engine family.
Playwright is not used because its Linux build has no WebKit.

```sh
python3 tests-browser/suite.py [--dist DIR] [--shots DIR]
```

Needs `webkit2gtk-4.1`, PyGObject and GTK 3. Exit 0 iff every row passes.

Input is dispatched as full `PointerEvent` sequences (`pointerdown`,
`pointermove`, `pointerup`, with `pointerId` and `pointerType: "touch"`),
never synthetic clicks. Rows: see spec §5, column "browser".

Audio: without a GStreamer audio sink (`autoaudiosink`) the page's audio
context stays suspended. The harness first checks that the game then shows
retry and keeps answers closed, then injects a shim that reports the
context as running and drives `currentTime` from `performance.now()`, so
every game can be played through. Real nodes are still created. Sound
itself is checked by ear on a phone.

The page exposes `data-route`, `data-phase`, `data-retry`, `data-audio`,
`data-guesses`, `data-right`, `data-wrong`, `data-answer`, `data-hits`,
`data-streak` (Drum along) and `data-errors` on `<body>` for the harness. None of them is rendered.
