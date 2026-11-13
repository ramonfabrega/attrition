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


def snapshot(binaries, directory, copy=None):
    """Copies of the test executables under `directory`, to time in place of
    the build's own: a re-time of the whole suite runs ninety minutes one
    test at a time, and a `cargo test` in the same worktree meanwhile writes
    the next build over the very path being timed (item 4216, stage 0). The
    copy keeps the file's name, so two executables never collide."""
    import shutil
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    out = []
    for binary in binaries:
        kept = directory / Path(binary).name
        (copy or shutil.copy2)(binary, kept)
        out.append(str(kept))
    return out


def list_tests(binary, ignored=False):
    args = [binary, '--list', '--format', 'terse'] + (['--ignored'] if ignored else [])
    done = subprocess.run(args, capture_output=True, text=True, check=True)
    return [l[:-len(': test')] for l in done.stdout.splitlines() if l.endswith(': test')]


RSS = re.compile(r'^\s*(\d+)\s+maximum resident set size', re.M)
CLOCK = re.compile(r'^\s*[\d.]+ real\s+([\d.]+) user\s+([\d.]+) sys', re.M)
INSTRUCTIONS = re.compile(r'^\s*(\d+)\s+instructions retired', re.M)


def work(stderr):
    """`(cpu seconds, instructions retired)` from `/usr/bin/time -l`'s
    stderr, `-1` for a line it did not print. Item 4216's baseline: the
    file's seconds are wall and rise with the host's load (a 9,632-frame
    word read 1.76 s alone and 8.20 inside a gate on the same 30 G
    instructions), so the score a change of the suite's form is read
    against is the CPU a test took and the instructions it retired, which
    the load does not move."""
    c, i = CLOCK.search(stderr), INSTRUCTIONS.search(stderr)
    return (float(c.group(1)) + float(c.group(2)) if c else -1.0,
            int(i.group(1)) if i else -1)


def time_one(binary, name, env):
    """Seconds, exit code, peak resident MiB, CPU seconds and instructions
    retired of one test alone (the thirty-fifth steer: a test binary peaked
    at 11.8 GiB under four threads, and whether that is one test or four is
    the memory column's to say; macOS's `/usr/bin/time -l` prints the peak
    in bytes on stderr)."""
    t0 = time.monotonic()
    done = subprocess.run(['/usr/bin/time', '-l', binary, '--exact', name, '--test-threads', '1'],
                          env=env, capture_output=True, text=True, check=False)
    m = RSS.search(done.stderr)
    mib = int(m.group(1)) / (1 << 20) if m else -1.0
    cpu, instructions = work(done.stderr)
    return time.monotonic() - t0, done.returncode, mib, cpu, instructions


def render_cpu(rows, when, tree, load=''):
    """A sibling file, `<stem>-cpu.txt`: CPU seconds (user + system), then
    instructions retired in units of 1e9, then the test, most CPU first; the
    gate reads only the timings file."""
    rows = sorted(rows, key=lambda r: -r[0])
    cpu = sum(c for c, _, _ in rows if c >= 0)
    ins = sum(i for _, i, _ in rows if i >= 0)
    head = [f"# Every rondata release test's CPU alone, {when}, tools/suite_timings.py (tree {tree}).",
            '# /usr/bin/time -l, one test per process, --test-threads 1; CPU seconds (user + sys), '
            'instructions retired (1e9), the test; -1 when unread.',
            f'# Sum {cpu:,.0f} CPU-seconds and {ins / 1e12:,.2f}e12 instructions over {len(rows)} tests{load}.']
    return '\n'.join(head + [f'{c:.2f}\t{i / 1e9:.2f}\t{n}' for c, i, n in rows]) + '\n'


def render_memory(rows, when, tree):
    """A sibling file, `<stem>-memory.txt`: peak resident MiB then the test,
    largest first; the gate reads only the timings file."""
    rows = sorted(rows, key=lambda r: -r[0])
    head = [f'# Every rondata release test\'s peak resident set alone, {when}, tools/suite_timings.py (tree {tree}).',
            '# /usr/bin/time -l, one test per process, --test-threads 1; MiB then the test; -1 when unread.']
    return '\n'.join(head + [f'{m:.0f}\t{n}' for m, n in rows]) + '\n'


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


