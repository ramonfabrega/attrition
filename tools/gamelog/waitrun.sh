#!/bin/zsh
# waitrun.sh <viadriver-log> [poll-seconds] — block until a detached capture
# queue has finished, then print its summary and exit with its verdict.
#
#   zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 174
#   # prints "log: /tmp/ron-runs/viadriver-<ts>.log"
#   zsh tools/gamelog/waitrun.sh /tmp/ron-runs/viadriver-<ts>.log
#
# Why this exists (parked 656, the thirteenth pass, 2026-09-23): `viadriver.sh`
# hands the queue to RonDriver.app and returns at once, so the session that
# launched it has nothing to wait on. Item 571 waited by grepping
# `runqueue-<ts>.log` for the banner `runqueue.sh` prints at its end — but the
# banner goes to the script's **stdout**, which RonDriver appends to the
# viadriver log, never to the runqueue log — so the loop spun for two hours
# after the capture finished. This script watches the file the banner actually
# lands in, and it also notices a runner that died without printing one.
#
# Run it under the harness's background lane (`run_in_background`) and end the
# turn: the harness re-invokes the session when this exits. The sleeping is
# this script's, not the session's.
#
# The click-free lane (`tools/explore/unattended_capture.py`, the golden
# chapters' runner) prints no banner: its verdict is one JSON receipt line per
# map, `{"map_requested": …, "success": true|false, …}`, on the same stdout.
# Four items of the fourteenth pass's tranche (651, 676, 696, 714) found this
# script exiting 2 at once on that lane and fell back on reading the receipt
# by hand, so the receipt is read here: once no runner is alive, a log holding
# receipts is judged by them.
#
# Exit status: 0 — the banner arrived and every check passed, or the runner
# has exited and every receipt says `"success": true`, or a lone capture
# script said `=== captured: `; 1 — the banner arrived and a check FAILED,
# or a receipt says `"success": false`, or the script said `=== capture
# failed: `; 2 — no runner is
# alive and neither a banner nor a receipt came (the queue died, or was never
# started; read the log's tail this prints), or the log ends in a Python
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
banner='=== the queue, as it went ==='
receipt='^{"map_requested"'
runner_pattern=${WAITRUN_RUNNER:-'gamelog/runqueue.sh|unattended_capture.py|gamelog/startcapture.sh'}
# A capture script launched on its own, with no queue around it, ends on one
# of these (`startcapture.sh`; parked 1080, the nineteenth pass): run381
# finished and this script called it a dead runner.
captured='^=== captured: '
failed='^=== capture failed: '
traceback='Traceback (most recent call last):'

# `run_in_background` reports a task the moment it exits, so a launch that
# has not written its log yet is waited for too — but only for a while: a
# log that never appears is a launch that never happened.
missing=0
while :; do
  if [ -r "$log" ] && grep -qF -- "$banner" "$log"; then
    sed -n "/$banner/,\$p" "$log"
    if sed -n "/$banner/,\$p" "$log" | grep -q 'checks FAILED'; then
      exit 1
    fi
    exit 0
  fi
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
    # Give a fresh launch one more poll to start, then call it.
    if [ -r "$log" ] || [ "$missing" -ge 1 ]; then
      echo "waitrun: no '$runner_pattern' process is alive and $log holds no banner" >&2
      [ -r "$log" ] && tail -n 20 "$log" >&2
      exit 2
    fi
    missing=$((missing + 1))
  fi
  sleep "$poll"
done
