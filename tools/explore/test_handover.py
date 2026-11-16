"""tools/handover.py's rules, on authored rosters and a repository made here: no ccc, no row."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import handover  # noqa: E402

TREE = '/repo/.claude/worktrees/replan-pdb'


def row(name, ref, state='done', cwd=TREE, archived=False):
    return {'archived': archived, 'job': {'tempo': 'idle'},
            'session': {'name': name, 'id': ref, 'state': state, 'cwd': cwd, 'pid': 1}}


class CheckTests(unittest.TestCase):
    def test_one_driver_row_passes(self):
        ok, line = handover.check_line([row('steer', 'aaaa0001', 'working')], TREE)
        self.assertTrue(ok)
        self.assertIn('steer aaaa0001', line)

    def test_two_driver_rows_fail_and_name_both(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('commander', 'bbbb0002')]
        ok, line = handover.check_line(rows, TREE)
        self.assertFalse(ok)
        self.assertIn('steer aaaa0001', line)
        self.assertIn('commander bbbb0002', line)
        self.assertIn('handover.py reap', line)

    def test_a_stopped_or_archived_row_cannot_wake_and_is_not_counted(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('commander', 'bbbb0002', 'stopped'),
                row('commander', 'cccc0003', archived=True)]
        self.assertTrue(handover.check_line(rows, TREE)[0])

    def test_a_lane_or_another_tree_s_row_is_not_a_driver_row(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('lane-4200', 'bbbb0002', 'working'),
                row('commander', 'cccc0003', cwd='/elsewhere')]
        self.assertTrue(handover.check_line(rows, TREE)[0])

    def test_no_row_passes(self):
        self.assertTrue(handover.check_line([], TREE)[0])


class StrayTests(unittest.TestCase):
    TABLE = {
        10: (1, 'claude --bg'),                      # the row
        20: (10, '/bin/zsh -c python3 tools/handover.py to commander'),  # this call's shell
        30: (20, 'python3 tools/handover.py to commander'),
        35: (30, 'ps -Ao pid=,ppid=,command='),      # this script's own child
        40: (10, '/bin/zsh -c zsh tools/gate_run.sh abc gate.log'),
        50: (40, 'python3 tools/release_gate.py'),
        60: (1, 'unrelated'),
        70: (10, 'bun /somewhere/server.ts'),        # an MCP server of the harness
        80: (10, 'caffeinate -i -t 300'),
    }

    def test_this_call_and_the_harness_s_own_children_are_not_strays(self):
        table = {k: v for k, v in self.TABLE.items() if k not in (40, 50)}
        self.assertEqual(handover.strays(10, table, 30), [])

    def test_a_gate_under_the_row_is_named(self):
        found = handover.strays(10, self.TABLE, 30)
        self.assertEqual([pid for pid, _ in found], [40])
        self.assertIn('gate_run.sh', found[0][1])


class DriverLineTests(unittest.TestCase):
    """The pass of 2026-11-15 (DECISIONS 135). The tranche before it booked
    fifty landings without a pass against a backstop of twenty, wrote
    "Twenty landings are booked today" at 12:16 and went on; its driver
    crossed 400 k at 13:10 and ended the day at 536 k without naming the
    line. Both rules were prose. These are what the gate and the brief ask."""

    def test_a_brief_is_refused_at_twenty_landings_since_the_pass(self):
        ok, line = handover.brief_line(19)
        self.assertTrue(ok)
        ok, line = handover.brief_line(20)
        self.assertFalse(ok)
        self.assertIn('20 landings since the last pass', line)
        self.assertIn('handover.py to steer', line)

    def test_a_scoped_gate_is_refused_past_what_the_seats_held(self):
        ok, line = handover.landings_line(19, scoped=True)
        self.assertTrue(ok)
        self.assertNotIn('backstop reached', line)
        ok, line = handover.landings_line(24, scoped=True)   # twenty, and the four seats
        self.assertTrue(ok)
        self.assertIn('backstop reached', line)
        ok, line = handover.landings_line(25, scoped=True)
        self.assertFalse(ok)
        self.assertIn('25 landings since the last pass', line)
        # The way out is the drain's own gate: whole, then the handover.
        ok, line = handover.landings_line(50, scoped=False)
        self.assertTrue(ok)
        self.assertIn('backstop reached', line)

    def test_a_count_that_cannot_be_read_says_so_and_refuses_nothing(self):
        ok, line = handover.landings_line(None, scoped=True)
        self.assertTrue(ok)
        self.assertIn('not read', line)
        self.assertTrue(handover.brief_line(None)[0])

    def test_a_waiver_is_printed_with_its_reason(self):
        ok, line = handover.landings_line(31, scoped=True, waiver="Fable's week reads 91 %")
        self.assertTrue(ok)
        self.assertIn("waived: Fable's week reads 91 %", line)
        self.assertTrue(handover.brief_line(31, waiver="Fable's week reads 91 %")[0])

    def test_the_driver_s_depth_warns_at_the_line_and_refuses_at_the_ceiling(self):
        ok, line = handover.depth_line(399_000)
        self.assertTrue(ok)
        self.assertNotIn('clear', line)
        ok, line = handover.depth_line(412_000)
        self.assertTrue(ok)
        self.assertIn('412 k', line)
        self.assertIn('bank on this gate and clear', line)
        ok, line = handover.depth_line(451_000)
        self.assertFalse(ok)
        self.assertIn('--ungated', line)
        ok, line = handover.depth_line(None)
        self.assertTrue(ok)
        self.assertIn('not read', line)

    def test_the_wake_s_opener_is_the_queue_s_own_paragraph(self):
        queue = ('## Where things stand\n*dated*\nScoreboard: x\n\n'
                 '**Opener: continue the tranche (`commander`, Opus 5.5)** — check the gate;\n'
                 'cut the seats.\n\n## The queue\n\n4159. An item.\n')
        self.assertEqual(handover.queue_opener(queue),
                         '**Opener: continue the tranche (`commander`, Opus 5.5)** — check the gate;\ncut the seats.')
        with self.assertRaises(ValueError):
            handover.queue_opener('## Where things stand\nno opener here\n')
        with self.assertRaises(ValueError):
            handover.queue_opener(queue + '\n**Opener: a second one**\n')

    def test_an_ungated_handover_tells_the_new_row_to_gate_first(self):
        text = handover.ungated_opener('abc123def', 'continue the tranche')
        self.assertTrue(text.startswith('The tip `abc123def` is committed and neither gated nor pushed'))
        self.assertIn('tools/gate_run.sh abc123def', text)
        self.assertTrue(text.endswith('continue the tranche'))


class PathTests(unittest.TestCase):
    def test_the_tool_finds_ccc_and_lore_from_a_bare_shell(self):
        # A cron's wake runs `handover.py wake` in a row's shell as it
        # stands, and a driver row's Bash has no Homebrew or bun on its PATH
        # until a call exports them: from `env -i`, `check` died on
        # `FileNotFoundError: 'ccc'` (the pass of 2026-11-15). The tool adds
        # the directories itself, once, ahead of what is there.
        path = handover.tool_path('/usr/bin:/bin', '/Users/x')
        self.assertEqual(path, '/opt/homebrew/bin:/Users/x/.local/bin:/Users/x/.bun/bin:/Users/x/.cargo/bin:/usr/bin:/bin')
        self.assertEqual(handover.tool_path(path, '/Users/x'), path)


class LandingsTests(unittest.TestCase):
    """The count's zero is the commit that added the newest pass record, and
    a landing is a first-parent merge after it: on a repository made here."""

    def git(self, *argv):
        out = subprocess.run(['git', *argv], cwd=self.dir, capture_output=True, text=True,
                             env={**os.environ, 'GIT_AUTHOR_NAME': 't', 'GIT_AUTHOR_EMAIL': 't@t',
                                  'GIT_COMMITTER_NAME': 't', 'GIT_COMMITTER_EMAIL': 't@t'})
        self.assertEqual(out.returncode, 0, out.stderr)
        return out.stdout.strip()

    def commit(self, path, text='x'):
        full = Path(self.dir) / path
        full.parent.mkdir(parents=True, exist_ok=True)
        full.write_text(text)
        self.git('add', path)
        self.git('commit', '-q', '-m', f'write {path}')

    def land(self, n):
        self.git('checkout', '-q', '-b', f'lane-{n}')
        self.commit(f'work-{n}.txt')
        self.git('checkout', '-q', 'trunk')
        self.git('merge', '-q', '--no-ff', '-m', f'Merge lane-{n}', f'lane-{n}')

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = self.tmp.name
        self.git('init', '-q', '-b', 'trunk')
        self.commit('README.md')

    def tearDown(self):
        self.tmp.cleanup()

    def test_no_pass_record_is_no_count(self):
        self.land(1)
        self.assertEqual(handover.landings_since_pass(self.dir), (None, None))

    def test_landings_are_the_first_parent_merges_since_the_record_was_added(self):
        self.land(1)
        self.commit('docs/audit/2026-01-01-fable-pass-first.md')
        self.land(2)
        self.land(3)
        self.commit('docs/audit/2026-01-01-fable-pass-first.md', 'amended: not a new pass')
        count, sha = handover.landings_since_pass(self.dir)
        self.assertEqual(count, 2)
        self.commit('docs/audit/2026-01-02-fable-pass-second.md')
        self.assertEqual(handover.landings_since_pass(self.dir)[0], 0)
        self.land(4)
        self.assertEqual(handover.landings_since_pass(self.dir)[0], 1)
        # A gate is asked about the commit it gates, not the caller's HEAD.
        self.assertEqual(handover.landings_since_pass(self.dir, tip='HEAD~1')[0], 0)


if __name__ == '__main__':
    unittest.main()