def stale_only(names, text):
    """The `(binary, name)` pairs the timings file's text has never timed."""
    known = timed(text)
    return [(b, n) for b, n in names if n not in known]


def merged(text, rows, when, tree):
    """The timings file's text with `rows` (`(seconds, name)`) merged under
    the kept ones — a re-timed test's new row wins — rendered whole. The
    thirty-sixth pass: ten tests were untimed one tranche after the suite
    was re-timed whole (849 tests, 59 minutes, the loop held the while), so
    `--stale-only` times the untimed alone and merges them here."""
    kept = timed(text)
    for secs, name in rows:
        kept[name] = secs
    return render([(s, n) for n, s in kept.items()], when, tree, '; --stale-only merged')


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('install', nargs='?', help="the game's install, for RON_INSTALL")
    ap.add_argument('--out', type=Path, help='the timings file to write (default: stdout)')
    ap.add_argument('--stale', action='store_true',
                    help="list the tests the lane gate's timings file has never timed, and exit")
    ap.add_argument('--timings', type=Path, default=None,
                    help="the file --stale reads (default: release_gate.py's TIMINGS)")
    ap.add_argument('--stale-only', action='store_true',
                    help='time only the tests the timings file has never timed, and merge '
                         'them into it (written to --out, or over the file itself)')
    ap.add_argument('--snapshot', type=Path, default=None,
                    help='copy the test executables into this directory and time the copies, '
                         'so a build in this worktree meanwhile cannot replace what is being timed')
    args = ap.parse_args(argv)
    import release_gate  # noqa: E402  (the gate's own TIMINGS path)
    timings = args.timings or release_gate.TIMINGS
    binaries = test_binaries()
    if args.snapshot:
        binaries = snapshot(binaries, args.snapshot)
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
    if args.stale_only:
        names = stale_only(names, timings.read_text())
        print(f'{len(names)} untimed test(s) to time', file=sys.stderr)
    rows, mem, cpu, red = [], [], [], []
    t0 = time.monotonic()
    load0 = os.getloadavg()[0]
    for i, (b, n) in enumerate(names, 1):
        secs, code, mib, used, instructions = time_one(b, n, env)
        rows.append((secs, n))
        mem.append((mib, n))
        cpu.append((used, instructions, n))
        if code != 0:
            red.append(n)
        print(f'[{i}/{len(names)}] {secs:8.2f} {used:8.2f}c {mib:7.0f}M  {n}{"  RED" if code else ""}',
              file=sys.stderr, flush=True)
    wall = time.monotonic() - t0
    load = f"; the host's load {load0:.1f} at the start and {os.getloadavg()[0]:.1f} at the end"
    if args.stale_only:
        text = merged(timings.read_text(), rows, date.today().isoformat(), tree)
    else:
        text = render(rows, date.today().isoformat(), tree, f'; the run {wall / 60:.0f} min wall')
    if args.out or args.stale_only:
        out = args.out or timings
        out.write_text(text)
        mem_out = out.with_name(out.name.replace('-suite-timings', '-suite-memory'))
        if args.stale_only and mem_out.exists():
            old_mem = {n: m for m, n in ((float(a), b) for a, b in
                       (l.split('\t') for l in mem_out.read_text().splitlines() if not l.startswith('#')))}
            for mib, n in mem:
                old_mem[n] = mib
            mem = [(m, n) for n, m in old_mem.items()]
        mem_out.write_text(render_memory(mem, date.today().isoformat(), tree))
        cpu_out = out.with_name(out.name.replace('-suite-timings', '-suite-cpu'))
        if args.stale_only and cpu_out.exists():
            old_cpu = {n: (float(c), float(i) * 1e9) for c, i, n in
                       (l.split('\t') for l in cpu_out.read_text().splitlines() if not l.startswith('#'))}
            for used, instructions, n in cpu:
                old_cpu[n] = (used, instructions)
            cpu = [(c, i, n) for n, (c, i) in old_cpu.items()]
        cpu_out.write_text(render_cpu(cpu, date.today().isoformat(), tree, load))
        print(f'wrote {out} ({len(timed(text))} tests, {sum(timed(text).values()):,.0f} s), {mem_out} and {cpu_out}')
    else:
        sys.stdout.write(text)
    if red:
        print(f'{len(red)} test(s) red when run alone: ' + ', '.join(red), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
