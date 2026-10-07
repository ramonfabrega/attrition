#!/usr/bin/env python3
"""suite_timings.py — time every rondata release test alone, for the lane gate.

    python3 tools/suite_timings.py <install> --out docs/audit/<date>-suite-timings.txt
    python3 tools/suite_timings.py --stale            # the tests the file has never timed

`tools/release_gate.py --lane` skips the rondata tests its timings file
(`TIMINGS` there) measured over its budget and runs every test the file
does not name — so every widening a landing adds runs on every lane gate
until the file is re-made, and the file ages a test a landing (parked
1573). The twenty-sixth pass made the first file by hand; this is the
third reach of that shape, as a tool: build the test binary once, list its
tests, run each in its own process with `--test-threads 1`, and write
`seconds<TAB>name` rows under the header `release_gate.py` reads. The
pass re-times the suite with it and commits the file; `--stale` says how
far the file has aged, from the same list.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))

HEADER_ROW = re.compile(r'^(?P<secs>\d+(?:\.\d+)?)\t(?P<name>\S+)$')


def timed(text):
    """`{name: seconds}` from a timings file's text; `#` lines are comments."""
    out = {}
    for line in text.splitlines():
        m = HEADER_ROW.match(line)
        if m:
            out[m.group('name')] = float(m.group('secs'))
    return out


def test_binaries(cwd=ROOT):
    """The rondata release test executables, built (`cargo test --no-run`)."""
    done = subprocess.run(
        ['cargo', 'test', '--release', '-p', 'rondata', '--no-run', '--message-format=json'],
        cwd=cwd, capture_output=True, text=True, check=True)
    out = []
    for line in done.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get('reason') == 'compiler-artifact' and msg.get('executable') \
                and msg.get('profile', {}).get('test'):
            out.append(msg['executable'])
    return out


def list_tests(binary, ignored=False):
    args = [binary, '--list', '--format', 'terse'] + (['--ignored'] if ignored else [])
    done = subprocess.run(args, capture_output=True, text=True, check=True)
    return [l[:-len(': test')] for l in done.stdout.splitlines() if l.endswith(': test')]


def time_one(binary, name, env):
    t0 = time.monotonic()
    done = subprocess.run([binary, '--exact', name, '--test-threads', '1'],
                          env=env, capture_output=True, text=True, check=False)
    return time.monotonic() - t0, done.returncode


def render(rows, when, tree, wall_note=''):
    """The file's text: the header `release_gate.py` reads past, then rows,
    slowest first."""
    rows = sorted(rows, key=lambda r: -r[0])
    total = sum(s for s, _ in rows)
    top = lambda n: sum(s for s, _ in rows[:n])  # noqa: E731
    under = sum(1 for s, _ in rows if s < 1)
    by_module = {}
    for s, n in rows:
        mod = n.split('::')[1] if n.startswith('diff::') and '::' in n[6:] else n.split('::')[0]
        a, b = by_module.get(mod, (0.0, 0))
        by_module[mod] = (a + s, b + 1)
    mods = ', '.join(f'{m} {s:.0f} s / {c} tests' for m, (s, c) in
                     sorted(by_module.items(), key=lambda kv: -kv[1][0])[:5])
    head = [
        f'# Every rondata release test timed alone, {when}, tools/suite_timings.py (tree {tree}).',
        '# One test per process, --test-threads 1, sequential, RON_INSTALL set; seconds then the test.',
        f'# Sum {total:,.0f} s over {len(rows)} tests{wall_note}. Top 10 = {top(10):.0f} s, '
        f'top 30 = {top(30):.0f}, top 100 = {top(100):.0f}; {under} tests under 1 s.',
        f'# By module: {mods}.',
    ]
    return '\n'.join(head + [f'{s:.2f}\t{n}' for s, n in rows]) + '\n'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('install', nargs='?', help="the game's install, for RON_INSTALL")
    ap.add_argument('--out', type=Path, help='the timings file to write (default: stdout)')
    ap.add_argument('--stale', action='store_true',
                    help="list the tests the lane gate's timings file has never timed, and exit")
    ap.add_argument('--timings', type=Path, default=None,
                    help="the file --stale reads (default: release_gate.py's TIMINGS)")
    args = ap.parse_args(argv)
    import release_gate  # noqa: E402  (the gate's own TIMINGS path)
    timings = args.timings or release_gate.TIMINGS
    binaries = test_binaries()
    names = []
    for b in binaries:
        ignored = set(list_tests(b, ignored=True))
        names += [(b, n) for n in list_tests(b) if n not in ignored]
    if args.stale:
        known = timed(timings.read_text())
        stale = sorted(n for _, n in names if n not in known)
        gone = sorted(n for n in known if n not in {n for _, n in names})
        print(f'{len(stale)} of {len(names)} release tests are untimed by {timings.relative_to(ROOT)}; '
              f'{len(gone)} timed tests are gone')
        for n in stale:
            print(f'  untimed  {n}')
        for n in gone:
            print(f'  gone     {n}')
        return 1 if stale else 0
    if not args.install:
        ap.error('an install path is needed to run the tests')
    env = dict(os.environ, RON_INSTALL=str(Path(args.install).resolve()))
    tree = subprocess.run(['git', 'rev-parse', '--short=8', 'HEAD'], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()
    rows, red = [], []
    t0 = time.monotonic()
    for i, (b, n) in enumerate(names, 1):
        secs, code = time_one(b, n, env)
        rows.append((secs, n))
        if code != 0:
            red.append(n)
        print(f'[{i}/{len(names)}] {secs:8.2f}  {n}{"  RED" if code else ""}', file=sys.stderr, flush=True)
    wall = time.monotonic() - t0
    text = render(rows, date.today().isoformat(), tree, f'; the run {wall / 60:.0f} min wall')
    if args.out:
        args.out.write_text(text)
        print(f'wrote {args.out} ({len(rows)} tests, {sum(s for s, _ in rows):,.0f} s)')
    else:
        sys.stdout.write(text)
    if red:
        print(f'{len(red)} test(s) red when run alone: ' + ', '.join(red), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
