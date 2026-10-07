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


def apply_edit(root, path, old, new):
    p = Path(root) / path
    text = p.read_text()
    n = text.count(old)
    if n != 1:
        raise Refused(f'{path}: the old text occurs {n} times, not once')
    p.write_text(text.replace(old, new, 1))
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


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--root', default='.', help='the repository (default: the current directory)')
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument('--edit', nargs=3, metavar=('FILE', 'OLD', 'NEW'),
                   help='replace OLD (exactly once) with NEW in FILE')
    g.add_argument('--patch', help='a unified diff for `git apply`')
    ap.add_argument('command', nargs='+', help='the test command, after `--`')
    a = ap.parse_args(argv)
    if len(a.command) == 1 and ' ' in a.command[0].strip():
        # `-- $T` under zsh hands the whole command as one word (parked
        # 1543): run as a program name it is a traceback, not a verdict.
        print(f'mutate.py refuses: the command arrived as one string ({a.command[0]!r}); '
              'pass it as words after `--`, or `${=T}` under zsh', file=sys.stderr)
        return 2
    root = str(Path(a.root).resolve())
    if a.edit:
        path, old, new = a.edit
        mutate = lambda r: apply_edit(r, path, old, new)  # noqa: E731
    else:
        mutate = lambda r: apply_patch(r, a.patch)  # noqa: E731
    try:
        return run(root, mutate, a.command)
    except Refused as e:
        print(f'mutate.py refuses: {e}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
