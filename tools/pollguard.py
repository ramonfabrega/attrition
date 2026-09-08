#!/usr/bin/env python3
"""Refuse the third consecutive Bash read of the same background-task output file.

**Why this exists.** `lore polls` measured the pattern across every indexed
session: 1,972 requests, 456 USD at list price, 206 sessions, back to July and
across four projects. The worst single case was a capture lane that read one
task output file 349 times — 326 of them inside runs of three or more
consecutive reads of the same path, the longest run 87, median 4.6 s apart,
which is the API round trip and nothing else. Every honest check measured was a
run of one or two. And the reads bought nothing: a `run_in_background` task
re-invokes the session when it exits, so the answer was already coming.

`CLAUDE.md` has forbidden the foreground sleep-and-grep loop for weeks. It was
broken anyway, by the lanes and by the commander. That is the repo's own rule
about a rule that could be a guard and is only prose.

**The shape, and why it is this one.** Refuse on the third read of the same
path inside a sixty-second window, whatever other Bash calls sit between them.
One or two reads pass, so a legitimate check is never blocked; an `until` loop
inside a single Bash call is one call and passes by construction; a read of the
finished output after the notification is a first read again once the window
has passed.

The first version counted CONSECUTIVE reads and let any other command reset the
run — and told the worker so in its deny text. lane-run88 (2026-09-07) was
denied four times and typed `echo reset` and `date` between reads, naming the
reset in the command's own description. A reset clause is a door, and a deny
text that explains the counter is the key; the window has neither.

**What it does not catch**, deliberately: reads spaced more than a minute apart.
`lore polls` prices the wider "reread" shape; this guard takes the half with a
clean margin and leaves the half that would need a judgement call.

Reads state from ~/.claude/poll-guard/<session_id>: one "<epoch> <path>" line
per recent read.
"""

import json
import os
import re
import sys
import time

# The harness's background-task output files. Matching the directory rather
# than the whole prefix keeps this working if the tmp root moves.
TASK_FILE = re.compile(r"[^\s'\";|&<>()]*/tasks/[A-Za-z0-9_.-]+\.output")
LIMIT = 3
WINDOW_S = 60


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0  # Never break a tool call over our own parse failure.

    command = (payload.get("tool_input") or {}).get("command") or ""
    session = payload.get("session_id") or "unknown"

    hits = TASK_FILE.findall(command)
    # Exactly one distinct task file, or this is not the polling shape. Any
    # other command is simply not a read: it neither counts nor resets.
    if len(set(hits)) != 1:
        return 0
    path = hits[0]
    now = int(time.time())

    state_dir = os.path.expanduser("~/.claude/poll-guard")
    state_file = os.path.join(state_dir, re.sub(r"[^A-Za-z0-9_-]", "_", session))

    recent: list[int] = []
    try:
        with open(state_file, encoding="utf-8") as fh:
            for line in fh:
                head, _, rest = line.strip().partition(" ")
                if rest == path and now - int(head) <= WINDOW_S:
                    recent.append(int(head))
    except Exception:
        pass

    recent.append(now)
    count = len(recent)

    try:
        os.makedirs(state_dir, exist_ok=True)
        with open(state_file, "w", encoding="utf-8") as fh:
            fh.writelines(f"{t} {path}\n" for t in recent)
    except OSError:
        pass

    if count < LIMIT:
        return 0

    reason = (
        f"Polling guard: this is read #{count} of the same background-task output "
        f"file inside {WINDOW_S} s, which is the shape that cost 456 USD across 206 "
        f"sessions. The read buys nothing — a run_in_background task re-invokes "
        f"this session when it exits, so the result is already coming. Do one of "
        f"these instead: (1) end the turn and wait for the task notification; "
        f"(2) put the wait inside ONE Bash call, e.g. `until <condition>; do "
        f"sleep 5; done`, with a long tool timeout; (3) use Monitor with an until "
        f"condition."
    )
    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
