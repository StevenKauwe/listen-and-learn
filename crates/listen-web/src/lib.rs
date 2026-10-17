//! Listen & Learn web layer: renders screens, forwards pointer events to
//! the core's touch gate and round machine, executes their effects (audio,
//! timers, animation). It makes no game decisions.

mod art;
mod audio;
mod view;

use audio::Audio;
use listen_core::settings::{HOLD_HOME_MS, HOLD_SETTINGS_MS};
use listen_core::{by_slug, Decision, Effect, Event, Phase, Round, Settings, TouchGate, ALL};
use std::cell::RefCell;
use view::{T_FULLSCREEN, T_LISTEN, T_TILE};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, Element, HtmlElement, HtmlInputElement, KeyboardEvent, PointerEvent, Window,
};

const STORE_KEY: &str = "listen-learn-settings";
/// Drift that cancels an adult hold (same as the tap tolerance).
const HOLD_SLOP: f64 = 35.0;

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok().and_then(|mut a| a.as_mut().map(f)))
}

fn after(ms: f64, f: impl FnOnce() + 'static) {
    if let Some(w) = web_sys::window() {
        let cb = Closure::once_into_js(f);
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.unchecked_ref(),
            ms.max(0.0) as i32,
        );
    }
}

fn listen<E: wasm_bindgen::convert::FromWasmAbi + 'static>(
    target: &web_sys::EventTarget,
    name: &str,
    passive: bool,
    f: impl FnMut(E) + 'static,
) {
    let c = Closure::<dyn FnMut(E)>::new(f);
    let opts = web_sys::AddEventListenerOptions::new();
    opts.set_passive(passive);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        name,
        c.as_ref().unchecked_ref(),
        &opts,
    );
    c.forget();
}

