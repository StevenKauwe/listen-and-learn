#!/usr/bin/env bash
# Build the suite into dist/ (static site) and dist/artifact/ (claude.ai page).
# Needs: rust target wasm32-unknown-unknown, wasm-bindgen-cli 0.2.129.
# Optional: wasm-opt on PATH (or WASM_OPT=path) shrinks the wasm further.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown -p listen-web
rm -rf dist && mkdir -p dist/artifact
wasm-bindgen --target web --no-typescript --out-dir dist \
  target/wasm32-unknown-unknown/release/listen_web.wasm
OPT="${WASM_OPT:-$(command -v wasm-opt || true)}"
if [ -n "$OPT" ]; then
  "$OPT" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals -o dist/listen_web_bg.wasm dist/listen_web_bg.wasm
fi
cp web/manifest.webmanifest web/sw.js dist/
cp -r web/icons dist/
# Static site: a real document head around the shared page body.
{
  cat <<'HEAD'
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="theme-color" content="#D6E7E2">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="default">
<meta name="description" content="Ear-training games for small children: high and low, same and different, up and down, Do Re Mi.">
<link rel="manifest" href="manifest.webmanifest">
<link rel="icon" href="icons/icon-192.png">
<link rel="apple-touch-icon" href="icons/icon-180.png">
HEAD
  sed 's|</script>|if ("serviceWorker" in navigator) navigator.serviceWorker.register("./sw.js").catch(() => {});\n</script>|' web/app.html
  printf '</html>\n'
} > dist/index.html
# claude.ai artifact: the host supplies the document skeleton.
cp web/app.html dist/artifact/index.html
cp dist/listen_web.js dist/listen_web_bg.wasm dist/artifact/
ls -l dist/listen_web_bg.wasm dist/listen_web.js
