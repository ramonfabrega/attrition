#!/bin/zsh
# wait for the RoN window to exist, then focus it and shoot a downscaled
# screenshot.
#
# The unix process is `riseofnations.exe` for the install's own executable and
# `riseofnations_trace.exe` for the traced copy (`tools/trace/`); match both,
# or a traced run waits forever (2026-08-25, run20). Under free WineHQ the
# unix side also carries a `start.exe /exec` parent, which matches the same
# pattern and is harmless here — we only use it to know a run is alive.
#
# **The window, though, is not found by that name.** System Events calls every
# free-Wine GUI process `wine`, so the window is matched by its title;
# `focus.sh` is the one place that knows how, and this waits on it.
proc=""
while true; do
  proc=$(pgrep -fl 'riseofnations(_trace)?\.exe' | grep -o 'riseofnations[_a-z]*\.exe' | head -1)
  if [ -n "$proc" ]; then
    w=$(zsh "${0:A:h}/focus.sh" --title 2>/dev/null)
    if [ -n "$w" ]; then break; fi
  fi
  sleep 3
done
sleep 8
zsh "${0:A:h}/focus.sh" >/dev/null 2>&1
sleep 2
out=${1:-/tmp/ron-runs/d1.png}
small=${out%.png}s.png
screencapture -x "$out"
sips -Z 900 "$out" --out "$small" >/dev/null
echo "ready ($proc, window: $w)"
