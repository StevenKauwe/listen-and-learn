# listen-and-learn

A series of free, browser-based learning toys. Pure Rust, compiled to WASM.
No accounts, no backend, no analytics, no ads.

## Project 1 — Do Re Mi

A toddler ear-training game (ages 2–4): the app plays a note; the child
taps the big colored button for the note they heard. Full spec:
`docs/do-re-mi-spec.md` (source of truth — behavior questions resolve there).

### Architecture

- `crates/doremi-core` — **all game logic, pure Rust, no browser**: the
  finite-state machine, the imprecise-touch gate, note tables, round
  scheduler, settings. Every behavioral acceptance row is a unit test here.
- `crates/doremi-app` — the Dioxus/WASM layer: renders state, forwards raw
  pointer events to the core, synthesizes notes via Web Audio (`web-sys`),
  makes **no decisions**.
- `web/` — static shell: `index.html`, PWA manifest, icons, and a ~10-line
  service worker (the one documented JS exception — SW is a browser API,
  not app logic).

### Build (no trunk needed)

Same pipeline as crabscape: `tools/build-web.sh` →
`cargo build --target wasm32-unknown-unknown` + `wasm-bindgen-cli` +
static `index.html`. Serve `web/` with any static file server.

### Testing

- `cargo test --workspace` — the acceptance matrix, headless (32 tests).
- `tests-browser/` — Playwright/WebKit harness for the rows that need a
  real browser (no-scroll, real pointer events, offline).

### Rules

- All rights reserved. No licence file.
- Facts about child development / audio claims in docs need real sources,
  or are marked "not verified".
- The core crate never depends on the app crate; UI stays dumb.
