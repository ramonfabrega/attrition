#!/bin/zsh
# prefix.sh — make `~/wine-ron` a prefix the capture lane can run in: a D3D11
# that Apple's GPU can serve, a `C:\users\crossover` that resolves, and a
# debugger that writes to the log rather than the screen.
#
# RoN:EE's renderer DLL is `d3dgl.dll`, and the name is a lie: it imports
# **d3d11.dll** and opens with
#
#   D3D11CreateDevice(NULL, HARDWARE, NULL, 0, {0xa000}, 1, 7, ...)
#
# — one feature level, `0xa000` = D3D_FEATURE_LEVEL_10_0 — and puts up
# "Could not initialize DirectX! ... DirectX 10 or higher" on any negative
# HRESULT. Two free translators can answer that call and neither stock one
# does:
#
#   - **wined3d** asks winemac.drv for a 3.2+ GL context, which upstream Wine
#     refuses on macOS ("OS X only supports forward-compatible 3.2+
#     contexts"), so it offers no feature level at all. `MaxVersionGL` only
#     changes which version it is refused for.
#   - **stock DXVK** (2.7) skips the M4 Max: "Device does not support
#     required feature 'geometryShader'" — Apple's GPUs have none.
#
#   - **DXVK-macOS** (Gcenx's fork of DXVK 1.10.3, the build Whisky used)
#     drops that requirement and reports
#     "D3D11CoreCreateDevice: Using feature level D3D_FEATURE_LEVEL_10_0".
#     That is exactly the level d3dgl asks for.
#
# The DLLs live in the prefix, never in this repo. Re-run after rebuilding
# the prefix; it is idempotent.
set -e

V=v1.10.3-20230507
F=dxvk-macOS-async-$V.tar.gz
P=${RON_WINEPREFIX:-$HOME/wine-ron}
T=${TMPDIR:-/tmp}/ron-dxvk
mkdir -p "$T"

if [ ! -d "$T/dxvk-macOS-async-$V" ]; then
  echo "fetching $F"
  curl -fsSL -o "$T/$F" \
    "https://github.com/Gcenx/DXVK-macOS/releases/download/$V/$F"
  tar xzf "$T/$F" -C "$T"
fi

# The game is PE32, so the DLLs that matter are the x32 ones, and under wow64
# those go in syswow64.
for d in d3d11 dxgi d3d10core; do
  src="$T/dxvk-macOS-async-$V/x32/$d.dll"
  dst="$P/drive_c/windows/syswow64/$d.dll"
  [ -f "$src" ] || { echo "missing $src" >&2; exit 1; }
  cp "$src" "$dst"
  echo "installed $d.dll -> $dst"
done

# A crash should land in the run's log, not in a dialog nobody is watching.
W=${RON_WINE_BIN:-/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine}
WINEPREFIX="$P" WINEDEBUG=-all "$W" reg add 'HKCU\Software\Wine\WineDbg' \
  /v ShowCrashDialog /t REG_DWORD /d 0 /f >/dev/null 2>&1
echo "ShowCrashDialog=0"

# **`C:\users\crossover` has to exist.** Every ini in the install carries
# absolute Windows paths written under CrossOver, whose prefix user was
# `crossover` — `gamelog.ini`'s `LogFile=` most of all. Free Wine's user is
# the macOS one, so those paths point at nothing, and the game **does not
# complain**: run903 played its 400 frames, quit cleanly, and wrote no
# `gamelog.txt` at all. One symlink makes every stored path valid and costs
# nothing; rewriting the inis would have to be redone each time the game
# rewrites them itself.
link="$P/drive_c/users/crossover"
if [ ! -e "$link" ]; then
  ln -s rf-studio "$link"
  echo "linked C:\\users\\crossover -> rf-studio"
else
  echo "C:\\users\\crossover already there"
fi
