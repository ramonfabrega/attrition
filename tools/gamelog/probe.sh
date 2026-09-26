#!/bin/zsh
# probe.sh — `perm_probe`, the macOS permission check the capture scripts run
# before they stage anything. Sourced, not executed.
#
# A capture needs three grants and each fails differently:
#
#   Screen Recording  `screencapture` writes nothing, "could not create image"
#   Automation        `osascript` hangs at 0 % CPU, AppleEvent timed out (-1712)
#   Accessibility     synthetic clicks vanish; UI scripting says -1728
#
# **Two probes have now failed to catch a missing Accessibility grant, and
# both failed the same way: they asked a question whose answer looks the same
# when granted and when refused.** Do not add a third of that shape.
#
#   `cliclick p` reads the cursor and takes no privilege at all, so it
#   answered a real `1649,0` on 2026-09-03 while every synthetic event was
#   being dropped. `cliclick` is the trap underneath it: denied, it still
#   exits 0 and still prints a plausible position, warning only on stderr.
#
#   Its replacement asked System Events for `every window of process
#   "Finder"`, on the reasoning that Finder always exists. Finder is usually
#   running with **no windows open**, and an empty list needs no accessibility
#   call to produce — so it returns the empty string and rc=0 whether or not
#   the grant is there. It printed `probe ok` on the run it was written to
#   catch. The same call against the game, which does have a window, refuses
#   with -1728; the probe simply never asked anything that had to be answered.
#
# So the Accessibility check here is **positive**: post a synthetic move and
# read back where the cursor actually went. Nothing about that can be faked by
# a process that is not allowed to post events. Twice, so a cursor already
# resting on the target cannot pass by luck, and the entry position is put
# back afterwards.
#
# `docs/ORACLE.md`, "The fourth permission", carries the history, and the
# section under it explains why re-ticking `ClaudeCode.app` never helps: the
# grant is keyed to `~/.local/share/claude/versions/<VERSION>`, a path that
# changes with every update. `tools/gamelog/viadriver.sh` is the way out.

# perm_probe [label] — echoes "probe ok (…)" and returns 0, or explains which
# grant is missing and returns 1.
perm_probe() {
  local shot=${1:-/tmp/ron-runs/permprobe.png}
  mkdir -p "${shot:h}"

  # --- Screen Recording. A denied screencapture writes no file at all.
  rm -f "$shot"
  screencapture -x "$shot" 2>/dev/null || true
  if [ ! -s "$shot" ]; then
    echo "Screen Recording is off — screencapture wrote nothing."
    echo "System Settings -> Privacy & Security -> Screen Recording."
    _perm_who
    return 1
  fi

  # --- Automation. A denied osascript hangs rather than answering, so cap it.
  local auto
  auto=$(osascript -e 'with timeout of 15 seconds
tell application "System Events" to get name of first process
end timeout' 2>&1) || true
  case "$auto" in
    ""|*"timed out"*|*-1712*|*"Not authorized"*|*-1743*)
      echo "Automation is off or timed out: ${auto:-<no answer>}"
      echo "System Settings -> Privacy & Security -> Automation -> System Events."
      _perm_who
      return 1 ;;
  esac

  # --- Accessibility, positively. Post two moves; read back both.
  local entry a b
  entry=$(cliclick p 2>/dev/null | tail -1)
  cliclick m:37,41 >/dev/null 2>&1
  a=$(cliclick p 2>/dev/null | tail -1)
  cliclick m:53,59 >/dev/null 2>&1
  b=$(cliclick p 2>/dev/null | tail -1)
  case "$entry" in
    *,*) cliclick "m:$entry" >/dev/null 2>&1 ;;
  esac
  if [ "$a" != "37,41" ] || [ "$b" != "53,59" ]; then
    echo "Accessibility is off — a synthetic move did not land."
    echo "  asked for 37,41 and 53,59; the cursor read back $a and $b."
    echo "System Settings -> Privacy & Security -> Accessibility."
    _perm_who
    return 1
  fi

  echo "probe ok (cursor $entry, synthetic move landed)"
  return 0
}

# Who to grant it to — the part that is easy to get wrong, and did cost an
# afternoon. The bundle in the Accessibility list is NOT what macOS judges.
_perm_who() {
  local app=${RONDRIVER_APP:-$HOME/bin/RonDriver.app}
  echo
  echo "  Grant it to the process macOS actually blames, which is NOT"
  echo "  ClaudeCode.app — adding that bundle does nothing, because macOS"
  echo "  never evaluates its path. Check who it blames with:"
  echo
  echo "    /usr/bin/log show --last 3m --predicate 'subsystem == \"com.apple.TCC\"' \\"
  echo "      --style compact | grep AUTHREQ_SUBJECT"
  echo
  if [ -x "$app/Contents/MacOS/RonDriver" ]; then
    echo "  This machine has $app for exactly this."
    echo "  Grant that once and it survives every Claude Code update:"
    echo "    zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - <item>"
  else
    echo "  Build the fixed-path launcher and grant that instead, once:"
    echo "    zsh tools/gamelog/rondriver/build.sh"
  fi
}
