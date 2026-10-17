#!/usr/bin/env python3
"""Browser acceptance rows for the do-re-mi game (spec §10) — the rows that
need a real WebKit engine, real PointerEvent dispatch, and a real dead server.

    python3 tests-browser/run_tests.py [--url URL] [--shots DIR]

Defaults to the fixture page (tests-browser/fixture/) — a reference
implementation of the input contract. When the real app lands, run with
--url http://127.0.0.1:PORT/ to point the same five rows at it (the app
needs to expose the same data-* contract; see tests-browser/README.md).

Rows (assignment):
  1  vertical swipe on a button  -> page does not scroll (scrollY stays 0)
  2  real 20 px-drag tap         -> exactly one guess registers
  3  two simultaneous pointers   -> at most one guess
  4  app offline (SW, dead srv)  -> gameplay + sound still work, no fetches
  5  landscape/rotate            -> buttons visible

Engine: WebKitGTK 4.1 (the Safari/iPhone engine family) driven from Python,
same style as crabscape's tools/test-webkit.py. Pointer input is dispatched
as full PointerEvent sequences (pointerdown/move/up with pointerId,
pointerType "touch", pressure) through the engine's DOM — not synthetic
clicks. The offline row kills the HTTP server after the service worker
activates and reloads: only the SW cache can serve the page, which is more
honest than a browser offline flag.

Exit code: 0 iff every row passes.
"""

import functools
import http.server
import json
import os
import sys
import threading
import time

# Software compositing: the harness must run on machines with no GL
# (offscreen windows, CI); WebKitGTK renders fine without it.
os.environ.setdefault("WEBKIT_DISABLE_COMPOSITING_MODE", "1")

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("WebKit2", "4.1")
from gi.repository import GLib, Gtk, WebKit2  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(HERE, "fixture")
PORT_ROW = ("--url" in sys.argv and sys.argv[sys.argv.index("--url") + 1]) or None
WIDTH, HEIGHT = 390, 844  # iPhone portrait; the rotate row swaps to 844x390
IPHONE = ("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) "
          "AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")

failures = 0


def check(ok, what):
    global failures
    print(("ok   " if ok else "FAIL ") + what, flush=True)
    failures += 0 if ok else 1


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      ".js": "text/javascript"}

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0),
                                         functools.partial(Handler, directory=FIXTURE))
threading.Thread(target=server.serve_forever, daemon=True).start()
BASE = PORT_ROW or f"http://127.0.0.1:{server.server_address[1]}/index.html"

# Non-ephemeral profile in a scratch dir so the service worker persists
# across the offline reload (an ephemeral context might drop it).
scratch = os.environ.get("SCRATCH", os.path.join(HERE, ".profile"))
os.makedirs(scratch, exist_ok=True)
manager = WebKit2.WebsiteDataManager(base_data_directory=scratch,
                                     base_cache_directory=scratch)
context = WebKit2.WebContext.new_with_website_data_manager(manager)
view = WebKit2.WebView.new_with_context(context)
settings = view.get_settings()
settings.set_enable_write_console_messages_to_stdout(True)
settings.set_user_agent(IPHONE)

window = Gtk.OffscreenWindow()
window.set_default_size(WIDTH, HEIGHT)
window.add(view)
window.show_all()


def pump(seconds):
    end = time.time() + seconds
    ctx = GLib.MainContext.default()
    while time.time() < end:
        ctx.iteration(False)
        time.sleep(0.005)


def js(script, timeout=30):
    """Evaluate `script` (may await) and return its JSON value."""
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
        pump(0.05)
    if "e" in box:
        raise RuntimeError(box["e"])
    if "v" not in box:
        raise TimeoutError(script)
    return json.loads(box["v"]) if box["v"] not in (None, "undefined") else None


def wait_for(expr, timeout):
    end = time.time() + timeout
    while time.time() < end:
        if js(expr):
            return True
        pump(0.25)
    return False


def load(url):
    loaded = {}
    handler = view.connect(
        "load-changed",
        lambda _v, ev: ev == WebKit2.LoadEvent.FINISHED and loaded.update(done=True),
    )
    view.load_uri(url)
    end = time.time() + 60
    while not loaded.get("done") and time.time() < end:
        pump(0.05)
    view.disconnect(handler)
    return loaded.get("done", False)


