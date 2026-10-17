#!/usr/bin/env python3
"""Browser rows for the whole suite (spec §5), in real WebKit (WebKitGTK).

    python3 tests-browser/suite.py [--dist DIR] [--shots DIR]

Serves the built site (default: dist/), opens it at iPhone size and checks,
for the home screen and every game: no page errors, no scrolling, every
target inside the viewport in portrait and landscape; then plays each game
through one wrong and one right answer with real PointerEvent sequences.
Also: a 100 px swipe never answers, a tap while the question plays never
answers, a second finger is ignored. Exit 0 iff every row passes.
"""

import functools
import http.server
import json
import os
import sys
import threading
import time

os.environ.setdefault("WEBKIT_DISABLE_COMPOSITING_MODE", "1")

import gi  # noqa: E402

gi.require_version("Gtk", "3.0")
gi.require_version("WebKit2", "4.1")
from gi.repository import GLib, Gtk, WebKit2  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


DIST = os.path.abspath(arg("--dist", os.path.join(ROOT, "dist")))
SHOTS = arg("--shots", None)
GAMES = ["notes", "birdfish", "twins", "stairs", "echo", "tempo", "loud", "play"]
IPHONE = ("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 "
          "(KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")

failures = 0


def check(ok, what):
    global failures
    print(("ok   " if ok else "FAIL ") + what, flush=True)
    failures += 0 if ok else 1


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      ".js": "text/javascript", ".wasm": "application/wasm"}

    def log_message(self, *a):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=DIST))
threading.Thread(target=server.serve_forever, daemon=True).start()
BASE = f"http://127.0.0.1:{server.server_address[1]}/index.html"

# Machines without a GStreamer audio sink (autoaudiosink) never get a
# running AudioContext. After checking the game's audio-failure path, the
# harness swaps in a clock shim: real nodes, but state "running" and a
# currentTime that follows performance.now().
AUDIO_SHIM = """(() => {
  const P = (window.BaseAudioContext || window.AudioContext).prototype, t0 = performance.now();
  Object.defineProperty(P, "state", { get() { return "running"; } });
  Object.defineProperty(P, "currentTime", { get() { return (performance.now() - t0) / 1000; } });
  AudioContext.prototype.resume = function () { return Promise.resolve(); };
})();"""
ctx = WebKit2.WebContext.new_ephemeral()
ucm = WebKit2.UserContentManager()
view = WebKit2.WebView(web_context=ctx, user_content_manager=ucm)
s = view.get_settings()
s.set_enable_write_console_messages_to_stdout(True)
s.set_user_agent(IPHONE)
s.set_media_playback_requires_user_gesture(False)
window = Gtk.OffscreenWindow()
window.set_default_size(390, 844)
view.set_size_request(390, 844)
window.add(view)
window.show_all()


def pump(sec):
    end = time.time() + sec
    c = GLib.MainContext.default()
    while time.time() < end:
        c.iteration(False)
        time.sleep(0.004)


def js(script, timeout=20):
    box = {}
    body = f"return JSON.stringify(await (async () => {{ return {script}; }})());"

    def done(v, res):
        try:
            box["v"] = v.call_async_javascript_function_finish(res).to_string()
        except Exception as e:  # noqa: BLE001
            box["e"] = str(e)

    view.call_async_javascript_function(body, -1, None, None, None, None, done)
    end = time.time() + timeout
    while "v" not in box and "e" not in box and time.time() < end:
        pump(0.02)
    if "e" in box:
        raise RuntimeError(box["e"])
    v = box.get("v")
    return json.loads(v) if v not in (None, "undefined") else None


def wait_for(expr, timeout=8):
    end = time.time() + timeout
    while time.time() < end:
        if js(expr):
            return True
        pump(0.05)
    return False


def attr(name):
    return js(f"document.body.dataset.{name}")


def resize(w, h):
    # An offscreen window will not shrink; move the view into a new one.
    global window
    window.remove(view)
    window.destroy()
    window = Gtk.OffscreenWindow()
    window.set_default_size(w, h)
    view.set_size_request(w, h)
    window.add(view)
    window.show_all()
    pump(0.6)
    got = js("[innerWidth, innerHeight]")
    check(got == [w, h], f"viewport is {w}x{h} (got {got})")


