#!/usr/bin/env python3
"""ctx.py — the context this session's next turn carries, in thousands.

`docs/DECISIONS.md` entry 134 §9. A lane was told it was deep by the driver's
waiter, at 400 k, and cleared wherever it stood; six of thirteen lanes of one
tranche passed 300 k and cost 11 to 17 USD where the shallow ones cost 4 to
7. A lane that can read its own depth can plan a landing instead: it reads
this at the end of its first act and before its build.

  python3 tools/ctx.py        prints `context: 212 k` and the plan line

The number is `lore agents --json`'s `ctx` for this session
(`CLAUDE_CODE_SESSION_ID`): input plus cache read plus cache write of the
last request. No session id, no lore or no row: it says so and exits 0, so
no lane stops on the instrument.
"""
import json
import os
import subprocess
import sys

PLAN_AT_K = 200


def ctx_of(rows, session):
    """The `ctx` of the row whose `sessionId` is `session`, or None."""
    for row in rows or []:
        if session and row.get('sessionId') == session and isinstance(row.get('ctx'), int):
            return row['ctx']
    return None


def rows_of(data):
    """The listing's rows: a list, or a dict's `agents` (lore b179) or `rows`."""
    if isinstance(data, list):
        return data
    return data.get('agents') or data.get('rows') or []


def line(ctx):
    k = ctx // 1000
    if k >= PLAN_AT_K:
        return (f'context: {k} k — past {PLAN_AT_K} k: if the build is not begun, land the first act '
                '(journal, value diff, the next word) and end done; never clear mid-build '
                '(DECISIONS 134 §9)')
    return f'context: {k} k — under {PLAN_AT_K} k: carry on'


def main():
    session = os.environ.get('CLAUDE_CODE_SESSION_ID')
    env = dict(os.environ)
    home = os.path.expanduser('~')
    env['PATH'] = f"{home}/.local/bin:{home}/.bun/bin:/opt/homebrew/bin:" + env.get('PATH', '')
    try:
        out = subprocess.run(['lore', 'agents', '--json'], capture_output=True, text=True, env=env, timeout=60)
        rows = rows_of(json.loads(out.stdout))
    except (OSError, ValueError, subprocess.TimeoutExpired) as e:
        print(f'context: not read ({type(e).__name__}); carry on')
        return 0
    ctx = ctx_of(rows, session)
    print(line(ctx) if ctx is not None else 'context: not read (no row for this session); carry on')
    return 0


if __name__ == '__main__':
    sys.exit(main())
