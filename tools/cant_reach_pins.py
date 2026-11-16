#!/usr/bin/env python3
"""cant_reach_pins.py — rewrite `diff::cant_reach::PINNED` from a test log.

Every test that holds `Pins` prints `  cant_reach tally <test> <n>` when its
tally or its pin is non-zero — libtest shows it only for a failing test, so
a log of a green suite names nothing, and `--nocapture` shows every one — (`crates/rondata/src/diff/cant_reach.rs`, item
3962). This reads those lines from a `cargo test` log and prints the table;
`--write` replaces the table in the file with the log's counts, merged over
what is there: a test the log names takes the log's count (0 drops it), a
test it does not name keeps its pin.

The rewrite is in place, so the table's comments stay where they stood
(item 4159's third residue: rebuilding the table dropped every one): a
moved row's count is replaced, a new row goes in by name, and a row that
leaves takes only its own code — a comment inside it or after it stands
on its own line where the row was. A row the log does not move is left
byte for byte, a pin of 0 kept by hand with it.

    zsh tools/check.sh test --release -p rondata -- --nocapture 2>&1 | tee /tmp/run.log
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
# rustfmt breaks a long row over four lines with a trailing comma; matched
# on the text with its comments blanked, so a comment inside a row is
# whitespace to it and its offsets are the file's.
ROW = re.compile(r'\(\s*"(?P<test>[^"]+)",\s*(?P<n>\d+),?\s*\)(?:[ \t]*,)?')


def blank_comments(text):
    """`text` with every `// …` outside a string turned to spaces of the
    same length."""
    out, k, in_str = [], 0, False
    while k < len(text):
        c = text[k]
        if in_str:
            out.append(c)
            if c == "\\" and k + 1 < len(text):
                k += 1
                out.append(text[k])
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out.append(c)
        elif text.startswith("//", k):
            end = text.find("\n", k)
            end = len(text) if end < 0 else end
            out.append(" " * (end - k))
            k = end
            continue
        else:
            out.append(c)
        k += 1
    return "".join(out)


def comments_in(raw, blank, a, b):
    """The comments that start inside `raw[a:b]`, each to its line's end."""
    out, k = [], a
    while k < b:
        if blank[k] == " " and raw.startswith("//", k):
            end = raw.find("\n", k)
            end = len(raw) if end < 0 else end
            out.append(raw[k:end])
            k = end
        k += 1
    return out


def rewrite(text, seen):
    """`(new_text, moved, table)`: `text` with `PINNED` re-pinned from
    `seen` in place, or None when there is no table. `moved` is
    `{test: (was, now)}`; `table` the merged pins, zeros out."""
    m = TABLE.search(text)
    if not m:
        return None
    blank = blank_comments(text)
    rows = [r for r in ROW.finditer(blank, m.start(2), m.end(2))]
    table = {r["test"]: int(r["n"]) for r in rows}
    moved = {t: (table.get(t, 0), n) for t, n in seen.items() if table.get(t, 0) != n}
    edits = []
    for r in rows:
        if r["test"] not in moved:
            continue
        now = moved[r["test"]][1]
        if now:
            edits.append((r.start("n"), r.end("n"), str(now)))
            continue
        # The row leaves: its own code goes, its comments stand.
        ls = text.rfind("\n", 0, r.start()) + 1
        start = ls if not text[ls:r.start()].strip() else r.start()
        indent = re.match(r"[ \t]*", text[ls:]).group(0)
        kept = comments_in(text, blank, r.start(), r.end())
        eol = text.find("\n", r.end())
        eol = len(text) if eol < 0 else eol
        rest = text[r.end():eol]
        end = r.end()
        if not rest.strip() or rest.strip().startswith("//"):
            kept += [rest.strip()] if rest.strip() else []
            end = min(eol + 1, len(text))
        edits.append((start, end, "".join(f"{indent}{c}\n" for c in kept)))
    # Descending, so rows inserted at one point land in ascending order.
    for t in sorted((t for t, (was, now) in moved.items() if t not in table and now), reverse=True):
        row = f'    ("{t}", {moved[t][1]}),\n'
        after = [r for r in rows if r["test"] > t]
        if after:
            ls = text.rfind("\n", 0, after[0].start()) + 1
            at = ls if not text[ls:after[0].start()].strip() else after[0].start()
        elif rows:
            eol = text.find("\n", rows[-1].end())
            at = m.end(2) if eol < 0 or eol >= m.end(2) else eol + 1
        else:
            at = m.end(2)
        if text[at - 1] != "\n":
            row = "\n" + row
        edits.append((at, at, row))
    # Last offset first (the sort is stable, so equal points keep the order above).
    for a, b, new in sorted(edits, key=lambda e: (e[0], e[1]), reverse=True):
        text = text[:a] + new + text[b:]
    merged = {**table, **seen}
    return text, moved, {t: n for t, n in merged.items() if n}


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--write"]
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    seen = {}
    with open(args[0], errors="replace") as log:
        for line in log:
            m = LINE.match(line)
            if m:
                seen[m["test"]] = int(m["n"])
    out = rewrite(FILE.read_text(), seen)
    if out is None:
        print(f"no PINNED table in {FILE}", file=sys.stderr)
        return 1
    text, moved, table = out
    for t, (was, now) in sorted(moved.items()):
        print(f"  {t}: {was} -> {now}")
    print(f"{len(moved)} moved; {len(table)} tests, {sum(table.values())} units in all")
    if "--write" in sys.argv:
        FILE.write_text(text)
        print(f"wrote {FILE.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
