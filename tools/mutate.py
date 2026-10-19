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
    mutation: hung — killed after N s          (counted as caught; Loop 3276)
    mutation: no verdict — the command did not compile   (Loop 3300)

A mutant past `--timeout` (a batch's default: five times the un-mutated
run, at least a minute) is killed with its process group and counted as
caught — a mutant of `find_ordered_collision` that never terminated held
one `cargo test` for 100 minutes (Loop 3276). A run whose output says the
command did not compile (`error: could not compile`, `error[E…]`) is no
verdict and counted apart: `cargo test` exits 101 for a type error as it
does for a red walk, and a lane read one as a catch (Loop 3300). A signal
under the run (`TaskStop`, SIGTERM, SIGHUP, ^C) restores the tree first.
"""
import argparse
import os
import signal
import subprocess
import sys
import time
from pathlib import Path


class Refused(Exception):
    pass


class Hung(Exception):
    """The command ran past the timeout; killed, the tree restored."""


class NoVerdict(Exception):
    """The command did not compile, so its exit scores nothing."""


COMPILE_FAILED = ('error: could not compile', 'error[E')


def timeout_for(unmutated_seconds):
    """A batch's per-mutant timeout: five times the un-mutated run, at
    least a minute (Loop 3276)."""
    return max(60.0, 5.0 * unmutated_seconds)


def run_command(command, root, timeout=None, out=sys.stdout):
    """The command in its own process group, its output copied to `out`
    when it ends; `(code, output)`, or `Hung` after `timeout` seconds."""
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    proc = subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        output, _ = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        proc.communicate()
        raise Hung(f'killed after {timeout:g} s')
    out.write(output)
    out.flush()
    return proc.returncode, output


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


def run(root, mutate, command, out=sys.stdout, timeout=None):
    """`mutate(root) -> [paths]`; returns the command's exit, or raises
    Refused, Hung (past `timeout`) or NoVerdict (it did not compile). The
    tree is restored on every path out, a signal's included."""
    before = dirty(root)
    if before:
        raise Refused('the tree is dirty; commit first: ' + ', '.join(before))
    paths = []
    applied = ''
    output = ''
    try:
        # The `try` opens before the mutation is written: a signal between
        # the write and a later `try` left the mutant in the tree under
        # load (item 3304: `test_a_signal_under_the_run_restores_the_tree`
        # failed one run in three while two `git` calls sat in the window).
        paths = mutate(root)
        stat = git(root, 'diff', '--stat').strip()
        if not stat:
            raise Refused('the mutation took nothing: `git diff --stat` is empty')
        print(stat, file=out)
        print(f'mutation applied to {", ".join(paths)}; running: {" ".join(command)}', file=out, flush=True)
        applied = git(root, 'diff')
        # No bytecode is written under the run: a `.pyc` records its
        # source's mtime to the second and its size, and a mutation of
        # the same length restored inside that second leaves one that
        # still matches — the next run of the restored source is the
        # mutant's (the twenty-fourth pass, on `tools/lanewait.py`).
        code, output = run_command(command, root, timeout=timeout, out=out)
    finally:
        # The whole diff, not the paths: an edit to the mutated file itself
        # under the run would vanish in the restore.
        changed = bool(applied) and git(root, 'diff') != applied
        touched = sorted(set(dirty(root)) - set(paths))
        restore(root, paths)
        after = dirty(root)
    if changed or after:
        raise Refused('the tree changed under the run, so there is no verdict: '
                      + ', '.join(touched + after or paths))
    if code and any(mark in output for mark in COMPILE_FAILED):
        raise NoVerdict(f'the command did not compile ({command[0]} exited {code}); a mutant that '
                        'does not type-check scores nothing (Loop 3300)')
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
        # A break whose OLD spans two lines cannot sit on one line of the
        # file, so `\\n` in OLD and NEW is a newline (Loop 3048: a two-line
        # break was refused line by line, and 3058 dropped one for it).
        out.append((parts[0], parts[1].replace('\\n', '\n'), parts[2].replace('\\n', '\n'), nth))
    if not out:
        raise Refused(f'{path}: no break on any line')
    return out