def shot(name):
    if not SHOTS:
        return
    os.makedirs(SHOTS, exist_ok=True)
    box = {}

    def done(v, res):
        try:
            box["s"] = v.get_snapshot_finish(res)
        except Exception as e:  # noqa: BLE001
            box["e"] = e

    view.get_snapshot(WebKit2.SnapshotRegion.VISIBLE, WebKit2.SnapshotOptions.NONE, None, done)
    end = time.time() + 10
    while not box and time.time() < end:
        pump(0.05)
    if "s" in box:
        box["s"].write_to_png(os.path.join(SHOTS, name + ".png"))


def ptr(kind, sel, dx=0, dy=0, pid=1):
    return js(f"""(() => {{
      const el = document.querySelector('{sel}');
      if (!el) return false;
      const r = el.getBoundingClientRect();
      const o = {{ bubbles: true, cancelable: true, composed: true, clientX: r.left + r.width / 2 + {dx},
        clientY: r.top + r.height / 2 + {dy}, pointerId: {pid}, pointerType: "touch",
        isPrimary: {str(pid == 1).lower()}, width: 20, height: 20, pressure: .5, button: 0, buttons: 1 }};
      const t = document.elementFromPoint(o.clientX - {dx}, o.clientY - {dy}) || el;
      t.dispatchEvent(new PointerEvent("{kind}", o));
      return true;
    }})()""")


def tap(sel, drift=(6, 4), pid=1):
    ptr("pointerdown", sel, 0, 0, pid)
    pump(0.03)
    ptr("pointermove", sel, *drift, pid)
    pump(0.03)
    return ptr("pointerup", sel, *drift, pid)


def swipe(sel, dy=100):
    ptr("pointerdown", sel)
    for i in range(1, 6):
        ptr("pointermove", sel, 0, dy * i / 5)
        pump(0.02)
    ptr("pointerup", sel, 0, dy)


FITS = """(() => {
  const vw = innerWidth, vh = innerHeight, bad = [];
  if (document.documentElement.scrollHeight > vh + 1 || document.documentElement.scrollWidth > vw + 1) bad.push("page");
  document.querySelectorAll("[data-t],[data-hold]").forEach((el) => {
    const r = el.getBoundingClientRect();
    if (!el.getClientRects().length) return;
    if (r.width < 1 || r.top < -1 || r.left < -1 || r.bottom > vh + 1 || r.right > vw + 1) bad.push(el.dataset.t || el.dataset.hold);
  });
  return bad.join(",");
})()"""


def fits(label):
    bad = js(FITS)
    check(bad == "" and js("scrollY === 0"), f"{label}: fits, no scroll (outside: {bad or 'none'})")


def go(slug):
    js(f"(location.hash = '{slug}', true)")
    return wait_for(f"document.body.dataset.route === '{slug or 'home'}'")


def play_game(slug):
    check(go(slug), f"{slug}: route opens")
    fits(f"{slug} portrait")
    if slug == "play":
        before = int(attr("guesses") or 0)
        tap('#stage [data-t="0"]')
        pump(0.2)
        check(int(attr("guesses") or 0) == before + 1, "play: a bar sounds at contact (one per touch)")
        return
    check(attr("phase") == "ready", f"{slug}: starts ready")
    first = '#stage [data-t="0"]'
    tap(first)
    pump(0.1)
    check(attr("guesses") == "0", f"{slug}: answer before Listen ignored")
    tap("#listen")
    check(wait_for("document.body.dataset.phase === 'question'", 3), f"{slug}: Listen plays the question (audio {attr('audio')})")
    tap(first)
    pump(0.05)
    check(attr("guesses") == "0", f"{slug}: tap while the question plays ignored")
    check(wait_for("document.body.dataset.phase === 'awaiting'", 8), f"{slug}: answers open after the question")
    answer = [int(x) for x in attr("answer").split(",")]
    targets = js("[...document.querySelectorAll('#stage [data-t]')].map(e => +e.dataset.t)")
    wrong = next(t for t in targets if t != answer[0])
    swipe(f'#stage [data-t="{wrong}"]', 100)
    pump(0.1)
    check(attr("guesses") == "0", f"{slug}: 100 px swipe is not an answer")
    # Two fingers: the second is ignored, the first answers once.
    ptr("pointerdown", f'#stage [data-t="{wrong}"]', pid=1)
    ptr("pointerdown", f'#stage [data-t="{answer[0]}"]', pid=2)
    ptr("pointerup", f'#stage [data-t="{answer[0]}"]', pid=2)
    ptr("pointerup", f'#stage [data-t="{wrong}"]', pid=1)
    pump(0.1)
    check(attr("guesses") == "1" and attr("wrong") == "1", f"{slug}: wrong answer counted once, second finger ignored")
    check(attr("phase") == "wrong", f"{slug}: gentle wrong state")
    if slug == "notes":
        shot("game-notes-wrong")
    check(wait_for("document.body.dataset.phase === 'awaiting'", 8), f"{slug}: same question reopens")
    check([int(x) for x in attr("answer").split(",")] == answer, f"{slug}: same question stays")
    for a in answer:
        tap(f'#stage [data-t="{a}"]')
        pump(0.08)
    check(attr("right") == "1", f"{slug}: right answer celebrated")
    pump(0.3)
    if slug in ("notes", "birdfish", "stairs"):
        shot(f"game-{slug}-right")
    check(wait_for("document.body.dataset.phase === 'question'", 4), f"{slug}: next round starts by itself")


