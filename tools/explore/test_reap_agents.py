"""`tools/reap_agents.py`: a lane's cleanup of its subagents' `agent-*`
worktrees by ownership, never by glob (Loop 3290, the forty-fourth pass).

Twice in two tranches a lane's closing `git worktree remove` loop over
every `.claude/worktrees/agent-*` removed other lanes' subagent trees —
five of 3238's (3258) and the three live lanes' (3281) — and the frame
row that said "only the ones your subagents named" did not hold. The
ownership test is the lane's own history: a subagent tree whose tip is an
ancestor of the lane's branch is the lane's and merged; one whose tip is
not is another lane's, or unmerged, and is refused either way.
"""
import subprocess
import os
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import reap_agents  # noqa: E402


def git(d, *args):
    return subprocess.run(['git', '-C', str(d), *args], check=True, capture_output=True, text=True).stdout


def repo():
    d = Path(tempfile.mkdtemp())
    git(d, 'init', '-q', '-b', 'main')
    git(d, 'config', 'user.email', 't@t')
    git(d, 'config', 'user.name', 't')
    (d / 'a.txt').write_text('base\n')
    git(d, 'add', '.')
    git(d, 'commit', '-q', '-m', 'base')
    git(d, 'checkout', '-q', '-b', 'worktree-att-1')
    return d


def agent(d, name, text, merge_into=None):
    """An `agent-<name>` worktree with one commit; merged into the lane's
    branch when `merge_into` is given."""
    path = d / '.claude' / 'worktrees' / f'agent-{name}'
    git(d, 'worktree', 'add', '-q', '-b', f'agent-{name}', str(path), 'main')
    (path / f'{name}.txt').write_text(text)
    git(path, 'add', '.')
    git(path, 'commit', '-q', '-m', name)
    if merge_into:
        git(d, 'merge', '-q', '--no-ff', '-m', f'merge {name}', f'agent-{name}')
    return path


class Reap(unittest.TestCase):
    def test_a_merged_tree_is_the_lane_s_and_an_unmerged_one_is_refused(self):
        d = repo()
        mine = agent(d, 'mine', 'merged\n', merge_into='worktree-att-1')
        other = agent(d, 'other', 'another lane\n')
        rows = reap_agents.survey(d, 'HEAD')
        self.assertEqual({r.path.name: r.verdict for r in rows},
                         {'agent-mine': 'mine', 'agent-other': 'not in HEAD'})
        removed, refused = reap_agents.reap(d, 'HEAD', dry_run=False)
        self.assertEqual([p.name for p in removed], ['agent-mine'])
        self.assertEqual([p.name for p in refused], ['agent-other'])
        self.assertFalse(mine.exists())
        self.assertTrue(other.exists())
        self.assertIn('agent-other', git(d, 'worktree', 'list'))

    def test_a_dirty_or_locked_tree_is_refused_even_when_merged(self):
        d = repo()
        dirty = agent(d, 'dirty', 'merged\n', merge_into='worktree-att-1')
        (dirty / 'scratch.txt').write_text('uncommitted\n')
        locked = agent(d, 'locked', 'merged\n', merge_into='worktree-att-1')
        git(d, 'worktree', 'lock', str(locked))
        removed, refused = reap_agents.reap(d, 'HEAD', dry_run=False)
        self.assertEqual(removed, [])
        self.assertEqual(sorted(p.name for p in refused), ['agent-dirty', 'agent-locked'])
        self.assertTrue(dirty.exists() and locked.exists())

    def test_a_lock_whose_holder_is_dead_is_stale_and_the_tree_goes(self):
        # The pass of 2026-11-15: seven merged builder trees stood locked
        # `claude agent agent-… (pid 62375 start …)` after the driver row
        # that cut them was reaped. The harness's lock names its process; a
        # lock whose process is gone protects nothing. A lock with a live
        # holder, or with no pid in its reason (a person's), still refuses.
        d = repo()
        stale = agent(d, 'stale', 'merged\n', merge_into='worktree-att-1')
        git(d, 'worktree', 'lock', '--reason', 'claude agent agent-stale (pid 999999 start Sat Nov 14 17:58:58 2026)', str(stale))
        live = agent(d, 'live', 'merged\n', merge_into='worktree-att-1')
        git(d, 'worktree', 'lock', '--reason', f'claude agent agent-live (pid {os.getpid()} start now)', str(live))
        person = agent(d, 'person', 'merged\n', merge_into='worktree-att-1')
        git(d, 'worktree', 'lock', '--reason', 'kept for a look', str(person))
        removed, refused = reap_agents.reap(d, 'HEAD', dry_run=False)
        self.assertEqual([p.name for p in removed], ['agent-stale'])
        self.assertEqual(sorted(p.name for p in refused), ['agent-live', 'agent-person'])
        self.assertFalse(stale.exists())

    def test_a_dry_run_removes_nothing_and_the_command_line_needs_yes(self):
        d = repo()
        mine = agent(d, 'mine', 'merged\n', merge_into='worktree-att-1')
        removed, _ = reap_agents.reap(d, 'HEAD', dry_run=True)
        self.assertEqual([p.name for p in removed], ['agent-mine'])
        self.assertTrue(mine.exists())
        out = subprocess.run([sys.executable, str(ROOT / 'tools/reap_agents.py'), '--root', str(d)],
                             capture_output=True, text=True)
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertIn('agent-mine', out.stdout)
        self.assertIn('--yes', out.stdout)
        self.assertTrue(mine.exists())


if __name__ == '__main__':
    unittest.main()
