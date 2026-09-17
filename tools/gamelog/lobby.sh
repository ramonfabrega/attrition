#!/bin/zsh
# lobby.sh — where the lobby's buttons are, on whichever screen is on.
#
# Source it, call `lobby_init`, then click by **name**:
#
#     source "$W/tools/gamelog/lobby.sh"
#     lobby_init || exit 1
#     lobby_click solo 4
#     lobby_click quick 8
#     lobby_click start 20
#
# **Why this exists.** This machine has two desktops — 3440x1440 with the
# main monitor on, 1920x1080 without — and the game lays its window out
# differently on each: at (760, 152) on the wide one and full-screen at
# (0, 0) on the small one. Every drive script used to carry the wide
# desktop's numbers as constants, so a run started with the monitor off
# clicked on nothing and sat on the Main Menu until it was killed (run32,
# 2026-08-28, ten minutes).
#
# `lobby_init` takes a screenshot and reads its pixel width, which is the
# only measurement that is always available and always right. An unknown
# width is an error rather than a guess: measure the buttons off a
# screenshot (`sips -Z 900` it, read the image coordinates, multiply by
# `width / 900`) and add a row here.
#
# **The Map Style combo is only measured on the wide desktop, and it does
# not need to be measured on the other**: the lobby reads the style from
# `PlayerProfile/Player.dat`'s `<MULTI>` block and `check.ini`'s
# `mapstyles=`, both of which can be edited with the game closed
# (`roadcapture.sh` does). That is also faster and cannot mis-click.

# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
typeset -gA LOBBY
typeset -g LOBBY_W LOBBY_H LOBBY_REGION LOBBY_STARTS

lobby_init() {
  local shot=${1:-${TMPDIR:-/tmp}/ron-lobby-probe.png}
  screencapture -x "$shot" || { echo "lobby: no screenshot (screen recording?)"; return 1 }
  LOBBY_W=$(sips -g pixelWidth "$shot" 2>/dev/null | awk '/pixelWidth/{print $2}')
  LOBBY_H=$(sips -g pixelHeight "$shot" 2>/dev/null | awk '/pixelHeight/{print $2}')
  case "$LOBBY_W" in
    3440)
      # The main monitor. The window sits at (760, 152), 1920x1108.
      LOBBY=(
        solo   "1715 744"
        quick  "1715 672"
        combo  "2392 291"
        indies "2262 551"
        start  "1061 1176"
      )
      LOBBY_REGION="-R760,152,1920,1108"
      # Start needs two presses when the window is inset (run22).
      LOBBY_STARTS=2
      ;;
    1920)
      # The laptop alone: the game is full-screen at (0, 0), and one press
      # of Start is enough (run32).
      LOBBY=(
        solo  "960 565"
        quick "960 495"
        start "292 994"
      )
      LOBBY_REGION=""
      LOBBY_STARTS=1
      ;;
    *)
      echo "lobby: unknown desktop ${LOBBY_W}x${LOBBY_H} — measure the buttons and add a row to tools/gamelog/lobby.sh"
      return 1
      ;;
  esac
  echo "lobby: ${LOBBY_W}x${LOBBY_H}, start needs $LOBBY_STARTS press(es)"
}

# lobby_shot <file> — the game's own region on this desktop.
lobby_shot() {
  # `LOBBY_REGION` is empty on a full-screen desktop, where the whole shot
  # *is* the game; `-R` with an empty argument is not.
  if [ -n "$LOBBY_REGION" ]; then
    screencapture -x ${=LOBBY_REGION} "$1"
  else
    screencapture -x "$1"
  fi
  sips -Z 900 "$1" --out "${1:r}s.png" >/dev/null 2>&1
}

# lobby_click <name> [settle seconds] [shot file]
lobby_click() {
  local name=$1 settle=${2:-3} shot=$3
  local where=${LOBBY[$name]}
  if [ -z "$where" ]; then
    echo "lobby: no '$name' on a ${LOBBY_W}-wide desktop (see the note in lobby.sh)"
    return 1
  fi
  local -a xy
  xy=(${=where})
  zsh "$RON_TOOLS/focus.sh" >/dev/null || return 1
  sleep 0.5
  cliclick m:${xy[1]},${xy[2]} w:400 c:${xy[1]},${xy[2]}
  sleep $settle
  [ -n "$shot" ] && lobby_shot "$shot"
  echo "$(date +%H:%M:%S) clicked $name at ${xy[1]},${xy[2]}"
}

# lobby_start [settle] [shot prefix] — Start, pressed as often as this
# desktop needs.
lobby_start() {
  local settle=${1:-20} prefix=$2 i
  for i in $(seq 1 ${LOBBY_STARTS:-1}); do
    if [ "$i" -lt "${LOBBY_STARTS:-1}" ]; then
      lobby_click start 3 ${prefix:+${prefix}start$i.png} || return 1
    else
      lobby_click start $settle ${prefix:+${prefix}start$i.png} || return 1
    fi
  done
}
