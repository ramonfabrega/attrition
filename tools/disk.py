#!/usr/bin/env python3
"""disk.py — the booking gate's two disk steps: a merged builder's build
output, and the decoded-oracle caches no build reads any more.

    python3 tools/disk.py --sha <sha> [--worktrees DIR] [--dry-run]   # the builders step
    python3 tools/disk.py --gc <install> [--dir DIR …]                # the oracle step

On 2026-11-12 the disk filled to 0 GB during a builder's test run. Two
leaks, both regenerable, neither removed by anything: seven merged
builders' `target/` directories held 40.4 GiB, and 94 stale oracle
generations stood over 133 caches. The owner's word that day: both are
cleared as a standing step. `tools/gate_run.sh` runs the builders step
before every booking gate and the oracle step after a green one, so nobody
has to remember.

**The builders step.** For each worktree (`git worktree list --porcelain`)
under `--worktrees` (default the repository's `.claude/worktrees`) whose
directory name starts with `agent-`, its `target/` is removed iff its HEAD

* is an ancestor of `<sha>` — a builder in flight with commits of its own
  is no ancestor, and keeps its build; and
* is not on `<sha>`'s first-parent line (`git rev-list --first-parent`).

A merged builder's tip is reachable from the booked commit, but only
through a merge's second parent (the driver merges with `--no-ff`; a
builder that merged its base into its branch first is still off the
line). A freshly cut builder that has not committed yet sits exactly on a
booked tip — this one, or an earlier one — which is on the first-parent
line, and must not lose its build mid-run. Only `target/` goes: the tree,
its branch and its files stay, and a tree that is not `agent-*` (the
integration tree, the `gate` tree, a ccc lane's `att-*`) is never touched,
merged or not. The harness's own `locked` mark is no refusal here: every
live agent tree carries one, and the rule above is what tells a merged one
from a running one. What the rule cannot see: a builder that keeps working
after its merge without a new commit loses its build at the next gate,
and rebuilds; so does one whose merge the driver backs out after a red.

A `target` that is itself a symlink is refused, and nothing inside one is
followed: the sizes are summed with `lstat` and the removal is
`shutil.rmtree`, which unlinks a symlink it meets rather than entering it.
The freed figure is allocated blocks (`st_blocks * 512`), not apparent size.

**The oracle step.** `cargo run --release -q -p rondata -- <install>
--gc-oracle <dir> …` from this checkout (`rondata::diff::oracle::gc_tree`):
every generation of every `.ron-oracle-v1` cache under each directory but
the running build's own two. The directories default to the golden archive
and the data root as `tools/archive_census.py` resolves them
(`$RON_GOLDEN_DIR` or `~/ron-golden`, and `~/ron-data`), plus the queue
lane's logs (`$RON_GAMELOG_DIR`) when they are under neither. A lane on
another parser fingerprint loses nothing it cannot rebuild: its next read
misses and re-decodes from the dump.

Both steps end with `Disk: <N> GB free` for the volume holding `$HOME`, and
a `Disk: LOW` line naming the remedy under 30 GB. A failure prints `Disk:
step failed: …` and exits 1; the gate's verdict never depends on it.
"""
import argparse
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
LOW_GB = 30  # the steering pass of 2026-11-12: a gate's test run fills ~10-20 GB


def git(cwd, *args, check=True):
    p = subprocess.run(['git', *args], cwd=cwd, capture_output=True, text=True)
    if check and p.returncode != 0:
        raise RuntimeError(f'git {" ".join(args)}: {p.stderr.strip() or p.stdout.strip()}')
    return p


def worktrees(cwd):
    """`(path, head)` for every worktree of the repository holding `cwd`."""
    out, path, head = [], None, None
    for line in git(cwd, 'worktree', 'list', '--porcelain').stdout.splitlines() + ['']:
        if line.startswith('worktree '):
            path, head = Path(line[9:]), None
        elif line.startswith('HEAD '):
            head = line[5:]
        elif line == '' and path is not None:
            out.append((path, head))
            path = None
    return out


def allocated(path):
    """Allocated bytes under `path`, itself included, never through a symlink."""
    total = os.lstat(path).st_blocks * 512
    for dirpath, dirnames, filenames in os.walk(path, followlinks=False):
        for name in dirnames + filenames:
            try:
                total += os.lstat(os.path.join(dirpath, name)).st_blocks * 512
            except FileNotFoundError:
                pass
    return total


def gib(n):
    return f'{n / (1 << 30):.2f} GiB'