fn get(obj: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(obj, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

/// Call a method if it exists; returns its result (often a Promise).
fn call(obj: &JsValue, method: &str, args: &[JsValue]) -> Option<JsValue> {
    let f = get(obj, method).dyn_into::<js_sys::Function>().ok()?;
    let a = js_sys::Array::new();
    for x in args {
        a.push(x);
    }
    f.apply(obj, &a).ok()
}

/// Await a maybe-promise; `done(ok, value)` runs afterwards.
fn settle(v: Option<JsValue>, done: impl FnOnce(bool, JsValue) + 'static) {
    match v {
        Some(v) if v.is_instance_of::<js_sys::Promise>() => {
            let p: js_sys::Promise = v.unchecked_into();
            wasm_bindgen_futures::spawn_local(async move {
                match wasm_bindgen_futures::JsFuture::from(p).await {
                    Ok(x) => done(true, x),
                    Err(e) => done(false, e),
                }
            });
        }
        Some(v) => done(true, v),
        None => done(false, JsValue::UNDEFINED),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Home,
    Game(usize),
}

struct Hold {
    pointer: i32,
    kind: String,
    x: f64,
    y: f64,
    el: Element,
    id: u32,
}

struct App {
    win: Window,
    doc: Document,
    body: HtmlElement,
    root: Element,
    settings: Settings,
    audio: Option<Audio>,
    audio_failed: bool,
    gate: TouchGate,
    pressed: Option<Element>,
    hold: Option<Hold>,
    hold_id: u32,
    screen: Screen,
    round: Option<Round>,
    /// Bumped on every screen/round change; stale timers check it.
    epoch: u32,
    wake: Option<JsValue>,
    sheet_open: bool,
    reset_armed: bool,
    ios: bool,
    standalone: bool,
    errors: u32,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let win = web_sys::window().ok_or("no window")?;
    let doc = win.document().ok_or("no document")?;
    let body = doc.body().ok_or("no body")?;
    let root = doc.get_element_by_id("app").ok_or("no #app")?;
    let nav: JsValue = win.navigator().into();
    let ua = win.navigator().user_agent().unwrap_or_default();
    let touch_points = get(&nav, "maxTouchPoints").as_f64().unwrap_or(0.0);
    let ios = ua.contains("iPhone")
        || ua.contains("iPad")
        || ua.contains("iPod")
        || (ua.contains("Macintosh") && touch_points > 1.0);
    let standalone = get(&nav, "standalone").as_bool().unwrap_or(false)
        || win
            .match_media("(display-mode: standalone)")
            .ok()
            .flatten()
            .map(|m| m.matches())
            .unwrap_or(false);
    let settings = win
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(STORE_KEY).ok().flatten())
        .map(|t| Settings::decode(&t))
        .unwrap_or_default();

    let app = App {
        win: win.clone(),
        doc: doc.clone(),
        body,
        root,
        settings,
        audio: None,
        audio_failed: false,
        gate: TouchGate::new(),
        pressed: None,
        hold: None,
        hold_id: 0,
        screen: Screen::Home,
        round: None,
        epoch: 0,
        wake: None,
        sheet_open: false,
        reset_armed: false,
        ios,
        standalone,
        errors: 0,
    };
    APP.with(|a| *a.borrow_mut() = Some(app));

    let d: &web_sys::EventTarget = doc.as_ref();
    let w: &web_sys::EventTarget = win.as_ref();
    listen(w, "pointerdown", true, |e: PointerEvent| {
        with(|a| a.on_down(&e));
    });
    listen(w, "pointermove", true, |e: PointerEvent| {
        with(|a| a.on_move(&e));
    });
    listen(w, "pointerup", true, |e: PointerEvent| {
        with(|a| a.on_up(&e, false));
    });
    listen(w, "pointercancel", true, |e: PointerEvent| {
        with(|a| a.on_up(&e, true));
    });
    // iOS only lets audio start from an activating event; a touch's
    // pointerdown isn't one, its touchend is.
    listen(w, "touchend", true, |_: web_sys::Event| {
        with(|a| a.unlock_audio());
    });
    listen(w, "keydown", false, |e: KeyboardEvent| {
        with(|a| a.on_key(&e));
    });
    listen(w, "hashchange", true, |_: web_sys::Event| {
        with(|a| a.route());
    });
    listen(d, "visibilitychange", true, |_: web_sys::Event| {
        with(|a| a.on_visibility());
    });
    for name in ["error", "unhandledrejection"] {
        listen(w, name, true, |_: web_sys::Event| {
            with(|a| {
                a.errors += 1;
                a.body
                    .set_attribute("data-errors", &a.errors.to_string())
                    .ok();
            });
        });
    }
    // No scrolling, zoom, callouts or selection outside the adult sheet.
    listen(d, "touchmove", false, |e: web_sys::Event| {
        if !in_sheet_scroll(&e) {
            e.prevent_default();
        }
    });
    for name in [
        "contextmenu",
        "selectstart",
        "dblclick",
        "gesturestart",
        "gesturechange",
    ] {
        listen(d, name, false, |e: web_sys::Event| {
            if !in_sheet_scroll(&e) {
                e.prevent_default();
            }
        });
    }
    listen(d, "click", true, |e: web_sys::MouseEvent| {
        with(|a| a.on_click(&e));
    });
    listen(d, "input", true, |e: web_sys::Event| {
        with(|a| a.on_input(&e));
    });

    with(|a| {
        a.body.set_attribute("data-errors", "0").ok();
        a.body.set_attribute("data-guesses", "0").ok();
        a.body.set_attribute("data-audio", "none").ok();
        a.route();
    });
    Ok(())
}

fn in_sheet_scroll(e: &web_sys::Event) -> bool {
    e.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|el| el.closest(".sheet-scroll, input").ok().flatten())
        .is_some()
}

fn target_el(e: &web_sys::Event) -> Option<Element> {
    e.target().and_then(|t| t.dyn_into::<Element>().ok())
}

fn data_t(el: &Element) -> Option<(Element, u8)> {
    let t = el.closest("[data-t]").ok().flatten()?;
    let id = t.get_attribute("data-t")?.parse().ok()?;
    Some((t, id))
}

impl App {
    fn now_ms(&self) -> u32 {
        self.win.performance().map(|p| p.now() as u32).unwrap_or(0)
    }

    fn free(&self) -> bool {
        matches!(self.screen, Screen::Game(i) if ALL[i].free())
    }

    // ---------- audio ----------

    fn unlock_audio(&mut self) {
        if self.audio.is_none() && !self.audio_failed {
            match Audio::new(self.settings.volume) {
                Ok(a) => self.audio = Some(a),
                Err(_) => self.audio_failed = true,
            }
        }
        if let Some(a) = self.audio.as_mut() {
            a.unlock();
        }
        self.sync_audio_attr();
    }

    fn sync_audio_attr(&self) {
        let s = match &self.audio {
            Some(a) => a.state_name(),
            None if self.audio_failed => "error",
            None => "none",
        };
        self.body.set_attribute("data-audio", s).ok();
    }

    fn play(&mut self, s: &listen_core::music::Sound) {
        let Some(a) = self.audio.as_mut() else { return };
        let t0 = a.play(s);
        let lead = t0 - a.now();
        let ep = self.epoch;
        for t in &s.tones {
            if let Some(target) = t.light {
                let on = (lead + t.at as f64) * 1000.0;
                let off = on + t.dur as f64 * 800.0;
                after(on, move || {
                    with(|a| a.mark(ep, target, "lit", true));
                });
                after(off, move || {
                    with(|a| a.mark(ep, target, "lit", false));
                });
            }
        }
    }

    fn mark(&self, ep: u32, target: u8, class: &str, on: bool) {
        if ep != self.epoch {
            return;
        }
        if let Some(el) = self.target(target) {
            let cl = el.class_list();
            let _ = if on {
                cl.add_1(class)
            } else {
                cl.remove_1(class)
            };
        }
    }

    fn target(&self, t: u8) -> Option<Element> {
        self.root
            .query_selector(&format!(r#"#stage [data-t="{t}"]"#))
            .ok()
            .flatten()
    }

    // ---------- pointer input ----------

    fn on_down(&mut self, e: &PointerEvent) {
        self.unlock_audio();
        if self.sheet_open {
            return;
        }
        let Some(el) = target_el(e) else { return };
        let (x, y) = (e.client_x() as f64, e.client_y() as f64);
        if let Some(h) = el.closest("[data-hold]").ok().flatten() {
            if self.hold.is_none() {
                self.hold_start(h, e.pointer_id(), x, y);
            }
            return;
        }
        let Some((tel, id)) = data_t(&el) else { return };
        let open = self.is_open(id);
        let d = self.gate.down(
            e.pointer_id() as u32 as u64,
            id,
            x as f32,
            y as f32,
            self.now_ms(),
            open,
        );
        if d == Decision::Track {
            let _ = tel.class_list().add_1("down");
            let _ = tel.set_pointer_capture(e.pointer_id());
            self.pressed = Some(tel.clone());
            if self.free() && id < T_LISTEN {
                self.free_play(id, &tel);
            }
        }
    }

    fn on_move(&mut self, e: &PointerEvent) {
        let (x, y) = (e.client_x() as f64, e.client_y() as f64);
        if let Some(h) = &self.hold {
            if h.pointer == e.pointer_id()
                && ((x - h.x).powi(2) + (y - h.y).powi(2)).sqrt() > HOLD_SLOP
            {
                self.hold_cancel(false);
            }
        }
        if self
            .gate
            .r#move(e.pointer_id() as u32 as u64, x as f32, y as f32)
            == Decision::Cancelled
        {
            self.release_pressed();
        }
    }

    fn on_up(&mut self, e: &PointerEvent, cancel: bool) {
        if self
            .hold
            .as_ref()
            .map(|h| h.pointer == e.pointer_id())
            .unwrap_or(false)
        {
            self.hold_cancel(!cancel);
        }
        let pid = e.pointer_id() as u32 as u64;
        let d = if cancel {
            self.gate.cancel(pid)
        } else {
            self.gate.up(pid, self.now_ms())
        };
        match d {
            Decision::Guess(id) => {
                self.release_pressed();
                if !(self.free() && id < T_LISTEN) && self.is_open(id) {
                    self.accept(id);
                }
            }
            Decision::Cancelled => self.release_pressed(),
            _ => {}
        }
        self.unlock_audio();
    }

    fn release_pressed(&mut self) {
        if let Some(el) = self.pressed.take() {
            let _ = el.class_list().remove_1("down");
        }
    }

    fn reset_input(&mut self) {
        self.gate.reset();
        self.release_pressed();
        self.hold_cancel(false);
    }

    fn on_key(&mut self, e: &KeyboardEvent) {
        let key = e.key();
        if self.sheet_open {
            if key == "Escape" {
                self.close_sheet();
            }
            return;
        }
        if key != "Enter" && key != " " {
            return;
        }
        let Some(el) = self.doc.active_element() else {
            return;
        };
        e.prevent_default();
        self.unlock_audio();
        if let Some(h) = el.closest("[data-hold]").ok().flatten() {
            let kind = h.get_attribute("data-hold").unwrap_or_default();
            self.hold_fire(&kind);
        } else if let Some((tel, id)) = data_t(&el) {
            if self.is_open(id) {
                if self.free() && id < T_LISTEN {
                    self.free_play(id, &tel);
                } else {
                    self.accept(id);
                }
            }
        }
    }

    fn is_open(&self, id: u8) -> bool {
        match self.screen {
            Screen::Home => id == T_FULLSCREEN || id >= T_TILE,
            Screen::Game(_) if self.free() => id < T_LISTEN,
            Screen::Game(_) => match &self.round {
                Some(r) if id == T_LISTEN => r.accepts_listen(),
                Some(r) if id < T_LISTEN => r.accepts_answer(),
                _ => false,
            },
        }
    }

    fn accept(&mut self, id: u8) {
        match id {
            T_FULLSCREEN => self.fullscreen(),
            T_LISTEN => self.listen_pressed(),
            t if t >= T_TILE => {
                let i = (t - T_TILE) as usize;
                let Some(g) = ALL.get(i) else { return };
                self.play(&g.hint());
                if let Some(el) = self
                    .root
                    .query_selector(&format!(r#"[data-t="{t}"]"#))
                    .ok()
                    .flatten()
                {
                    let _ = el.class_list().add_1("chosen");
                }
                let slug = g.slug();
                let ep = self.epoch;
                after(420.0, move || {
                    with(|a| {
                        if a.epoch == ep {
                            a.win.location().set_hash(slug).ok();
                        }
                    });
                });
            }
            t => {
                let fx = match self.round.as_mut() {
                    Some(r) => r.handle(Event::Answer(t)),
                    None => return,
                };
                self.run(fx);
            }
        }
    }

    // ---------- adult holds ----------

    fn hold_start(&mut self, el: Element, pointer: i32, x: f64, y: f64) {
        let kind = el.get_attribute("data-hold").unwrap_or_default();
        let ms = if kind == "home" {
            HOLD_HOME_MS
        } else {
            HOLD_SETTINGS_MS
        };
        el.set_attribute("style", &format!("--hold:{ms}ms")).ok();
        let _ = el.class_list().remove_1("nudge");
        let _ = el.class_list().add_1("holding");
        self.hold_id += 1;
        let id = self.hold_id;
        self.hold = Some(Hold {
            pointer,
            kind,
            x,
            y,
            el,
            id,
        });
        after(ms as f64, move || {
            with(|a| {
                if a.hold.as_ref().map(|h| h.id == id).unwrap_or(false) {
                    let h = a.hold.take().unwrap();
                    let _ = h.el.class_list().remove_1("holding");
                    a.hold_fire(&h.kind);
                }
            });
        });
    }

    fn hold_cancel(&mut self, nudge: bool) {
        if let Some(h) = self.hold.take() {
            let _ = h.el.class_list().remove_1("holding");
            if nudge {
                let _ = h.el.class_list().add_1("nudge");
                let el = h.el.clone();
                after(1400.0, move || {
                    let _ = el.class_list().remove_1("nudge");
                });
            }
        }
    }

    fn hold_fire(&mut self, kind: &str) {
        if kind == "home" {
            self.win.location().set_hash("").ok();
        } else {
            self.open_sheet();
        }
    }

    // ---------- screens ----------

    fn route(&mut self) {
        let hash = self.win.location().hash().unwrap_or_default();
        match by_slug(hash.trim_start_matches('#')) {
            Some((i, _)) => self.enter_game(i),
            None => self.enter_home(),
        }
    }

    fn leave(&mut self) {
        self.epoch += 1;
        self.reset_input();
        if let Some(a) = self.audio.as_mut() {
            a.stop_all();
        }
        self.round = None;
    }

    fn enter_home(&mut self) {
        self.leave();
        self.screen = Screen::Home;
        let fs = ["requestFullscreen", "webkitRequestFullscreen"]
            .iter()
            .any(|m| {
                get(
                    &self
                        .doc
                        .document_element()
                        .map(JsValue::from)
                        .unwrap_or_default(),
                    m,
                )
                .is_function()
            });
        self.root
            .set_inner_html(&view::home(fs, self.ios && !self.standalone));
        self.body.set_attribute("data-route", "home").ok();
        self.body.set_attribute("data-phase", "home").ok();
        self.unwake();
    }

    fn enter_game(&mut self, i: usize) {
        self.leave();
        self.screen = Screen::Game(i);
        let g = ALL[i];
        let seed = (js_sys::Date::now() as u64 as u32) ^ ((js_sys::Math::random() * 4e9) as u32);
        if !g.free() {
            self.round = Some(Round::new(g, self.settings.round_cfg(i), seed));
        }
        self.root
            .set_inner_html(&view::game(g, self.settings.levels[i]));
        self.body.set_attribute("data-route", g.slug()).ok();
        self.wake();
        self.sync();
    }

    // ---------- the round ----------

    fn listen_pressed(&mut self) {
        self.unlock_audio();
        if self.audio.is_none() {
            return self.feed(Event::AudioFailed);
        }
        self.await_audio(self.epoch, 0);
    }

    /// Start the question once audio really runs; give up after ~1 s.
    fn await_audio(&mut self, ep: u32, tries: u32) {
        if ep != self.epoch {
            return;
        }
        let running = self.audio.as_ref().map(|a| a.running()).unwrap_or(false);
        self.sync_audio_attr();
        if running {
            self.feed(Event::Listen);
        } else if tries >= 20 {
            self.feed(Event::AudioFailed);
        } else {
            after(50.0, move || {
                with(|a| a.await_audio(ep, tries + 1));
            });
        }
    }

    fn feed(&mut self, ev: Event) {
        let fx = match self.round.as_mut() {
            Some(r) => r.handle(ev),
            None => return,
        };
        self.run(fx);
    }

    fn run(&mut self, fx: Vec<Effect>) {
        for e in fx {
            match e {
                Effect::Play(s) => self.play(&s),
                Effect::StopAudio => {
                    if let Some(a) = self.audio.as_mut() {
                        a.stop_all();
                    }
                    if let Ok(list) = self.root.query_selector_all(".lit") {
                        for i in 0..list.length() {
                            if let Some(el) =
                                list.item(i).and_then(|n| n.dyn_into::<Element>().ok())
                            {
                                let _ = el.class_list().remove_1("lit");
                            }
                        }
                    }
                }
                Effect::Wait {
                    seq,
                    secs,
                    audio_clock,
                } => {
                    let end = self
                        .audio
                        .as_ref()
                        .map(|a| a.now() + 0.03 + secs as f64)
                        .unwrap_or(0.0);
                    self.wait(
                        self.epoch,
                        seq,
                        end,
                        audio_clock,
                        secs as f64 * 1000.0 + 40.0,
                    );
                }
                Effect::Right { target, name } => {
                    let text = match name {
                        Some(n) => format!("{}<span>{n}!</span>", art::STAR),
                        None => format!("{}<span>Yes!</span>", art::STAR),
                    };
                    self.set_msg(&text, "yay");
                    if let Some(el) = self.target(target) {
                        self.flash_class(&el, "yay", 900.0);
                        self.sparks(&el, target);
                    }
                }
                Effect::Wrong { target } => {
                    self.set_msg("<span>Try again</span>", "soft");
                    if let Some(el) = self.target(target) {
                        self.flash_class(&el, "oops", 600.0);
                    }
                }
                Effect::Progress { done, total } => {
                    if let (Some(b), Some(r)) = (self.doc.get_element_by_id("beads"), &self.round) {
                        if total > 1 {
                            b.set_inner_html(&view::beads(&r.q.answer, done));
                        }
                    }
                }
                Effect::Hint(t) => {
                    if let Ok(list) = self.root.query_selector_all(".glow") {
                        for i in 0..list.length() {
                            if let Some(el) =
                                list.item(i).and_then(|n| n.dyn_into::<Element>().ok())
                            {
                                let _ = el.class_list().remove_1("glow");
                            }
                        }
                    }
                    if let Some(el) = t.and_then(|t| self.target(t)) {
                        let _ = el.class_list().add_1("glow");
                    }
                }
                Effect::Speak(say) => after(450.0, move || speak(say)),
            }
        }
        self.sync();
    }

    fn wait(&mut self, ep: u32, seq: u32, end: f64, audio_clock: bool, ms: f64) {
        after(ms, move || {
            with(|a| {
                if a.epoch != ep {
                    return;
                }
                if audio_clock {
                    match a.audio.as_ref() {
                        Some(au) if au.running() => {
                            let left = end - au.now();
                            if left > 0.02 {
                                return a.wait(ep, seq, end, true, left * 1000.0 + 20.0);
                            }
                        }
                        _ => return a.feed(Event::AudioFailed),
                    }
                }
                a.feed(Event::Timer(seq));
            });
        });
    }

    fn sync(&mut self) {
        self.sync_audio_attr();
        let Some(r) = &self.round else {
            if self.free() {
                self.body.set_attribute("data-phase", "free").ok();
            }
            return;
        };
        let phase = match r.phase {
            Phase::Ready => "ready",
            Phase::Question => "question",
            Phase::Awaiting => "awaiting",
            Phase::Right => "right",
            Phase::Wrong => "wrong",
        };
        self.body.set_attribute("data-phase", phase).ok();
        self.body
            .set_attribute("data-retry", if r.retry { "1" } else { "0" })
            .ok();
        self.body
            .set_attribute("data-guesses", &r.guesses.to_string())
            .ok();
        self.body
            .set_attribute("data-right", &r.rights.to_string())
            .ok();
        self.body
            .set_attribute("data-wrong", &r.wrongs.to_string())
            .ok();
        // For the browser harness only; never rendered.
        let ans: Vec<String> = r.q.answer.iter().map(|a| a.to_string()).collect();
        self.body.set_attribute("data-answer", &ans.join(",")).ok();
        let echo = r.q.answer.len() > 1;
        let (text, mood) = match r.phase {
            Phase::Ready if r.retry => ("Tap again", "soft"),
            Phase::Ready => ("Listen!", ""),
            Phase::Question => ("Listen…", "hush"),
            Phase::Awaiting if echo => ("Your turn", ""),
            Phase::Awaiting => ("Which one?", ""),
            _ => return,
        };
        self.set_msg(&format!("<span>{text}</span>"), mood);
    }

    fn set_msg(&self, html: &str, mood: &str) {
        if let Some(m) = self.doc.get_element_by_id("msg") {
            if m.inner_html() != html {
                m.set_inner_html(html);
            }
            m.set_class_name(&format!("msg {mood}"));
        }
    }

    fn flash_class(&self, el: &Element, class: &str, ms: f64) {
        let cl = el.class_list();
        let _ = cl.remove_1(class);
        // Force a style flush so the animation restarts.
        let _ = el.get_bounding_client_rect();
        let _ = cl.add_1(class);
        let el = el.clone();
        let class = class.to_string();
        after(ms, move || {
            let _ = el.class_list().remove_1(&class);
        });
    }

    fn sparks(&self, el: &Element, target: u8) {
        let Some(fx) = self.doc.get_element_by_id("fx") else {
            return;
        };
        let r = el.get_bounding_client_rect();
        let (cx, cy) = (r.left() + r.width() / 2.0, r.top() + r.height() / 2.0);
        let keys = ["do", "re", "mi", "fa", "sol", "la", "ti"];
        for i in 0..9 {
            let Ok(s) = self.doc.create_element("i") else {
                continue;
            };
            let ang = i as f64 / 9.0 * std::f64::consts::TAU + js_sys::Math::random() * 0.5;
            let dist = 70.0 + js_sys::Math::random() * 50.0;
            let key = keys[(i + target as usize) % 7];
            s.set_class_name("spark");
            s.set_attribute(
                "style",
                &format!(
                    "left:{cx:.0}px;top:{cy:.0}px;--dx:{:.0}px;--dy:{:.0}px;--c:var(--{key})",
                    ang.cos() * dist,
                    ang.sin() * dist
                ),
            )
            .ok();
            let _ = fx.append_child(&s);
            after(1000.0, move || s.remove());
        }
    }

    fn free_play(&mut self, id: u8, el: &Element) {
        let i = match self.screen {
            Screen::Game(i) => i,
            _ => return,
        };
        let g = ALL[i];
        let k = self.settings.round_cfg(i).knobs;
        self.play(&g.answer_sound(k, id));
        self.flash_class(el, "yay", 600.0);
        let n = self
            .body
            .get_attribute("data-guesses")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        self.body
            .set_attribute("data-guesses", &(n + 1).to_string())
            .ok();
        let Some(fx) = self.doc.get_element_by_id("fx") else {
            return;
        };
        let r = el.get_bounding_client_rect();
        if let Ok(s) = self.doc.create_element("i") {
            let key = listen_core::music::SCALE[id as usize].key;
            s.set_class_name("fly");
            s.set_inner_html(art::NOTE);
            let x = r.left() + r.width() * (0.3 + js_sys::Math::random() * 0.4);
            s.set_attribute(
                "style",
                &format!(
                    "left:{x:.0}px;top:{:.0}px;color:var(--{key})",
                    r.top() + r.height() / 2.0
                ),
            )
            .ok();
            let _ = fx.append_child(&s);
            after(1300.0, move || s.remove());
        }
    }

    // ---------- page lifecycle ----------

    fn on_visibility(&mut self) {
        if self.doc.hidden() {
            self.reset_input();
            if self.round.is_some() {
                self.feed(Event::Interrupted);
            } else if let Some(a) = self.audio.as_mut() {
                a.stop_all();
            }
        } else if matches!(self.screen, Screen::Game(_)) {
            self.wake();
        }
        self.sync_audio_attr();
    }

    fn wake(&mut self) {
        let wl = get(&self.win.navigator().into(), "wakeLock");
        if wl.is_undefined() || self.wake.is_some() {
            return;
        }
        settle(
            call(&wl, "request", &[JsValue::from_str("screen")]),
            |ok, v| {
                if ok {
                    with(|a| a.wake = Some(v));
                }
            },
        );
    }

    fn unwake(&mut self) {
        if let Some(s) = self.wake.take() {
            settle(call(&s, "release", &[]), |_, _| {});
        }
    }

    fn fullscreen(&mut self) {
        let doc: JsValue = self.doc.clone().into();
        let active = !get(&doc, "fullscreenElement").is_null()
            && !get(&doc, "fullscreenElement").is_undefined()
            || (!get(&doc, "webkitFullscreenElement").is_null()
                && !get(&doc, "webkitFullscreenElement").is_undefined());
        if active {
            let r = call(&doc, "exitFullscreen", &[])
                .or_else(|| call(&doc, "webkitExitFullscreen", &[]));
            settle(r, |_, _| {});
            return;
        }
        let de: JsValue = self
            .doc
            .document_element()
            .map(Into::into)
            .unwrap_or_default();
        let r = call(&de, "requestFullscreen", &[])
            .or_else(|| call(&de, "webkitRequestFullscreen", &[]));
        settle(r, |ok, _| {
            if !ok {
                with(|a| a.toast("Full screen isn't available here"));
            }
        });
    }

    fn toast(&self, text: &str) {
        if let Some(t) = self.doc.get_element_by_id("toast") {
            t.set_text_content(Some(text));
            let _ = t.class_list().add_1("show");
            let t = t.clone();
            after(2600.0, move || {
                let _ = t.class_list().remove_1("show");
            });
        }
    }

    // ---------- adult settings ----------

    fn current_game(&self) -> Option<usize> {
        match self.screen {
            Screen::Game(i) => Some(i),
            Screen::Home => None,
        }
    }

    fn open_sheet(&mut self) {
        self.reset_input();
        if self.round.is_some() {
            self.feed(Event::Interrupted);
        }
        self.render_sheet();
        if let Some(s) = self.doc.get_element_by_id("sheet") {
            s.remove_attribute("hidden").ok();
        }
        self.sheet_open = true;
        self.reset_armed = false;
    }

    fn render_sheet(&self) {
        if let Some(b) = self.doc.get_element_by_id("sheet-body") {
            b.set_inner_html(&view::sheet(&self.settings, self.current_game()));
        }
    }

    fn close_sheet(&mut self) {
        if let Some(s) = self.doc.get_element_by_id("sheet") {
            s.set_attribute("hidden", "").ok();
        }
        self.sheet_open = false;
        self.save();
        if let Screen::Game(i) = self.screen {
            self.enter_game(i);
        }
    }

    fn save(&self) {
        if let Ok(Some(s)) = self.win.local_storage() {
            let _ = s.set_item(STORE_KEY, &self.settings.encode());
        }
    }

    fn on_click(&mut self, e: &web_sys::MouseEvent) {
        if !self.sheet_open {
            return;
        }
        let Some(el) = target_el(e) else { return };
        if el.id() == "sheet" {
            return self.close_sheet(); // tap on the dimmed backdrop
        }
        let Some(b) = el.closest("[data-act]").ok().flatten() else {
            return;
        };
        let act = b.get_attribute("data-act").unwrap_or_default();
        let parts: Vec<&str> = act.split(':').collect();
        match parts.as_slice() {
            ["close"] => return self.close_sheet(),
            ["lv", i, l] => {
                if let (Ok(i), Ok(l)) = (i.parse::<usize>(), l.parse::<u8>()) {
                    if i < self.settings.levels.len() {
                        self.settings.levels[i] = l;
                    }
                }
            }
            ["tog", key] => {
                let s = &mut self.settings;
                match *key {
                    "speak" => s.speak = !s.speak,
                    "cheer" => s.cheer = !s.cheer,
                    "auto_next" => s.auto_next = !s.auto_next,
                    "assist" => s.assist = !s.assist,
                    _ => {}
                }
            }
            ["reset"] => {
                if !self.reset_armed {
                    self.reset_armed = true;
                    b.set_text_content(Some("Tap again to reset"));
                    return;
                }
                self.settings = Settings::default();
                if let Some(a) = &self.audio {
                    a.set_volume(self.settings.volume);
                }
                self.reset_armed = false;
            }
            _ => return,
        }
        self.settings = self.settings.clone().sanitized();
        self.save();
        let scroll = self
            .doc
            .query_selector(".sheet-scroll")
            .ok()
            .flatten()
            .map(|s| s.scroll_top());
        self.render_sheet();
        if let (Some(top), Some(s)) = (
            scroll,
            self.doc.query_selector(".sheet-scroll").ok().flatten(),
        ) {
            s.set_scroll_top(top);
        }
    }

    fn on_input(&mut self, e: &web_sys::Event) {
        let Some(input) = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let v: f32 = input.value().parse().unwrap_or(0.0);
        match input.id().as_str() {
            "s-volume" => {
                self.settings.volume = (v / 100.0).clamp(0.0, 1.0);
                if let Some(a) = &self.audio {
                    a.set_volume(self.settings.volume);
                }
                self.out("o-volume", &format!("{}%", v.round()));
            }
            "s-len" => {
                self.settings.note_len = v.clamp(0.8, 2.0);
                self.out("o-len", &format!("{:.1} s", self.settings.note_len));
            }
            _ => return,
        }
        self.save();
    }

    fn out(&self, id: &str, text: &str) {
        if let Some(o) = self.doc.get_element_by_id(id) {
            o.set_text_content(Some(text));
        }
    }
}

fn speak(say: &str) {
    let Some(w) = web_sys::window() else { return };
    let Ok(synth) = w.speech_synthesis() else {
        return;
    };
    if let Ok(u) = web_sys::SpeechSynthesisUtterance::new_with_text(say) {
        u.set_rate(0.85);
        u.set_pitch(1.15);
        synth.speak(&u);
    }
}
