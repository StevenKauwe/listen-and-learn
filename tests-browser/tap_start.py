"""Tap-start regression smoke test: build first, then run with Playwright installed.
Set CHROME to a local Chrome binary, or install Playwright Chromium.
"""
import os,threading,http.server,functools
from playwright.sync_api import sync_playwright
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(http.server.SimpleHTTPRequestHandler,directory='dist'))
threading.Thread(target=server.serve_forever,daemon=True).start()
base=f'http://127.0.0.1:{server.server_port}/'
with sync_playwright() as p:
 b=p.chromium.launch(executable_path=os.environ.get('CHROME'),headless=True,args=['--autoplay-policy=no-user-gesture-required'])
 page=b.new_page(viewport={'width':390,'height':844})
 for slug in ['notes','birdfish','twins','stairs','echo','tempo','loud','who','drum','play']:
  print('BEGIN',slug,flush=True);page.goto(base+'#'+slug,wait_until='domcontentloaded');page.wait_for_selector('#stage [data-t]')
  assert page.locator('#listen,.listen-wrap').count()==0
  for w,h in [(390,844),(844,390)]:
   page.set_viewport_size({'width':w,'height':h})
   assert page.evaluate("document.documentElement.scrollHeight <= innerHeight+1")
   assert page.locator('#stage').bounding_box()['height']>200
  page.set_viewport_size({'width':390,'height':844})
  field=page.locator('#stage [data-t]').last
  field.click(force=True)
  phase='free' if slug=='play' else 'drumming' if slug=='drum' else 'awaiting'
  page.wait_for_function('(v)=>document.body.dataset.phase===v',arg=phase,timeout=15000)
  if slug not in ['play','drum']:
   assert page.get_attribute('body','data-guesses')=='0'
   page.locator('.status').click(force=True)
   page.wait_for_function("document.body.dataset.phase==='question'")
   page.wait_for_function("document.body.dataset.phase==='awaiting'",timeout=15000)
   # Restart after an interruption using keyboard on an answer field.
   page.evaluate("window.dispatchEvent(new Event('pagehide'));window.dispatchEvent(new Event('pageshow'))")
   page.wait_for_function("document.body.dataset.phase==='ready'")
   field.focus();page.keyboard.press('Enter')
   page.wait_for_function("document.body.dataset.phase==='awaiting'",timeout=15000)
  print('PASS',slug,flush=True)
  if slug=='who':page.screenshot(path=os.environ.get('SCREENSHOT', '/tmp/listen-tap-start.png'))
 b.close()
