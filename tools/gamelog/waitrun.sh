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
# Exit status: 0 — the banner arrived and every check passed; 1 — the banner
# arrived and a check FAILED; 2 — no runner is alive and no banner came (the
# queue died, or was never started; read the log's tail this prints);
# 64 — usage.
set -u
log=${1:-}
poll=${2:-20}
if [ -z "$log" ]; then
  echo "usage: waitrun.sh <viadriver-log> [poll-seconds]" >&2
  exit 64
fi
banner='=== the queue, as it went ==='
runner_pattern=${WAITRUN_RUNNER:-'gamelog/runqueue.sh'}

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
  if ! pgrep -f -- "$runner_pattern" >/dev/null 2>&1; then
    # No runner. Give a fresh launch one more poll to start, then call it.
    if [ -r "$log" ] || [ "$missing" -ge 1 ]; then
      echo "waitrun: no '$runner_pattern' process is alive and $log holds no banner" >&2
      [ -r "$log" ] && tail -n 20 "$log" >&2
      exit 2
    fi
    missing=$((missing + 1))
  fi
  sleep "$poll"
done