def main():
    view.load_uri(BASE + "#notes")
    check(wait_for("document.body.dataset.route === 'notes'", 20), "wasm boots, deep link opens a game")
    pump(0.5)
    tap("#listen")
    wait_for("document.body.dataset.retry === '1' || document.body.dataset.audio === 'running'", 5)
    real = attr("audio")
    if real != "running":
        check(attr("phase") == "ready" and attr("retry") == "1",
              f"audio that cannot start (state {real}) never opens answers and shows retry "
              f"(phase {attr('phase')}, retry {attr('retry')}, errors {attr('errors')})")
        ucm.add_script(WebKit2.UserScript.new(AUDIO_SHIM, WebKit2.UserContentInjectedFrames.TOP_FRAME,
                                              WebKit2.UserScriptInjectionTime.START, None, None))
    view.load_uri(BASE)
    check(wait_for("document.body.dataset.route === 'home'", 20), "home loads")
    pump(1.0)
    fits("home portrait")
    shot("home-portrait")
    check(attr("errors") == "0", "no page errors on load")
    # A tile tap plays its hint and opens the game.
    tap('[data-t="110"]')
    check(wait_for("document.body.dataset.route === 'notes'", 3), "home tile opens its game")
    pump(0.4)
    shot("game-notes-ready")
    for g in GAMES:
        play_game(g)
    # Hold-to-home: a short tap does nothing, a 1.5 s hold goes home.
    go("birdfish")
    shot("game-birdfish")
    tap('[data-hold="home"]')
    pump(0.3)
    check(attr("route") == "birdfish", "short tap on home does not leave")
    ptr("pointerdown", '[data-hold="home"]')
    pump(1.7)
    ptr("pointerup", '[data-hold="home"]')
    check(wait_for("document.body.dataset.route === 'home'", 2), "1.5 s hold on home leaves the game")
    ptr("pointerdown", '[data-hold="gear"]')
    pump(2.2)
    ptr("pointerup", '[data-hold="gear"]')
    check(js("!document.getElementById('sheet').hidden"), "2 s hold on the gear opens settings")
    shot("settings")
    js("(document.querySelector('[data-act=close]').click(), true)")
    check(js("document.getElementById('sheet').hidden"), "settings close")
    resize(844, 390)
    fits("home landscape")
    shot("home-landscape")
    for g in GAMES:
        go(g)
        pump(0.15)
        fits(f"{g} landscape")
        if g in ("notes", "twins"):
            shot(f"game-{g}-landscape")
    resize(390, 844)
    for g in ["twins", "tempo", "loud", "echo", "play"]:
        go(g)
        pump(0.3)
        shot(f"game-{g}")
    js("(document.documentElement.dataset.theme = 'light', true)")
    for g in ["", "notes", "birdfish", "loud"]:
        go(g)
        pump(0.3)
        shot(f"light-{g or 'home'}")
    check(attr("errors") == "0", "no page errors after the whole run")
    print(f"\n{'ALL ROWS PASS' if failures == 0 else str(failures) + ' ROW(S) FAILED'}  [WebKitGTK 2.{WebKit2.get_minor_version()}]")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    finally:
        server.shutdown()