def batch(root, path, sep, command, out=sys.stdout, timeout=None):
    """Every break of the file in turn, each on the restored tree, with
    one verdict a line and a count at the end; 0 only when every break
    held (Loop 2965) — a hung mutant counts as held (Loop 3276), one that
    did not compile as no verdict (Loop 3300). `timeout` is seconds a
    mutant; None takes `timeout_for` the un-mutated run."""
    breaks = read_batch(path, sep)
    # The un-mutated command once, first (Loop 3037): a port that does not
    # compile exits 101 as a red walk does, and fifteen "held" breaks of
    # one batch were a type error. A tree the command does not pass has
    # no verdict to give.
    before = dirty(root)
    if before:
        raise Refused('the tree is dirty; commit first: ' + ', '.join(before))
    print(f'batch: the un-mutated tree first; running: {" ".join(command)}', file=out, flush=True)
    began = time.monotonic()
    code, _ = run_command(command, root, timeout=timeout, out=out)
    took = time.monotonic() - began
    if code:
        raise Refused(f'the command exits {code} on the un-mutated tree, so no break can be held by it; '
                      'make it pass first (Loop 3037)')
    if timeout is None:
        timeout = timeout_for(took)
    print(f'batch: the un-mutated run took {took:.1f} s; a mutant is killed after {timeout:g} s (Loop 3276)',
          file=out, flush=True)
    held = failed = refused = hung = noverdict = 0
    for k, (file, old, new, nth) in enumerate(breaks, 1):
        try:
            code = run(root, lambda r: apply_edit(r, file, old, new, nth), command, out, timeout=timeout)
            verdict = 'held' if code else 'failed nothing'
            held += bool(code)
            failed += not code
        except Hung as e:
            verdict = f'hung — {e}; counted as held'
            held += 1
            hung += 1
        except NoVerdict as e:
            verdict = f'no verdict — {e}'
            noverdict += 1
        except Refused as e:
            verdict = f'refused — {e}'
            refused += 1
        print(f'batch {k}/{len(breaks)} {file} {old!r}: {verdict}', file=out, flush=True)
    print(f'batch: {len(breaks)} break(s) — {held} held' + (f' ({hung} hung)' if hung else '')
          + f', {failed} failed nothing, {refused} refused' + (f', {noverdict} no verdict' if noverdict else ''),
          file=out)
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
    ap.add_argument('--timeout', type=float, metavar='SECONDS',
                    help='kill a mutant after this long and count it caught (Loop 3276); '
                         "a batch's default is five times the un-mutated run, at least 60 s")
    ap.add_argument('command', nargs='+', help='the test command, after `--`')
    a = ap.parse_args(argv)
    if len(a.command) == 1 and ' ' in a.command[0].strip():
        # `-- $T` under zsh hands the whole command as one word (parked
        # 1543): run as a program name it is a traceback, not a verdict.
        print(f'mutate.py refuses: the command arrived as one string ({a.command[0]!r}); '
              'pass it as words after `--`, or `${=T}` under zsh', file=sys.stderr)
        return 2
    root = str(Path(a.root).resolve())

    # A signal under the run (`TaskStop`, SIGTERM, SIGHUP, ^C) becomes an
    # exception, so `run`'s restore runs before the exit (Loop 3300: a
    # stopped `mutate.py` left the mutation in the tree).
    def interrupted(signum, _frame):
        raise KeyboardInterrupt(signal.Signals(signum).name)
    for sig in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
        signal.signal(sig, interrupted)
    try:
        return _main(a, root)
    except KeyboardInterrupt as e:
        print(f'mutate.py: {e}; the mutation is restored and there is no verdict', file=sys.stderr)
        return 130


def _main(a, root):
    if a.batch:
        try:
            return batch(root, a.batch, a.sep, a.command, timeout=a.timeout)
        except Refused as e:
            print(f'mutate.py refuses: {e}', file=sys.stderr)
            return 2
    if a.edit:
        path, old, new = a.edit
        mutate = lambda r: apply_edit(r, path, old, new, a.nth)  # noqa: E731
    else:
        mutate = lambda r: apply_patch(r, a.patch)  # noqa: E731
    try:
        return run(root, mutate, a.command, timeout=a.timeout)
    except Hung as e:
        print(f'mutation: hung — {e}', file=sys.stderr)
        return 124
    except NoVerdict as e:
        print(f'mutation: no verdict — {e}', file=sys.stderr)
        return 2
    except Refused as e:
        print(f'mutate.py refuses: {e}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
