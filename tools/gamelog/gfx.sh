#!/bin/zsh
# If the clean run still shows no window, the next lever is the renderer.
# `rise.ini` carries `GraphicsDLL=d3dgl.dll` — RoN:EE picks its renderer by
# name and ships its own D3D-to-OpenGL wrapper, which is what CrossOver was
# feeding into D3DMetal. Try the game with that key cleared, so it takes
# whatever its default path is and Wine's own wined3d carries it.
#
# The ini is restored at the end whatever happens: every capture on disk
# depends on it, and `window.py restore` does not know about this key.
W="/Applications/Wine Stable.app/Contents/Resources/wine/bin"
export WINEPREFIX=/Users/rf-studio/wine-ron
export WINEDLLOVERRIDES="mscoree,mshtml=d;winedbg.exe=d"
export WINEDEBUG=-all
G=/Users/rf-studio/code/fun/attrition/game
INI="/Users/rf-studio/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/rise.ini"
BAK=/tmp/ron-runs/rise.ini.gfxbak

cp "$INI" "$BAK" || exit 1
restore () { cp "$BAK" "$INI"; echo "rise.ini restored"; }
trap restore EXIT INT TERM

for variant in "" "wined3d"; do
  if [ -z "$variant" ]; then
    sed 's/^GraphicsDLL=.*/GraphicsDLL=/' "$BAK" > "$INI"
    label=empty
  else
    cp "$BAK" "$INI"
    label=d3dgl
  fi
  echo "=== GraphicsDLL variant: $label"
  grep '^GraphicsDLL' "$INI"
  pkill -f 'riseofnations' 2>/dev/null; pkill -f 'winedbg' 2>/dev/null; sleep 2
  cd "$G"
  "$W/wine" "$G/riseofnations.exe" -config check.ini -automation \
    > "/tmp/ron-runs/gfx-$label.log" 2>&1 &
  found=""
  for i in 1 2 3 4 5 6 7 8; do
    sleep 5
    w=$(osascript -e 'tell application "System Events" to get name of every window of process "riseofnations.exe"' 2>&1)
    case "$w" in *error*|"[]"|"") ;; *) echo "$label t$i -> [$w]"; found=1; break;; esac
  done
  screencapture -x "/tmp/ron-runs/gfx-$label.png" 2>/dev/null
  sips -Z 1200 "/tmp/ron-runs/gfx-$label.png" --out "/tmp/ron-runs/gfx-$label-s.png" >/dev/null 2>&1
  echo "$label verdict: ${found:+WINDOW}${found:-NO WINDOW}"
  grep -vE '^\s|^\[mvk' "/tmp/ron-runs/gfx-$label.log" | tail -4
done
pkill -f 'riseofnations' 2>/dev/null
echo "gfx done"