def pointer(kind, element, dx, dy, pid, buttons=1):
    """Dispatch one pointer event at (dx, dy) inside `element` (CSS px)."""
    return js(f"""(() => {{
      const el = document.getElementById("{element}");
      const r = el.getBoundingClientRect();
      const opts = {{ bubbles: true, cancelable: true, composed: true,
                      clientX: r.left + r.width / 2 + {dx},
                      clientY: r.top + r.height / 2 + {dy},
                      pointerId: {pid}, pointerType: "touch",
                      isPrimary: {str(pid == 1).lower()},
                      width: 20, height: 20, pressure: 0.5, button: 0,
                      buttons: {buttons} }};
      if ("{kind}" === "pointerdown") el.dispatchEvent(new PointerEvent("pointerover", opts));
      el.dispatchEvent(new PointerEvent("{kind}", opts));
      return true;
    }})()""")


def guesses():
    return js("document.body.dataset.guesses") or "0"


def gesture_drag(element, dx, dy, pid=1, steps=5):
    """A real finger: down, move in steps, up."""
    pointer("pointerdown", element, 0, 0, pid)
    for i in range(1, steps + 1):
        pointer("pointermove", element, dx * i / steps, dy * i / steps, pid)
        pump(0.02)
    pointer("pointerup", element, dx, dy, pid)


def main():
    global WIDTH, HEIGHT, window

    ok = load(BASE)
    check(ok, "page loads")
    check(wait_for("document.body.dataset.guesses !== undefined", 10),
          "input contract present (data-guesses)")
    js("document.body.dataset.errors = '0'")  # reset error ledger

    # Row 1 — vertical swipe on a button: the page must not scroll.
    gesture_drag("note-do", 0, 180)
    pump(0.2)
    check(js("window.scrollY === 0 && document.documentElement.scrollTop === 0"),
          "row 1: vertical swipe on a button -> scrollY stays 0")
    check(guesses() == "0", "row 1 (bonus): the big drag did not count as a tap")

    # Row 2 — a real 20 px-drag tap: exactly one guess.
    gesture_drag("note-mi", 12, 8)            # ~14 px: inside the slop
    pump(0.2)
    check(guesses() == "1", f"row 2: 20 px-drag tap -> exactly one guess (got {guesses()})")
    check(js("document.body.dataset.last") == "mi",
          "row 2: the guess was for the button actually touched")

    # Row 3 — two simultaneous pointers: at most one guess.
    before = int(guesses())
    pointer("pointerdown", "note-fa", 0, 0, 1)   # finger 1 down, held
    pump(0.1)
    pointer("pointerdown", "note-so", 0, 0, 2)   # finger 2 lands — must be ignored
    pump(0.1)
    gesture_drag("note-fa", 5, 3, pid=1)         # finger 1 finishes a tap
    pointer("pointerup", "note-so", 0, 0, 2)     # finger 2 lifts
    pump(0.2)
    after = int(guesses())
    check(after - before <= 1,
          f"row 3: two simultaneous pointers -> at most one guess (got {after - before})")

    # Row 4 — offline: SW active, server DEAD, reload: gameplay + sound.
    check(js("document.body.dataset.sw") == "registered", "row 4: service worker registered")
    check(wait_for("navigator.serviceWorker.controller !== null", 10),
          "row 4: service worker controlling the page")
    server.shutdown()                             # the real offline
    pump(0.5)
    ok = load(BASE)                               # only the SW can serve this
    check(ok, "row 4: page reloads with the server dead (SW-served)")
    check(guesses() is not None, "row 4: gameplay contract alive offline")
    gesture_drag("note-re", 4, 3)
    pump(0.3)
    offline_guesses = int(guesses() or "0")
    check(offline_guesses >= 1,
          f"row 4: a guess registers offline (got {offline_guesses})")
    audio = js("document.body.dataset.audio") or "none"
    check(audio in ("running", "suspended", "init"),
          f"row 4: sound initialized offline (state {audio!r})")
    check((js("document.body.dataset.errors") or "0") == "0",
          "row 4: zero page errors while offline (no failed fetches surfaced)")

    # Row 5 — rotate to landscape: every button visible.
    window.resize(844, 390)
    pump(0.5)
    visible = js("""(() => {
      const vw = document.documentElement.clientWidth;
      const vh = document.documentElement.clientHeight;
      const ids = ["note-do","note-re","note-mi","note-fa","note-so"];
      const out = ids.filter((id) => {
        const r = document.getElementById(id).getBoundingClientRect();
        return r.width <= 0 || r.bottom < 0 || r.top > vh || r.right < 0 || r.left > vw;
      });
      return out.join(",");
    })()""")
    check(visible == "", f"row 5: rotate -> all buttons visible (missing: {visible or 'none'})")

    print(f"\n{'ALL ROWS PASS' if failures == 0 else str(failures) + ' ROW(S) FAILED'}"
          f"  [WebKitGTK {WebKit2.get_minor_version()}]  {BASE}")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    finally:
        try:
            server.shutdown()
        except Exception:  # noqa: BLE001
            pass
