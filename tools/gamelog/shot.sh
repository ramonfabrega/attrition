#!/bin/zsh
# shot.sh [out.png] [width]         -- whole desktop, plus a downscaled copy
# shot.sh -r X Y W H out.png [width] -- one region of the desktop
#
# Coordinates everywhere are desktop points, which is also what cliclick takes.
# The game window is 1920x1080 inside whatever the desktop is and moves between
# launches, so locate it from a shot rather than trusting stored coordinates.
if [ "$1" = "-r" ]; then
  shift
  x=$1 y=$2 w=$3 h=$4; shift 4
  out=${1:-${TMPDIR:-/tmp}/ron.png}
  screencapture -x -R$x,$y,$w,$h "$out"
else
  out=${1:-${TMPDIR:-/tmp}/ron.png}
  screencapture -x "$out"
fi
sips -Z ${2:-1400} "$out" --out "${out%.png}s.png" >/dev/null
echo "${out%.png}s.png"
