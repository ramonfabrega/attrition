#!/usr/bin/env python3
"""mutate.py — one mutation, scored by its command's exit, on a clean tree.

A mutation is "commit, mutate, run, restore from git, `touch`" (the brief's
build rows 784 and 907). Seventy-eight of them in one tranche were scratch
scripts, and one let its worker edit sources while it ran, so the verdict
was measured on a tree nobody could name (parked 1387, 1391 — the third
reach of the shape, the twenty-third pass).

    python3 tools/mutate.py --edit crates/sim/src/x.rs 'old text' 'new text' \\
        -- cargo test --release -p rondata run470_is_great_sahara
    python3 tools/mutate.py --patch mutation.diff -- cargo test …

What it refuses, each before anything is written:

  * a tree with a tracked modification (`git status --porcelain -uno`
    non-empty) — commit first, so the restore has something to restore to;
  * a mutation that took nothing (`git diff --stat` empty after it);
  * a tree that changed under the command — anything tracked that differs
    from the restore, or the mutated files differing again after the
    restore. Then there is no verdict: exit 2 and the reason.

Otherwise the command's own exit code is this script's, the restored
files are `touch`ed so cargo rebuilds, and the last line is one of

    mutation: held — <command> exited N       (the walk caught it)
    mutation: failed nothing — <command> exited 0
"""
import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


class Refused(Exception):
    pass


def git(root, *args, check=True):
    p = subprocess.run(['git', *args], cwd=root, capture_output=True, text=True)
    if check and p.returncode != 0:
        raise Refused(f'git {" ".join(args)}: {p.stderr.strip() or p.stdout.strip()}')
    return p.stdout


def dirty(root):
    """Tracked paths that differ from HEAD, one per line."""
    return [l[3:] for l in git(root, 'status', '--porcelain', '-uno').splitlines() if l]


def apply_edit(root, path, old, new, nth=None):
    """Replace one occurrence of `old`: the only one, or the `nth` (from one,
    Loop 2784 — an ambiguous match cost a lane three refusals and a rerun)."""
    p = Path(root) / path
    text = p.read_text()
    n = text.count(old)
    if nth is None:
        if n != 1:
            raise Refused(f'{path}: the old text occurs {n} times, not once; --nth N takes the Nth')
        nth = 1
    elif not 1 <= nth <= n:
        raise Refused(f'{path}: --nth {nth} but the old text occurs {n} time(s)')
    at = -1
    for _ in range(nth):
        at = text.index(old, at + 1)
    p.write_text(text[:at] + new + text[at + len(old):])
    return [path]


def apply_patch(root, patch):
    patch = Path(patch).resolve()
    files = git(root, 'apply', '--numstat', str(patch)).split()
    paths = files[2::3]
    git(root, 'apply', str(patch))
    return paths


def restore(root, paths):
    git(root, 'checkout', '--', *paths)
    now = time.time()
    for p in paths:
        os.utime(Path(root) / p, (now, now))


def run(root, mutate, command, out=sys.stdout):
    """`mutate(root) -> [paths]`; returns the command's exit, or raises Refused."""
    before = dirty(root)
    if before:
        raise Refused('the tree is dirty; commit first: ' + ', '.join(before))
    paths = mutate(root)
    stat = git(root, 'diff', '--stat').strip()
    if not stat:
        restore(root, paths)
        raise Refused('the mutation took nothing: `git diff --stat` is empty')
    print(stat, file=out)
    print(f'mutation applied to {", ".join(paths)}; running: {" ".join(command)}', file=out, flush=True)
    applied = git(root, 'diff')
    try:
        # No bytecode is written under the run: a `.pyc` records its
        # source's mtime to the second and its size, and a mutation of
        # the same length restored inside that second leaves one that
        # still matches — the next run of the restored source is the
        # mutant's (the twenty-fourth pass, on `tools/lanewait.py`).
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
        code = subprocess.run(command, cwd=root, env=env).returncode
    finally:
        # The whole diff, not the paths: an edit to the mutated file itself
        # under the run would vanish in the restore.
        changed = git(root, 'diff') != applied
        touched = sorted(set(dirty(root)) - set(paths))
        restore(root, paths)
        after = dirty(root)
    if changed or after:
        raise Refused('the tree changed under the run, so there is no verdict: '
                      + ', '.join(touched + after or paths))
    verdict = f'held — {command[0]} exited {code}' if code else f'failed nothing — {command[0]} exited 0'
    print(f'mutation: {verdict}', file=out)
    return code


