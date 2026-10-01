#!/usr/bin/env python3
"""repin.py — rewrite the `want` of every `pin_eq!` a test run reported moved.

A floor or widening test takes `let _pins = Pins::hold();` and reports
every pin that moved at once (`rondata::diff::testkit::pins`, parked 973):

    3 pins moved:
      crates/rondata/src/diff/second.rs:570: run471 whole: blocks 5371..5627
        got  258
        want 257

Three landings re-pinned 140, 8 and 85 of these by scratch script (1330,
1407, 1398; parked 1349 and 1411 — the third reach). This reads the
report from a `cargo test` log and rewrites each site's `want` with its
`got`, when the `want` is a literal the `got` can replace: a number, a
tuple or array of them, `None`, `Some(…)`, a string, a bool. A `want`
that is an expression — a constant's name, a call — is printed as a site
to re-pin by hand, with its got and want, and left alone; so is a site
two tests moved to two values. `pin!` and `pin_ne!` sites are listed and
never rewritten: they have no `want`.

    cargo test --release -p rondata sahara_toughest 2>&1 | tee /tmp/run.log
    python3 tools/repin.py /tmp/run.log            # report, write nothing
    python3 tools/repin.py /tmp/run.log --write    # rewrite the literal sites

The delta in each constant's comment and the word's block in its widening
stay the worker's to write (`CLAUDE.md`, "the re-pin is split on purpose").
"""
import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SITE = re.compile(r'^\s*(?P<file>[\w./-]+\.rs):(?P<line>\d+): (?P<msg>.*)$')
GOT = re.compile(r'^\s*got  (?P<got>.*)$')
WANT = re.compile(r'^\s*want (?P<want>.*)$')
BOTH = re.compile(r'^\s*both (?P<both>.*)$')
# A `want` the `got`'s Debug print can replace: literals, nested.
LITERAL = re.compile(
    r'^(?:&?(?:vec!)?)?(?:None|true|false|-?\d[\d_]*(?:\.\d+)?|"(?:[^"\\]|\\.)*"'
    r"|'(?:[^'\\]|\\.)'|Some\(.*\)|[\[(].*[\])])$", re.S)


def parse(log):
    """`{(file, line): [(msg, got, want)]}` from every moved-pin report in a
    log; a `pin!`/`pin_ne!` site has `got` and `want` of None."""
    sites = defaultdict(list)
    lines = log.split('\n')
    i = 0
    while i < len(lines):
        m = SITE.match(lines[i])
        if not m:
            i += 1
            continue
        got = want = None
        j = i + 1
        if j < len(lines) and GOT.match(lines[j]):
            got = GOT.match(lines[j]).group('got').strip()
            j += 1
            if j < len(lines) and WANT.match(lines[j]):
                want = WANT.match(lines[j]).group('want').strip()
                j += 1
        elif j < len(lines) and BOTH.match(lines[j]):
            j += 1
        sites[(m.group('file'), int(m.group('line')))].append((m.group('msg'), got, want))
        i = j
    return sites


def invocation(text, line):
    """`(start, end)` offsets of the `pin_eq!(…)` whose opening is on
    `line` (1-based), or None."""
    offset = sum(len(l) + 1 for l in text.split('\n')[:line - 1])
    m = re.compile(r'pin_eq!\s*\(').search(text, offset)
    if not m or text[offset:m.start()].count('\n') > 0:
        return None
    depth, k, in_str = 0, m.end() - 1, False
    while k < len(text):
        c = text[k]
        if in_str:
            if c == '\\':
                k += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
            if depth == 0:
                return m.start(), k + 1
        k += 1
    return None


def args_of(text, start, end):
    """The top-level arguments of `pin_eq!(…)` as `(arg_start, arg_end)`
    offsets, whitespace trimmed."""
    inner_start = text.index('(', start) + 1
    depth, in_str, cut, out = 0, False, inner_start, []
    k = inner_start
    while k < end - 1:
        c = text[k]
        if in_str:
            if c == '\\':
                k += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        elif c == ',' and depth == 0:
            out.append((cut, k))
            cut = k + 1
        k += 1
    if text[cut:end - 1].strip():
        out.append((cut, end - 1))
    trimmed = []
    for a, b in out:
        while a < b and text[a].isspace():
            a += 1
        while b > a and text[b - 1].isspace():
            b -= 1
        trimmed.append((a, b))
    return trimmed


def replacement(want_src, got):
    """The source to put where `want_src` was, keeping a `&` or `vec!`
    prefix the Debug print drops; None when the want is not a literal."""
    if not LITERAL.match(want_src.strip()):
        return None
    prefix = re.match(r'^(&?(?:vec!)?)', want_src.strip()).group(1)
    return prefix + got


def plan(sites, root=ROOT):
    """`(edits, by_hand)`: edits as `{file: [(start, end, new)]}` by offset
    into the file's text; by_hand as `[(file, line, why, msg, got, want)]`."""
    edits = defaultdict(list)
    by_hand = []
    for (file, line), reports in sorted(sites.items()):
        msg, got, want = reports[0]
        if got is None:
            by_hand.append((file, line, 'no want to rewrite (pin!/pin_ne!)', msg, None, None))
            continue
        gots = {g for _, g, _ in reports}
        if len(gots) > 1:
            by_hand.append((file, line, f'moved to {len(gots)} values', msg, ' | '.join(sorted(gots)), want))
            continue
        path = root / file
        if not path.exists():
            by_hand.append((file, line, 'no such file', msg, got, want))
            continue
        text = path.read_text()
        span = invocation(text, line)
        if not span:
            by_hand.append((file, line, 'no pin_eq! opens on that line', msg, got, want))
            continue
        arguments = args_of(text, *span)
        if len(arguments) < 2:
            by_hand.append((file, line, 'pin_eq! with one argument', msg, got, want))
            continue
        a, b = arguments[1]
        new = replacement(text[a:b], got)
        if new is None:
            by_hand.append((file, line, 'the want is not a literal', msg, got, want))
            continue
        edits[file].append((a, b, new))
    return edits, by_hand


def apply(edits, root=ROOT):
    """Write every edit, last offset first, so earlier offsets hold."""
    for file, spans in edits.items():
        path = root / file
        text = path.read_text()
        for a, b, new in sorted(spans, reverse=True):
            text = text[:a] + new + text[b:]
        path.write_text(text)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('log', help='a cargo test log holding "N pins moved:" reports')
    ap.add_argument('--write', action='store_true', help='rewrite the literal sites (default: report)')
    ap.add_argument('--root', default=str(ROOT))
    a = ap.parse_args(argv)
    root = Path(a.root)
    sites = parse(Path(a.log).read_text())
    edits, by_hand = plan(sites, root)
    n = sum(len(v) for v in edits.values())
    print(f'{len(sites)} site(s) moved: {n} literal, {len(by_hand)} by hand')
    for file, spans in sorted(edits.items()):
        for a, b, new in sorted(spans):
            text = (root / file).read_text()
            line = text[:a].count('\n') + 1
            print(f'  {file}:{line}: {text[a:b].strip()} -> {new}')
    for file, line, why, msg, got, want in by_hand:
        print(f'  by hand: {file}:{line}: {why} — {msg}' + (f' (got {got}, want {want})' if got else ''))
    if a.write and edits:
        apply(edits, root)
        print(f'wrote {n} pin(s) in {len(edits)} file(s)')
    elif not a.write and edits:
        print('nothing written: pass --write')
    return 0


if __name__ == '__main__':
    sys.exit(main())
