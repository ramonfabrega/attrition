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

**The shape, and why it is this one.** Refuse on the third CONSECUTIVE read of
the same path, where "consecutive" means no other Bash call in between. One or
two reads pass, so a legitimate check is never blocked; an `until` loop inside a
single Bash call is one call and passes by construction. Any other Bash command
resets the run, which is what keeps the rule about polling rather than about
counting.

**What it does not catch**, deliberately: a session alternating between two long
jobs never reaches three in a row on one path (1,119 of the 1,975 measured reads
are this wider "reread" shape). `lore polls` reports both columns; this guard
takes the half with a clean margin and leaves the half that would need a
judgement call.

Reads state from ~/.claude/poll-guard/<session_id>, one line: "<count> <path>".
"""

import json
import os
import re
import sys

# The harness's background-task output files. Matching the directory rather
# than the whole prefix keeps this working if the tmp root moves.
TASK_FILE = re.compile(r"[^\s'\";|&<>()]*/tasks/[A-Za-z0-9_.-]+\.output")
LIMIT = 3


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0  # Never break a tool call over our own parse failure.

    command = (payload.get("tool_input") or {}).get("command") or ""
    session = payload.get("session_id") or "unknown"

    hits = TASK_FILE.findall(command)
    # Exactly one distinct task file, or this is not the polling shape.
    path = hits[0] if len(set(hits)) == 1 else None

    state_dir = os.path.expanduser("~/.claude/poll-guard")
    state_file = os.path.join(state_dir, re.sub(r"[^A-Za-z0-9_-]", "_", session))

    last_path, count = None, 0
    try:
        with open(state_file, encoding="utf-8") as fh:
            head, _, rest = fh.read().strip().partition(" ")
            count, last_path = int(head), rest
    except Exception:
        pass

    if path is None:
        # Any other command breaks the run. That is what "consecutive" means.
        try:
            os.remove(state_file)
        except OSError:
            pass
        return 0

    count = count + 1 if path == last_path else 1

    try:
        os.makedirs(state_dir, exist_ok=True)
        with open(state_file, "w", encoding="utf-8") as fh:
            fh.write(f"{count} {path}")
    except OSError:
        pass

    if count < LIMIT:
        return 0

    reason = (
        f"Polling guard: this is read #{count} in a row of the same background-task "
        f"output file with nothing in between, which is the shape that cost 456 USD "
        f"across 206 sessions. The read buys nothing — a run_in_background task "
        f"re-invokes this session when it exits, so the result is already coming. "
        f"Do one of these instead: (1) end the turn and wait for the task "
        f"notification; (2) put the wait inside ONE Bash call, e.g. "
        f"`until <condition>; do sleep 5; done`, with a long tool timeout; "
        f"(3) use Monitor with an until condition. Run any other Bash command to "
        f"reset this counter."
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
