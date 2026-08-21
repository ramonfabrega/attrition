#!/bin/zsh
# wait for the RoN window to exist, then focus it and shoot a downscaled screenshot
while true; do
  if pgrep -f 'riseofnations.exe' >/dev/null; then
    w=$(osascript -e 'tell application "System Events" to get name of every window of process "riseofnations.exe"' 2>/dev/null)
    if [ -n "$w" ]; then break; fi
  fi
  sleep 3
done
sleep 8
osascript -e 'tell application "System Events" to set frontmost of process "riseofnations.exe" to true' >/dev/null 2>&1
sleep 2
out=${1:-/Users/rf-studio/.claude/jobs/3c382923/tmp/d1.png}
small=${out%.png}s.png
screencapture -x "$out"
sips -Z 900 "$out" --out "$small" >/dev/null
echo ready
