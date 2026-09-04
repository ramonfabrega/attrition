#!/usr/bin/env python3
"""cmdsran.py TRACE [FRAME:COUNT ...] — did the scenario's cheat lines run?

    cmdsran.py rontrace-run43.log 100:2

A capture staged from `rontrace.cmd` can fail in a way that leaves a
perfectly ordinary-looking dump behind: the lines are parsed at attach and
then, on their frame, handed to `ConsoleWin::parse_cmd`, which can refuse
one — a misspelled type name, a coordinate off the map, a command in the
half the line's `!` prefix does not reach. The game plays on and the archive
is a capture of the scenario that did *not* happen. run23 is the standing
example: its `war` line changed nothing, and only a word-for-word comparison
against run21 said so.

The trace already carries the answer. Every executed line is an `INFO cmd`
record — `(frame, line index, from_chat, parse_cmd's return)` — and `INFO
cmds` at attach is how many parsed (`tools/trace/README.md`). This reads
them back and fails loudly when a line did not run, or ran and returned 0.

With no `FRAME:COUNT` arguments it prints what it found and asks nothing.
With them it asserts, per frame, that exactly COUNT lines ran there and that
every one returned non-zero.
"""
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def records(trace):
    """[(frame, index, from_chat, ret)] for every INFO cmd in the trace."""
    out = subprocess.run(
        [sys.executable, os.path.join(HERE, "..", "trace", "report.py"),
         trace, "summary"],
        capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit("report.py failed on %s:\n%s" % (trace, out.stderr.strip()))
    found, parsed = [], None
    for line in out.stdout.splitlines():
        m = re.match(r"\s*INFO cmds\s+(0x[0-9a-f]+)", line)
        if m:
            parsed = int(m.group(1), 16)
            continue
        m = re.match(r"\s*INFO cmd\s+(0x[0-9a-f]+)\s+(0x[0-9a-f]+)"
                     r"\s+(0x[0-9a-f]+)\s+(0x[0-9a-f]+)", line)
        if m:
            found.append(tuple(int(g, 16) for g in m.groups()))
    return found, parsed


def main():
    trace = sys.argv[1]
    if not os.path.isabs(trace):
        trace = os.path.join(ARCHIVE, trace)
    found, parsed = records(trace)
    print("%s: %s lines parsed at attach, %d ran"
          % (os.path.basename(trace),
             "?" if parsed is None else parsed, len(found)))
    for frame, idx, chat, ret in found:
        print("  frame %-6d line %-3d %-7s returned %d"
              % (frame, idx, "chat" if chat else "console", ret))

    bad = 0
    for want in sys.argv[2:]:
        frame, _, count = want.partition(":")
        frame, count = int(frame), int(count)
        here = [r for r in found if r[0] == frame]
        if len(here) != count:
            print("FAIL: frame %d ran %d lines, expected %d"
                  % (frame, len(here), count))
            bad += 1
        refused = [r for r in here if r[3] == 0]
        if refused:
            print("FAIL: frame %d had %d line(s) parse_cmd refused: %s"
                  % (frame, len(refused), refused))
            bad += 1
    if bad:
        return 1
    if len(sys.argv) > 2:
        print("every asserted line ran and was accepted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
