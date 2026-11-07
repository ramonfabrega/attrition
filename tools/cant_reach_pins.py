#!/usr/bin/env python3
"""cant_reach_pins.py — rewrite `diff::cant_reach::PINNED` from a test log.

Every test that holds `Pins` prints `  cant_reach tally <test> <n>` when its
tally or its pin is non-zero (`crates/rondata/src/diff/cant_reach.rs`, item
3962). This reads those lines from a `cargo test` log and prints the table;
`--write` replaces the table in the file with the log's counts, merged over
what is there: a test the log names takes the log's count (0 drops it), a
test it does not name keeps its pin.

    zsh tools/check.sh test --release -p rondata 2>&1 | tee /tmp/run.log
    python3 tools/cant_reach_pins.py /tmp/run.log            # report
    python3 tools/cant_reach_pins.py /tmp/run.log --write    # rewrite
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILE = ROOT / "crates/rondata/src/diff/cant_reach.rs"
LINE = re.compile(r"^\s*cant_reach tally (?P<test>\S+) (?P<n>\d+)\s*$")
TABLE = re.compile(r"(pub\(crate\) const PINNED: &\[\(&str, usize\)\] = &\[)(.*?)(\];)", re.S)
ROW = re.compile(r'\(\s*"(?P<test>[^"]+)",\s*(?P<n>\d+)\s*\)')


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--write"]
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    seen = {}
    for line in open(args[0], errors="replace"):
        m = LINE.match(line)
        if m:
            seen[m["test"]] = int(m["n"])
    text = FILE.read_text()
    m = TABLE.search(text)
    if not m:
        print(f"no PINNED table in {FILE}", file=sys.stderr)
        return 1
    table = {r["test"]: int(r["n"]) for r in ROW.finditer(m.group(2))}
    moved = {t: (table.get(t, 0), n) for t, n in seen.items() if table.get(t, 0) != n}
    for t, (was, now) in sorted(moved.items()):
        print(f"  {t}: {was} -> {now}")
    table.update(seen)
    table = {t: n for t, n in table.items() if n}
    print(f"{len(moved)} moved; {len(table)} tests, {sum(table.values())} units in all")
    if "--write" in sys.argv:
        body = "".join(f'\n    ("{t}", {n}),' for t, n in sorted(table.items()))
        FILE.write_text(text[: m.start(2)] + body + ("\n" if body else "") + text[m.end(2):])
        print(f"wrote {FILE.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
