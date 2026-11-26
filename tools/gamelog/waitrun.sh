#!/bin/zsh
# waitrun.sh <viadriver-log> [poll-seconds] — block until a detached capture
# has finished, then print its verdict lines and exit with its verdict.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh …
#   # prints "log: /tmp/ron-runs/viadriver-<ts>.log"
#   zsh tools/gamelog/waitrun.sh /tmp/ron-runs/viadriver-<ts>.log
#
# Why this exists (parked 656, the thirteenth pass, 2026-09-23): `viadriver.sh`
# hands the capture to RonDriver.app and returns at once, so the session that
# launched it has nothing to wait on. Item 571 waited by grepping a log the
# verdict never reached — the runner's stdout goes to the viadriver log — so
# the loop spun for two hours after the capture finished. This script watches
# the file the verdict actually lands in, and it also notices a runner that
# died without printing one. The queue lane's banner (`runqueue.sh`'s `=== the
# queue, as it went ===`) was read here until item 3485 retired that lane.
#
# Run it under the harness's background lane (`run_in_background`) and end the
# turn: the harness re-invokes the session when this exits. The sleeping is
# this script's, not the session's.
#
# The click-free lane (`tools/explore/unattended_capture.py`, the golden
# chapters' runner) prints its verdict as one JSON receipt line per
# map, `{"map_requested": …, "success": true|false, …}`, on the same stdout.
# Four items of the fourteenth pass's tranche (651, 676, 696, 714) found this
# script exiting 2 at once on that lane and fell back on reading the receipt
# by hand, so the receipt is read here: once no runner is alive, a log holding
# receipts is judged by them.
#
# Exit status: 0 — the runner has exited and every receipt says
# `"success": true`, or a lone capture script said `=== captured: `; 1 — a
# receipt says `"success": false`, or the script said `=== capture
# failed: `; 2 — no runner is
# alive and no verdict came (the runner died, or was never
# started within the grace; read the log's tail this prints), or the log ends in a Python
# traceback no receipt follows — a runner that died (run659's first take,
# item 1503, parked 1513; the twenty-fifth pass): it is called at once,
# without waiting for every other lane's runner to go quiet, because
# `pgrep` over the runner pattern sees any lane's; 64 — usage.
set -u
log=${1:-}
poll=${2:-20}
if [ -z "$log" ]; then
  echo "usage: waitrun.sh <viadriver-log> [poll-seconds]" >&2
  exit 64
fi
receipt='^{"map_requested"'
# Every launcher in the tree — a script that calls `ron_wine`, and `lane.sh`
# — held to this list by `test_waitrun.py` (parked 1574: four launchers
# each missed it once, and the waiter read a live capture as dead).
runner_pattern=${WAITRUN_RUNNER:-'unattended_capture.py|gamelog/startcapture.sh|gamelog/lane.sh|gamelog/longtrace.sh|gamelog/live.sh|gamelog/runwin.sh|gamelog/roadcapture.sh|gamelog/censuswindow.sh|fuzz/run.sh|fuzz/gate.sh'}
# `lane.sh N` builds a pool lane before its admission run starts a runner
# (item 1569): run689's waiter read the build as a dead runner and exited 2.
# A capture script launched on its own, with no runner around it, ends on one
# of these (`startcapture.sh`; parked 1080, the nineteenth pass): run381
# finished and this script called it a dead runner.
captured='^=== captured: '
failed='^=== capture failed: '
traceback='Traceback (most recent call last):'

# `run_in_background` reports a task the moment it exits, so a launch that
# has not written its log yet, or whose runner has not started, is waited
# for too — but only for `WAITRUN_GRACE` seconds: a launch that shows no
# start by then never happened.
grace=${WAITRUN_GRACE:-120}
waited=0
header='^(args|lane|output):|^$'
while :; do
  if [ -r "$log" ] && grep -qE -- "$failed" "$log"; then
    grep -E -- "$failed" "$log"
    exit 1
  fi
  if [ -r "$log" ] && grep -qE -- "$captured" "$log"; then
    grep -E -- "$captured" "$log"
    exit 0
  fi
  # A traceback that no receipt follows is a runner that died: the
  # click-free runner prints its receipt line after `capture()` returns,
  # and a `capture()` that raised printed none.
  if [ -r "$log" ] && grep -qF -- "$traceback" "$log"; then
    if ! sed -n "/$traceback/,\$p" "$log" | grep -qE -- "$receipt"; then
      echo "waitrun: the runner died with a traceback and no receipt followed it" >&2
      tail -n 20 "$log" >&2
      exit 2
    fi
  fi
  # Every staged map has its receipt: the click-free runner prints
  # `Staged: …/map-N` before each take and its `"map_requested": N` receipt
  # after, so a log where the two sets agree is finished whatever else
  # runs — the `pgrep` below sees every lane's runner, and run913's
  # receipt sat 81 minutes behind run912's capture on the other pool lane
  # (Loop 3063, the fortieth tranche).
  if [ -r "$log" ] && grep -qE -- "$receipt" "$log"; then
    staged_maps=$(grep -oE '^Staged: .*/map-[0-9]+' "$log" | sed 's/.*map-//' | sort -un)
    receipt_maps=$(grep -E -- "$receipt" "$log" | grep -oE '"map_requested": [0-9]+' | grep -oE '[0-9]+' | sort -un)
    if [ -n "$staged_maps" ] && [ -z "$(comm -23 <(printf '%s\n' "$staged_maps") <(printf '%s\n' "$receipt_maps"))" ]; then
      grep -E -- "$receipt" "$log"
      if grep -E -- "$receipt" "$log" | grep -q '"success": false'; then
        exit 1
      fi
      exit 0
    fi
  fi
  # Whether the log shows a start is read before the process is asked
  # for, never after (gate-4685-4683, 2026-11-25): asked the other way
  # round, a runner that started and printed its first line between the
  # two reads was a log with a start and no process, and was called dead
  # while it played. Read first, a start the log shows and a process that
  # is gone are a runner that has ended; a receipt it printed on its way
  # out is read below, after the process check.
  started=0
  if [ -r "$log" ] && grep -qvE -- "$header" "$log"; then
    started=1
  fi
  if ! pgrep -f -- "$runner_pattern" >/dev/null 2>&1; then
    # No runner. The click-free lane's receipts are its verdict, read only
    # once the runner is gone: it prints one per map and may still be on
    # the next.
    if [ -r "$log" ] && grep -qE -- "$receipt" "$log"; then
      grep -E -- "$receipt" "$log"
      if grep -E -- "$receipt" "$log" | grep -q '"success": false'; then
        exit 1
      fi
      exit 0
    fi
    # A log that shows a runner started — any line past the launch's own
    # header (`args:`, `lane:`, `output:`) — and no runner alive is a dead
    # runner, called at once. One that shows none is a launch RonDriver
    # has not acted on yet, and it is given a grace period to start
    # (run1794, item 4566, parked 4566a: `viadriver.sh` returns before the
    # runner exists, the waiter found no process and no verdict, exited 2
    # the same second, and a chained script put the next take on the
    # second pool lane beside a game still playing). A log that never
    # appears, or never shows a start, is called when the grace has run.
    if [ "$started" -eq 1 ] || [ "$waited" -ge "$grace" ]; then
      echo "waitrun: no '$runner_pattern' process is alive and $log holds no verdict" >&2
      [ -r "$log" ] && tail -n 20 "$log" >&2
      exit 2
    fi
    waited=$((waited + poll))
  fi
  sleep "$poll"
done
