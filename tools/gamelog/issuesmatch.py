#!/usr/bin/env python3
"""issuesmatch.py GOLDEN TRACE — did every `@` line issue as it did in the golden run?

    issuesmatch.py ~/ron-golden/ch23/map-14/rontrace.log rontrace-run318.log

An `@` line leaves an `INFO 17` record — `(frame, line index | refusal <<
16, package size before, after, objects named)` — and an `INFO 18` per
object it named, with the object's uid and position (`tools/trace/tracer.c`,
`issue_line`). A capture that re-runs a golden chapter's script is the same
game only if each line issued on the same frame, refused nothing, and grew
the package by the same bytes from the same objects in the same places.
This compares the two traces' records one for one and fails loudly on the
first that differs; run314 is why (every line refused with code 2 under
`cover=1`, item 934), and `cmdsran.py` cannot see it, since an `@` line
carries no `INFO cmd`.

A golden run with no `@` records fails too: a check that compares nothing
is not a check.

    issuesmatch.py --none-refused rontrace-run340.log [N]

is the same check for a stanza with no golden twin (parked 966): every `@`
line left an `INFO 17`, none refused, and — with N — there are exactly N of
them. A trace with no issue record fails here too.
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
    """[(kind, a, b, c, d, e)] for every INFO 17 and INFO 18, in order."""
    out = subprocess.run(
        [sys.executable, os.path.join(HERE, "..", "trace", "report.py"),
         trace, "summary"],
        capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit("report.py failed on %s:\n%s" % (trace, out.stderr.strip()))
    found = []
    for line in out.stdout.splitlines():
        m = re.match(r"\s*INFO (17|18)\s+((?:0x[0-9a-f]+\s*)+)$", line)
        if m:
            found.append((int(m.group(1)),) + tuple(int(v, 16) for v in m.group(2).split()))
    return found


def show(r):
    if r[0] == 17:
        return "frame %d line %d refusal %d package %d -> %d, %d named" % (
            r[1], r[2] & 0xffff, r[2] >> 16, r[3], r[4], r[5])
    return "  frame %d object %d of %d uid %d at %d,%d" % (
        r[1], r[2] & 0xffff, r[2] >> 16, r[3], r[4], r[5])


def none_refused(trace, count):
    got = records(trace)
    issued = [r for r in got if r[0] == 17]
    refused = [r for r in issued if r[2] >> 16]
    print("%s: %d issue records, %d refused" % (os.path.basename(trace), len(issued), len(refused)))
    for r in got:
        print("  " + show(r))
    if not issued:
        print("FAIL: the trace has no `@` record: nothing issued, or nothing was asked to")
        return 1
    if refused:
        print("FAIL: refused\n  " + "\n  ".join(show(r) for r in refused))
        return 1
    if count is not None and len(issued) != count:
        print("FAIL: %d lines issued, the stanza wrote %d" % (len(issued), count))
        return 1
    print("ok: every `@` line issued, none refused")
    return 0


def main():
    if len(sys.argv) in (3, 4) and sys.argv[1] == "--none-refused":
        trace = sys.argv[2]
        if not os.path.isabs(trace):
            trace = os.path.join(ARCHIVE, trace)
        return none_refused(trace, int(sys.argv[3]) if len(sys.argv) == 4 else None)
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    golden, trace = sys.argv[1], sys.argv[2]
    if not os.path.isabs(trace):
        trace = os.path.join(ARCHIVE, trace)
    want, got = records(golden), records(trace)
    issued = [r for r in got if r[0] == 17]
    print("%s: %d issue records, %d refused; golden %d"
          % (os.path.basename(trace), len(issued),
             sum(1 for r in issued if r[2] >> 16), sum(1 for r in want if r[0] == 17)))
    for r in got:
        print("  " + show(r))
    if not want:
        print("FAIL: the golden run has no `@` records to compare")
        return 1
    for i, (w, g) in enumerate(zip(want, got)):
        if w != g:
            print("FAIL: record %d differs\n  golden   %s\n  captured %s" % (i, show(w), show(g)))
            return 1
    if len(want) != len(got):
        print("FAIL: golden has %d records, the capture %d" % (len(want), len(got)))
        return 1
    print("ok: every record as in the golden run")
    return 0


if __name__ == "__main__":
    sys.exit(main())
