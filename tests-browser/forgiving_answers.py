"""Behavioral checks for forgiving, target-consistent answer gestures.
Build first; run with Playwright installed. CHROME may name a local binary.
"""
import functools
import http.server
import os
import threading
from playwright.sync_api import sync_playwright

server = http.server.ThreadingHTTPServer(
    ("127.0.0.1", 0),
    functools.partial(http.server.SimpleHTTPRequestHandler, directory="dist"),
)
threading.Thread(target=server.serve_forever, daemon=True).start()
base = f"http://127.0.0.1:{server.server_port}/"

with sync_playwright() as p:
    browser = p.chromium.launch(
        executable_path=os.environ.get("CHROME"), headless=True,
        args=["--autoplay-policy=no-user-gesture-required"],
    )
    page = browser.new_page(viewport={"width": 390, "height": 844})
    page.add_init_script("localStorage.setItem('listen-learn-settings', 'v=2\\nage=5\\n');")

    def ready(slug="who"):
        page.goto(base + "#" + slug, wait_until="domcontentloaded")
        page.reload(wait_until="domcontentloaded")
        page.wait_for_selector("#stage [data-t]")
        page.locator("#stage [data-t]").first.click(force=True)
        page.wait_for_function("document.body.dataset.phase === 'awaiting'", timeout=15000)
        ans = int(page.get_attribute("body", "data-answer").split(",")[0])
        choices = page.locator("#stage [data-t]").evaluate_all("(xs)=>xs.map(x=>+x.dataset.t)")
        return ans, next(t for t in choices if t != ans)

    def ptr(kind, target, pid=1, fraction=0.5):
        page.evaluate("""([kind,target,pid,f]) => {
          const el=document.querySelector('#stage [data-t="'+target+'"]');
          const r=el.getBoundingClientRect();
          el.dispatchEvent(new PointerEvent(kind, {
            bubbles:true, pointerId:pid, pointerType:'touch', isPrimary:pid===1,
            clientX:r.left+r.width*f, clientY:r.top+r.height*.5,
            buttons:kind==='pointerup'?0:1
          }));
        }""", [kind, target, pid, fraction])

    def tap(target, pid=1):
        ptr("pointerdown", target, pid)
        ptr("pointerup", target, pid)

    def counts(right=0, wrong=0):
        page.wait_for_timeout(260)
        assert int(page.get_attribute("body", "data-right")) == right
        assert int(page.get_attribute("body", "data-wrong")) == wrong

    for slug in ["notes", "birdfish", "twins", "stairs", "tempo", "loud", "who"]:
        ans, other = ready(slug)
        tap(ans)
        page.wait_for_timeout(35)
        tap(ans, 2)
        counts(right=1)
        print("PASS same-field double tap:", slug, flush=True)

    ans, other = ready()
    ptr("pointerdown", ans)
    ptr("pointerdown", ans, 2)
    ptr("pointerup", ans, 2)
    ptr("pointerup", ans)
    counts(right=1)
    print("PASS same-field simultaneous fingers", flush=True)

    ans, other = ready()
    ptr("pointerdown", ans, fraction=.1)
    ptr("pointermove", ans, fraction=.9)  # > 70px, still inside the same field
    ptr("pointerup", ans, fraction=.9)
    counts(right=1)
    print("PASS long within-field slide", flush=True)

    ans, other = ready()
    tap(ans)
    page.wait_for_timeout(35)
    tap(other, 2)
    page.wait_for_timeout(35)
    tap(ans, 3)
    counts()
    tap(ans)
    counts(right=1)
    print("PASS mixed rapid burst rejected, fresh answer recovers", flush=True)

    ans, other = ready()
    ptr("pointerdown", ans)
    ptr("pointerdown", other, 2)
    ptr("pointerup", ans)
    ptr("pointerup", other, 2)
    counts()
    print("PASS mixed simultaneous fingers", flush=True)

    ans, other = ready()
    ptr("pointerdown", ans)
    ptr("pointermove", other)
    ptr("pointermove", ans)
    ptr("pointerup", ans)
    counts()
    print("PASS cross-answer slide cannot return to score", flush=True)

    ans, other = ready()
    tap(ans)
    page.locator(".status").click(force=True)
    page.wait_for_function("document.body.dataset.phase==='question'")
    page.wait_for_function("document.body.dataset.phase==='awaiting'", timeout=15000)
    counts()
    print("PASS replay clears pending answer", flush=True)

    ans, other = ready()
    tap(ans)
    page.evaluate("window.dispatchEvent(new Event('pagehide'));window.dispatchEvent(new Event('pageshow'))")
    counts()
    assert page.get_attribute("body", "data-phase") == "ready"
    print("PASS interruption clears pending answer", flush=True)

    ans, other = ready()
    field = page.locator(f'#stage [data-t="{ans}"]')
    field.focus()
    page.keyboard.press("Enter")
    page.keyboard.press("Enter")
    counts(right=1)
    print("PASS keyboard same-field repeat", flush=True)

    ans, other = ready()
    field = page.locator(f'#stage [data-t="{ans}"]')
    field.focus()
    page.keyboard.press("Enter")
    page.locator(f'#stage [data-t="{other}"]').focus()
    page.keyboard.press("Enter")
    counts()
    print("PASS mixed keyboard burst", flush=True)

    ans, other = ready()
    tap(ans)
    ptr("pointerdown", ans)
    page.evaluate("""document.body.dispatchEvent(new PointerEvent('pointermove',
      {bubbles:true,pointerId:1,pointerType:'touch',buttons:1,clientX:1,clientY:1}))""")
    ptr("pointermove", other)
    ptr("pointerup", other)
    counts()
    print("PASS swipe across gap invalidates preceding answer", flush=True)

    ans, other = ready()
    ptr("pointerdown", ans)
    page.wait_for_timeout(450)
    ptr("pointerup", ans)
    counts(right=1)
    page.wait_for_timeout(1800)
    assert page.get_attribute("body", "data-right") == "1"
    print("PASS held answer counts once, never advances multiple rounds", flush=True)

    ans, other = ready()
    tap(other)
    counts(wrong=1)
    print("PASS a deliberate wrong answer still counts normally", flush=True)

    # Echo deliberately accepts repeated notes only as distinct gestures.
    ans, other = ready("echo")
    answer = [int(t) for t in page.get_attribute("body", "data-answer").split(",")]
    for target in answer:
        tap(target)
        page.wait_for_timeout(260)
    assert page.get_attribute("body", "data-right") == "1"
    print("PASS intentional Echo sequence", flush=True)
    browser.close()
