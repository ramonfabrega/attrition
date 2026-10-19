#!/usr/bin/env python3
"""reap_agents.py — a lane's cleanup of its subagents' `agent-*` worktrees,
by ownership, never by glob (Loop 3290, the forty-fourth pass).

    python3 tools/reap_agents.py            # the table: what is mine, what is refused
    python3 tools/reap_agents.py --yes      # remove the trees that are mine

Twice in two tranches a lane's closing `git worktree remove` loop over
every `.claude/worktrees/agent-*` removed other lanes' subagent trees —
five of 3238's (3258) and the three live lanes' (3281) — and the frame
row that said "only the ones your subagents named" did not hold. The
ownership test is the lane's own history, which no glob consults: a
subagent tree whose tip is an ancestor of `--mine` (default `HEAD`, the
lane's branch) is the lane's and merged, and goes; one whose tip is not
in that history is another lane's or unmerged, and is refused either
way. A tree with uncommitted changes or a lock is refused too, however
merged: the lock is the harness's or a person's, and the dirt is work.
Exit 0 when nothing was refused, 1 otherwise; nothing is removed without
`--yes`.
"""
import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


def git(root, *args, check=True):
    p = subprocess.run(['git', *args], cwd=root, capture_output=True, text=True)
    if check and p.returncode != 0:
        raise SystemExit(f'git {" ".join(args)}: {p.stderr.strip() or p.stdout.strip()}')
    return p.stdout


@dataclass
class Row:
    path: Path
    head: str
    verdict: str  # 'mine', 'not in <mine>', 'locked', 'dirty: <paths>'


def agent_worktrees(root):
    """`(path, head, locked)` for every `agent-*` worktree of the repository."""
    out = []
    path = head = None
    locked = False
    for line in git(root, 'worktree', 'list', '--porcelain').splitlines() + ['']:
        if line.startswith('worktree '):
            path, head, locked = Path(line[9:]), None, False
        elif line.startswith('HEAD '):
            head = line[5:]
        elif line == 'locked' or line.startswith('locked '):
            locked = True
        elif line == '':
            if path is not None and path.name.startswith('agent-'):
                out.append((path, head, locked))
            path = None
    return out


def survey(root, mine='HEAD'):
    """One row a tree: the lane's (its tip in `mine`'s history, clean,
    unlocked) or why not."""
    rows = []
    for path, head, locked in agent_worktrees(root):
        if locked:
            verdict = 'locked'
        elif subprocess.run(['git', 'merge-base', '--is-ancestor', head, mine], cwd=root,
                            capture_output=True).returncode != 0:
            verdict = f'not in {mine}'
        else:
            dirt = [l[3:] for l in git(path, 'status', '--porcelain').splitlines() if l] if path.is_dir() else []
            verdict = 'mine' if not dirt else 'dirty: ' + ', '.join(dirt[:5])
        rows.append(Row(path, head, verdict))
    return rows


def reap(root, mine='HEAD', dry_run=True, out=sys.stdout):
    """`(removed, refused)` paths; removes the lane's own trees unless `dry_run`."""
    removed, refused = [], []
    for row in survey(root, mine):
        if row.verdict == 'mine':
            if not dry_run:
                git(root, 'worktree', 'remove', str(row.path))
            removed.append(row.path)
            print(f'{"would remove" if dry_run else "removed"}  {row.path.name}  {row.head[:9]}  merged into {mine}',
                  file=out)
        else:
            refused.append(row.path)
            print(f'refused       {row.path.name}  {row.head[:9]}  {row.verdict}', file=out)
    if dry_run and removed:
        print(f'{len(removed)} tree(s) are this lane\'s; `--yes` removes them', file=out)
    return removed, refused


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--root', default='.', help='the lane\'s worktree (default: the current directory)')
    ap.add_argument('--mine', default='HEAD', help='the lane\'s branch; a tree merged into it is the lane\'s')
    ap.add_argument('--yes', action='store_true', help='remove the lane\'s own trees (default: the table alone)')
    a = ap.parse_args(argv)
    root = str(Path(a.root).resolve())
    _, refused = reap(root, a.mine, dry_run=not a.yes)
    return 1 if refused else 0


if __name__ == '__main__':
    sys.exit(main())
