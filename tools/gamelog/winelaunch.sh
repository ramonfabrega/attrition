#!/bin/zsh
# winelaunch.sh — the one line that starts the original, sourced by every
# capture script. It was eight copies of CrossOver's `wine --bottle ron`
# until 2026-09-04, when the bottle's licence expired and the lane moved to
# free WineHQ Stable 11.0. `docs/ORACLE.md`, "Off CrossOver", carries why
# each piece below is there; the short version:
#
#   - **The prefix is `~/wine-ron`**, whose `AppData\Roaming\Microsoft Games`
#     is a symlink at the old bottle's, so `rise.ini`, `gamelog.ini` and
#     `Logs/` are the same files every other tool already knows.
#   - **`d3d11`, `dxgi` and `d3d10core` are native** — DXVK-macOS, installed
#     by `tools/gamelog/dxvk.sh`. RoN:EE's renderer is `d3dgl.dll`, which
#     despite the name imports **d3d11** and asks for feature level 10_0;
#     Wine's own wined3d cannot serve that on macOS (winemac.drv refuses a
#     3.2+ GL context), and stock DXVK refuses Apple's GPU for want of
#     `geometryShader`. The macOS fork is the build that does neither.
#   - **`winedbg.exe` is left enabled.** A crash then prints its fault to the
#     run's log instead of parking a dialog on the screen — and with
#     `ShowCrashDialog=0` in the prefix it does not even do that.
#   - **MoltenVK's banner goes to stderr** and is a hundred lines a run, so
#     every reader of these logs filters `^\[mvk` or `^info:`.
#
# Usage, from a script that has already `cd`-ed to the install:
#
#   source "$W/tools/gamelog/winelaunch.sh"
#   ron_wine "$T/wine$N.log" "$G/$P" ${=CFG} -automation
#   echo "launched pid $RON_WINE_PID"

RON_WINE_BIN=${RON_WINE_BIN:-/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine}
RON_WINEPREFIX=${RON_WINEPREFIX:-$HOME/wine-ron}

ron_wine () {
  local log=$1; shift
  export WINEPREFIX="$RON_WINEPREFIX"
  export WINEDLLOVERRIDES="mscoree,mshtml=d;d3d11,dxgi,d3d10core=n"
  export WINEDEBUG=${WINEDEBUG:--all}
  export DXVK_LOG_LEVEL=${DXVK_LOG_LEVEL:-info}
  export DXVK_LOG_PATH=${DXVK_LOG_PATH:-none}
  nohup "$RON_WINE_BIN" "$@" > "$log" 2>&1 &
  RON_WINE_PID=$!
}
