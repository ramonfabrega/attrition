#!/usr/bin/env python3
"""Refuse a Bash command that signals processes by name (a PreToolUse hook).

**Why this exists.** On 2026-10-20 a sweep lane's subagent ran, every eleven
seconds for four and a half minutes,

    pkill -f "debug/deps/sweep_u_z-a19… unit_find_path" -n

macOS `pkill` stops reading options at its first operand, so the trailing `-n`
became a second pattern and `-f` matched it against every process's whole
argument list: every `rustc` (`--crate-name`), every budgeted test (`nice -n`,
`--nocapture`) and every Claude Code shell whose command held a `grep -n` got
SIGTERM. Two lanes reported it as two different faults — "release builds
SIGTERM'd from outside" and "check.sh dies with exit 144 while waiting for
admission" — and the admission budget was suspected for a day
(`docs/audit/2026-10-21-signals-3392-3394.md`). Parked 1713's rule, "kill a
gate by its pid, never `pkill -f`", had been prose for a month.

**The shape.** Three refusals, each the incident's own:
  - `killall <name>` and `pkill -f <pattern>`: a signal by name reaches other
    lanes' processes on a shared box; kill by pid (`kill <pid>`, `pkill -P`).
  - `pgrep`/`pkill` with an option token after the first operand: that token
    is a second pattern, not an option, on macOS.
A `pgrep` that only reads passes; so does `kill` with a pid, and any mention
of these words that is not a command (an `echo`, a `grep`).

**Wiring.** `.claude/settings.json`, beside `tools/pollguard.py`'s line:
    g="${CLAUDE_PROJECT_DIR:-$PWD}/tools/killguard.py"; [ -f "$g" ] || exit 0; python3 "$g"
The classifier refuses a session's edit of its own hooks, so that line is
the user's to add (the forty-seventh pass, DECISIONS 86).
"""

import json
import shlex
import sys

SEPARATORS = {';', '&&', '||', '|', '&', '(', ')', '{', '}', 'do', 'then', 'else', 'done', 'fi'}
KILLERS = ('pkill', 'pgrep', 'killall')


def _commands(command):
    """The argument lists of every simple command in `command`, split on the
    shell's separators; a line shlex cannot read is split on whitespace."""
    try:
        tokens = shlex.split(command, posix=True)
    except ValueError:
        tokens = command.split()
    out, cur = [], []
    for tok in tokens:
        if tok in SEPARATORS or tok.endswith(';'):
            if tok.endswith(';') and tok not in SEPARATORS:
                cur.append(tok[:-1])
            if cur:
                out.append(cur)
            cur = []
        else:
            cur.append(tok)
    if cur:
        out.append(cur)
    return out


def offence(command):
    """Why `command` is refused, or None."""
    for argv in _commands(command):
        name = argv[0].rsplit('/', 1)[-1] if argv else ''
        if name not in KILLERS:
            continue
        if name == 'killall':
            return (f"`killall {' '.join(argv[1:])}` signals by name and reaches other lanes' processes; "
                    "kill by pid (`kill <pid>`) — parked 1713, Loop 3392/3394")
        args = argv[1:]
        first_operand = next((i for i, a in enumerate(args) if not a.startswith('-')), None)
        options = args[:first_operand] if first_operand is not None else args
        if first_operand is not None:
            stray = [a for a in args[first_operand + 1:] if a.startswith('-')]
            if stray:
                return (f"`{name}` reads `{stray[0]}` after its first operand as a SECOND PATTERN, not an option "
                        f"(macOS stops option parsing at the first operand): on 2026-10-20 `pkill -f <pattern> -n` "
                        "sent SIGTERM to every process whose arguments held `-n` — every rustc and every "
                        "budgeted test on the box (docs/audit/2026-10-21-signals-3392-3394.md). Put the options "
                        "first, or kill by pid")
        if name == 'pkill' and any(a == '-f' or (a.startswith('-') and not a.startswith('--') and 'f' in a[1:])
                                   for a in options):
            return ("`pkill -f` signals every process whose arguments match, on a box other lanes share; "
                    "kill a gate by its `Gate pid:` and a test by its pid (parked 1713, Loop 3392/3394)")
    return None


def main():
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0  # Never break a tool call over our own parse failure.
    command = (payload.get('tool_input') or {}).get('command') or ''
    why = offence(command)
    if why is None:
        return 0
    print(json.dumps({'hookSpecificOutput': {'hookEventName': 'PreToolUse',
                                             'permissionDecision': 'deny',
                                             'permissionDecisionReason': 'Kill guard: ' + why}}))
    return 0


if __name__ == '__main__':
    sys.exit(main())
