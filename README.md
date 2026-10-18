# Listen & Learn

Ear-training games for children aged 2 to 5, played on a phone: Do Re Mi
(note names sung on their pitch), Bird or Fish (high and low), Same or
Different, Up or Down, Echo, Turtle or Rabbit (slow and fast), Mouse or
Elephant (quiet and loud), Who's singing? (timbre), Drum along (steady
beat) and a free-play xylophone. One age setting calibrates every game. Pure Rust compiled to WebAssembly. No accounts, no backend, no
analytics, no ads.

Play: https://stevenkauwe.github.io/listen-and-learn/

- [docs/spec.md](docs/spec.md): shared rules for every game (touch, audio,
  feedback, settings, platform, architecture, acceptance rows).
- [docs/games.md](docs/games.md): each game's rules and ladders.
- [docs/research.md](docs/research.md): the evidence behind the games,
  ages and numbers.

## Layout

- `crates/listen-core`: all game logic, headless and unit-tested.
- `crates/listen-web`: the browser layer (`wasm-bindgen`, `web-sys`).
- `web/`: the page body and CSS, PWA manifest, icons, service worker.
- `tests-browser/`: the real-WebKit harness.
- `tools/build.sh`: builds `dist/` (static site) and `dist/artifact/`.

## Build and test

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
cargo test --workspace
tools/build.sh                       # wasm-opt on PATH is used if present
python3 -m http.server -d dist 8000  # then open http://localhost:8000/
python3 tests-browser/suite.py       # needs WebKitGTK 4.1 + PyGObject
```

On iPhone: open the site in Safari, Share, Add to Home Screen. The Home
Screen app runs full screen and works offline after the first visit.

All rights reserved.