def verdict(tree, head, sha, line, repo):
    """`None` when `tree`'s `target/` goes, else why it stays."""
    if head is None:
        return 'no HEAD'
    if head in line:
        return f"on {sha[:10]}'s first-parent line (fresh, or cut from a booked tip)"
    if git(repo, 'merge-base', '--is-ancestor', head, sha, check=False).returncode != 0:
        return f'not merged into {sha[:10]} (in flight)'
    if not tree.is_dir():
        return 'no tree on disk'
    target = tree / 'target'
    try:
        st = os.lstat(target)
    except FileNotFoundError:
        return f'merged into {sha[:10]}, no target/ to remove'
    if stat.S_ISLNK(st.st_mode):
        return 'target is a symlink: refused'
    if not stat.S_ISDIR(st.st_mode):
        return 'target is not a directory: refused'
    if Path(os.path.realpath(target)).parent != Path(os.path.realpath(tree)):
        return 'target resolves outside the tree: refused'
    return None


def builders(sha, wt_dir, dry_run=False, out=None):
    out = out or sys.stdout
    wt_dir = Path(wt_dir).resolve()
    sha = git(wt_dir, 'rev-parse', '--verify', f'{sha}^{{commit}}').stdout.strip()
    line = set(git(wt_dir, 'rev-list', '--first-parent', sha).stdout.split())
    freed = cleaned = 0
    for tree, head in worktrees(wt_dir):
        if not tree.name.startswith('agent-'):
            continue
        try:
            Path(os.path.realpath(tree)).relative_to(wt_dir)
        except ValueError:
            continue
        why = verdict(tree, head, sha, line, wt_dir)
        h = (head or '-')[:10]
        if why is not None:
            print(f'kept          {tree.name}  {h}  {why}', file=out)
            continue
        target = tree / 'target'
        size = allocated(target)
        if not dry_run:
            shutil.rmtree(target)
        freed += size
        cleaned += 1
        print(f'{"would clean " if dry_run else "cleaned     "}  {tree.name}  {h}  target/ {gib(size)} '
              f'{"to free" if dry_run else "freed"} (merged into {sha[:10]})', file=out)
    print(f'Builders: {cleaned} target/ {"would be " if dry_run else ""}removed, {gib(freed)}', file=out)


def oracle_dirs():
    """The golden archive, the data root, and the lane's logs where they are under neither."""
    sys.path.insert(0, str(HERE))
    import archive_census
    roots = [archive_census.golden_root(), archive_census.data_root()]
    logs = os.path.realpath(archive_census.census_mod.ARCHIVE)
    if not any(logs == os.path.realpath(r) or logs.startswith(os.path.realpath(r) + os.sep) for r in roots):
        roots.append(archive_census.census_mod.ARCHIVE)
    return roots


def oracle(install, dirs, out=None):
    out = out or sys.stdout
    present = []
    for d in dirs:
        if os.path.isdir(d):
            present.append(d)
        else:
            print(f'gc-oracle {d}: not a directory; skipped', file=out)
    if not present:
        return
    cmd = ['cargo', 'run', '--release', '-q', '-p', 'rondata', '--', install]
    for d in present:
        cmd += ['--gc-oracle', d]
    p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    for l in p.stdout.splitlines() + p.stderr.splitlines():
        if l.startswith('gc-oracle '):
            print(l, file=out)
    if p.returncode != 0:
        tail = (p.stderr.strip().splitlines() or ['(no output)'])[-1]
        raise RuntimeError(f'cargo exited {p.returncode}: {tail}')


def free_lines(free):
    gb = free // 10**9
    lines = [f'Disk: {gb} GB free']
    if gb < LOW_GB:
        lines.append(f'Disk: LOW — under {LOW_GB} GB: `cargo clean --profile dev` in the integration worktree, '
                     '`python3 tools/archive_census.py` for the archive\'s candidates, '
                     'and `python3 tools/disk.py --sha <tip> --dry-run` for builders kept')
    return lines


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    step = ap.add_mutually_exclusive_group(required=True)
    step.add_argument('--sha', help='the builders step: the gated commit')
    step.add_argument('--gc', metavar='INSTALL', help='the oracle step: the install rondata reads')
    ap.add_argument('--worktrees', help="the builders' directory (default the repository's .claude/worktrees)")
    ap.add_argument('--dry-run', action='store_true', help='the builders step prints and removes nothing')
    ap.add_argument('--dir', action='append', help='the oracle step: a directory to collect (repeatable; '
                    'default the golden archive and the data root)')
    a = ap.parse_args(argv)
    code = 0
    try:
        if a.sha:
            wt = a.worktrees
            if wt is None:
                common = git(os.getcwd(), 'rev-parse', '--path-format=absolute', '--git-common-dir').stdout.strip()
                wt = Path(common).parent / '.claude' / 'worktrees'
            builders(a.sha, wt, a.dry_run)
        else:
            oracle(a.gc, a.dir or oracle_dirs())
    except Exception as e:  # noqa: BLE001 — a disk step never takes the gate down with it
        print(f'Disk: step failed: {e}')
        code = 1
    sys.stdout.flush()
    for l in free_lines(shutil.disk_usage(Path.home()).free):
        print(l)
    return code


if __name__ == '__main__':
    sys.exit(main())