def read_batch(path, sep):
    """The breaks of a `--batch` file: one a line, `FILE<sep>OLD<sep>NEW[<sep>NTH]`,
    blank and `#` lines dropped (Loop 2965: a shell loop over `old|new`
    pairs split a Rust `||` in two and reported four breaks as passed; the
    separator is a tab by default, which no formatted source holds)."""
    out = []
    for n, line in enumerate(Path(path).read_text().splitlines(), 1):
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        parts = line.split(sep)
        if len(parts) not in (3, 4):
            raise Refused(f'{path}:{n}: {len(parts)} field(s) split on {sep!r}; a break is FILE, OLD, NEW and an optional NTH')
        nth = int(parts[3]) if len(parts) == 4 and parts[3].strip() else None
        out.append((parts[0], parts[1], parts[2], nth))
    if not out:
        raise Refused(f'{path}: no break on any line')
    return out


def batch(root, path, sep, command, out=sys.stdout):
    """Every break of the file in turn, each on the restored tree, with
    one verdict a line and a count at the end; 0 only when every break
    held (Loop 2965)."""
    breaks = read_batch(path, sep)
    held = failed = refused = 0
    for k, (file, old, new, nth) in enumerate(breaks, 1):
        try:
            code = run(root, lambda r: apply_edit(r, file, old, new, nth), command, out)
            verdict = 'held' if code else 'failed nothing'
            held += bool(code)
            failed += not code
        except Refused as e:
            verdict = f'refused — {e}'
            refused += 1
        print(f'batch {k}/{len(breaks)} {file} {old!r}: {verdict}', file=out, flush=True)
    print(f'batch: {len(breaks)} break(s) — {held} held, {failed} failed nothing, {refused} refused', file=out)
    return 0 if held == len(breaks) else 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--root', default='.', help='the repository (default: the current directory)')
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument('--edit', nargs=3, metavar=('FILE', 'OLD', 'NEW'),
                   help='replace OLD (exactly once, or the --nth occurrence) with NEW in FILE')
    g.add_argument('--patch', help='a unified diff for `git apply`')
    g.add_argument('--batch', metavar='FILE',
                   help='one break a line, FILE<sep>OLD<sep>NEW[<sep>NTH], each run on the restored tree; '
                        'exit 0 only when every break held (Loop 2965)')
    ap.add_argument('--sep', default='\t', help="--batch: the field separator (default: a tab)")
    ap.add_argument('--nth', type=int, metavar='N', help='with --edit: which occurrence of OLD, from one (Loop 2784)')
    ap.add_argument('command', nargs='+', help='the test command, after `--`')
    a = ap.parse_args(argv)
    if len(a.command) == 1 and ' ' in a.command[0].strip():
        # `-- $T` under zsh hands the whole command as one word (parked
        # 1543): run as a program name it is a traceback, not a verdict.
        print(f'mutate.py refuses: the command arrived as one string ({a.command[0]!r}); '
              'pass it as words after `--`, or `${=T}` under zsh', file=sys.stderr)
        return 2
    root = str(Path(a.root).resolve())
    if a.batch:
        try:
            return batch(root, a.batch, a.sep, a.command)
        except Refused as e:
            print(f'mutate.py refuses: {e}', file=sys.stderr)
            return 2
    if a.edit:
        path, old, new = a.edit
        mutate = lambda r: apply_edit(r, path, old, new, a.nth)  # noqa: E731
    else:
        mutate = lambda r: apply_patch(r, a.patch)  # noqa: E731
    try:
        return run(root, mutate, a.command)
    except Refused as e:
        print(f'mutate.py refuses: {e}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
