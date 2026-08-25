#!/bin/zsh
# wait for the RoN window to exist, then focus it and shoot a downscaled screenshot
#
# The process is `riseofnations.exe` for the install's own executable and
# `riseofnations_trace.exe` for the traced copy (`tools/trace/`); match both,
# or a traced run waits forever (2026-08-25, run20).
proc=""
while true; do
  proc=$(pgrep -fl 'riseofnations(_trace)?\.exe' | grep -o 'riseofnations[_a-z]*\.exe' | head -1)
  if [ -n "$proc" ]; then
    w=$(osascript -e "tell application \"System Events\" to get name of every window of process \"$proc\"" 2>/dev/null)
    if [ -n "$w" ]; then break; fi
  fi
  sleep 3
done
sleep 8
osascript -e "tell application \"System Events\" to set frontmost of process \"$proc\" to true" >/dev/null 2>&1
sleep 2
out=${1:-/Users/rf-studio/.claude/jobs/3c382923/tmp/d1.png}
small=${out%.png}s.png
screencapture -x "$out"
sips -Z 900 "$out" --out "$small" >/dev/null
echo "ready ($proc)"
