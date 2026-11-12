"""`tools/disk.py`: the booking gate's two disk steps (the steering pass of
2026-11-12, after the disk filled to 0 GB during a builder's test run).

The builders step removes a merged builder's `target/` and nothing else; it
is tested on a fixture repository made with real `git`, never on this one.
The oracle step is tested by its command line only, against a fake `cargo`
that records its arguments: no real cache is deleted by a test.

Red first: there was no `disk.py`; and with the first-parent test left out
of the rule, (b) the fresh builder and (e) the builder cut from an earlier
tip lose their builds; with the ancestor test left out, (c) the builder in
flight does; with a rule that asks whether the tip forks straight off the
line, (d) the builder that merged its base is kept.
"""
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import disk  # noqa: E402


def git(d, *args):
    return subprocess.run(['git', '-C', str(d), *args], check=True, capture_output=True, text=True).stdout.strip()


def commit(d, name, text):
    (Path(d) / f'{name}.txt').write_text(text)
    git(d, 'add', f'{name}.txt')
    git(d, 'commit', '-q', '-m', name)
    return git(d, 'rev-parse', 'HEAD')


def built(tree):
    """A `target/` holding a file, as a cargo build leaves one."""
    (tree / 'target' / 'release').mkdir(parents=True)
    (tree / 'target' / 'release' / 'rondata').write_bytes(b'\0' * 65536)
    return tree / 'target'


class Fixture:
    """An integration branch `main` and its worktrees under
    `.claude/worktrees`:

    (a) `agent-merged`: one commit, merged with `--no-ff`;
    (b) `agent-fresh`: cut on the booked tip, no commit of its own;
    (c) `agent-flight`: one commit, unmerged;
    (d) `agent-based`: one commit, then the base merged into it, then
        merged with `--no-ff`;
    (e) `agent-older`: cut on an earlier booked tip, no commit;
    and two trees that are not builders, both on merged commits: `gate`
    (detached at the tip) and `att-1` (a lane's, merged with `--no-ff`)."""

    def __init__(self):
        self.tmp = tempfile.TemporaryDirectory()
        d = self.root = Path(self.tmp.name) / 'repo'
        d.mkdir()
        git(d, 'init', '-q', '-b', 'main')
        git(d, 'config', 'user.email', 't@t')
        git(d, 'config', 'user.name', 't')
        (d / '.gitignore').write_text('target/\n.claude/\n')
        git(d, 'add', '.gitignore')
        commit(d, 'base', 'base\n')
        self.wt = d / '.claude' / 'worktrees'
        self.wt.mkdir(parents=True)
        self.trees = {}
        flight = self.tree('agent-flight')
        commit(flight, 'flight', 'in flight\n')
        based = self.tree('agent-based')
        commit(based, 'based', 'before the base moved\n')
        lane = self.tree('att-1')
        commit(lane, 'lane', 'a lane\n')
        git(d, 'merge', '-q', '--no-ff', '-m', 'merge att-1', 'worktree-att-1')
        self.tree('agent-older')
        merged = self.tree('agent-merged')
        commit(merged, 'merged', 'merged\n')
        git(d, 'merge', '-q', '--no-ff', '-m', 'merge agent-merged', 'worktree-agent-merged')
        git(based, 'merge', '-q', '--no-ff', '-m', 'merge the base into the builder', 'main')
        git(d, 'merge', '-q', '--no-ff', '-m', 'merge agent-based', 'worktree-agent-based')
        self.tip = git(d, 'rev-parse', 'HEAD')
        self.tree('agent-fresh')
        gate = self.wt / 'gate'
        git(d, 'worktree', 'add', '-q', '--detach', str(gate), self.tip)
        self.trees['gate'] = gate
        self.targets = {name: built(path) for name, path in self.trees.items()}

    def tree(self, name):
        path = self.wt / name
        git(self.root, 'worktree', 'add', '-q', '-b', f'worktree-{name}', str(path), 'main')
        self.trees[name] = path
        return path

    def run(self, *extra):
        out = StringIO()
        with redirect_stdout(out):
            code = disk.main(['--sha', self.tip, '--worktrees', str(self.wt), *extra])
        return code, out.getvalue()

    def standing(self):
        return {name: target.is_dir() for name, target in self.targets.items()}


class Builders(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.tmp.cleanup)

    def test_only_merged_builders_lose_their_target(self):
        code, out = self.f.run()
        self.assertEqual(code, 0, out)
        self.assertEqual(self.f.standing(), {
            'agent-merged': False,   # (a)
            'agent-fresh': True,     # (b) on the first-parent line
            'agent-flight': True,    # (c) no ancestor
            'agent-based': False,    # (d) merged, its base merged into it first
            'agent-older': True,     # (e) on the line, an earlier tip
            'att-1': True,           # not a builder, though merged
            'gate': True,            # not a builder, though on the tip
        })
        # The trees themselves, and their checked-out files, stay.
        self.assertTrue((self.f.trees['agent-merged'] / 'merged.txt').is_file())
        lines = {l.split()[1]: l for l in out.splitlines() if l.split()[:1] in (['cleaned'], ['kept'])}
        self.assertEqual(sorted(lines), ['agent-based', 'agent-flight', 'agent-fresh', 'agent-merged', 'agent-older'])
        self.assertIn('freed', lines['agent-merged'])
        self.assertIn('first-parent', lines['agent-fresh'])
        self.assertIn('not merged', lines['agent-flight'])
        self.assertRegex(out, r'(?m)^Disk: \d+ GB free$')

    def test_the_freed_bytes_are_the_allocated_blocks(self):
        target = self.f.targets['agent-merged']
        blocks = sum(p.lstat().st_blocks * 512 for p in target.rglob('*'))
        self.assertEqual(disk.allocated(target), blocks + target.lstat().st_blocks * 512)
        self.assertGreater(disk.allocated(target), 0)

    def test_a_dry_run_removes_nothing_and_says_what_it_would(self):
        code, out = self.f.run('--dry-run')
        self.assertEqual(code, 0, out)
        self.assertTrue(all(self.f.standing().values()))
        self.assertRegex(out, r'(?m)^would clean +agent-merged ')
        self.assertRegex(out, r'(?m)^would clean +agent-based ')
        self.assertNotRegex(out, r'(?m)^cleaned')

    def test_a_target_that_is_a_symlink_is_refused(self):
        outside = Path(self.f.tmp.name) / 'outside'
        (outside / 'keep').mkdir(parents=True)
        (outside / 'keep' / 'file').write_text('not the tree\'s\n')
        target = self.f.targets['agent-merged']
        disk_rm(target)
        target.symlink_to(outside, target_is_directory=True)
        code, out = self.f.run()
        self.assertEqual(code, 0, out)
        self.assertTrue((outside / 'keep' / 'file').is_file())
        self.assertTrue(target.is_symlink())
        self.assertRegex(out, r'(?m)^kept +agent-merged .*symlink')

    def test_a_symlink_inside_target_is_removed_not_followed(self):
        outside = Path(self.f.tmp.name) / 'outside'
        outside.mkdir()
        (outside / 'file').write_text('not the tree\'s\n')
        target = self.f.targets['agent-merged']
        (target / 'link').symlink_to(outside, target_is_directory=True)
        code, out = self.f.run()
        self.assertEqual(code, 0, out)
        self.assertFalse(target.exists() or target.is_symlink())
        self.assertTrue((outside / 'file').is_file())

    def test_a_tree_without_a_target_is_kept_and_says_so(self):
        disk_rm(self.f.targets['agent-merged'])
        code, out = self.f.run()
        self.assertEqual(code, 0, out)
        self.assertRegex(out, r'(?m)^kept +agent-merged .*no target/')


def disk_rm(path):
    """The fixture's own `target/`, removed by the test that made it."""
    for p in sorted(path.rglob('*'), reverse=True):
        p.rmdir() if p.is_dir() and not p.is_symlink() else p.unlink()
    path.rmdir()


class Free(unittest.TestCase):
    def test_under_thirty_gb_is_low_and_names_the_remedy(self):
        self.assertEqual(disk.free_lines(100 * 10**9), ['Disk: 100 GB free'])
        low = disk.free_lines(12 * 10**9)
        self.assertEqual(low[0], 'Disk: 12 GB free')
        self.assertTrue(low[1].startswith('Disk: LOW'))
        self.assertIn('archive_census.py', low[1])


FAKE_CARGO = """#!/bin/sh
{ pwd; for a in "$@"; do echo "$a"; done; } > "$FAKE_CARGO_LOG"
echo "gc-oracle fake: 2 cache(s), 1 stale generation(s) removed; kept g1 and g2"
exit ${FAKE_CARGO_EXIT:-0}
"""


class Oracle(unittest.TestCase):
    """The oracle step's command line, against a fake `cargo`."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = Path(tmp.name)
        bin_ = self.tmp / 'bin'
        bin_.mkdir()
        (bin_ / 'cargo').write_text(FAKE_CARGO)
        (bin_ / 'cargo').chmod(0o755)
        self.home = self.tmp / 'home'
        (self.home / 'ron-data' / 'Logs').mkdir(parents=True)
        self.golden = self.tmp / 'golden'
        self.golden.mkdir()
        self.log = self.tmp / 'cargo.log'
        self.env = dict(os.environ, PATH=f'{bin_}:{os.environ["PATH"]}', HOME=str(self.home),
                        RON_GOLDEN_DIR=str(self.golden), RON_GAMELOG_DIR=str(self.home / 'ron-data' / 'Logs'),
                        FAKE_CARGO_LOG=str(self.log))

    def run_disk(self, *args, **env):
        return subprocess.run([sys.executable, str(ROOT / 'tools' / 'disk.py'), '--gc', '/the/install', *args],
                              env=dict(self.env, **env), capture_output=True, text=True, cwd=self.tmp)

    def recorded(self):
        cwd, *argv = self.log.read_text().splitlines()
        return cwd, argv

    def test_the_default_roots_are_the_golden_archive_and_the_data_root(self):
        p = self.run_disk()
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        cwd, argv = self.recorded()
        self.assertEqual(Path(cwd).resolve(), ROOT.resolve())
        self.assertEqual(argv, ['run', '--release', '-q', '-p', 'rondata', '--', '/the/install',
                                '--gc-oracle', str(self.golden), '--gc-oracle', str(self.home / 'ron-data')])
        self.assertIn('gc-oracle fake: 2 cache(s)', p.stdout)
        self.assertRegex(p.stdout, r'(?m)^Disk: \d+ GB free$')

    def test_a_logs_directory_outside_the_data_root_is_its_own_root(self):
        logs = self.tmp / 'elsewhere-logs'
        logs.mkdir()
        p = self.run_disk(RON_GAMELOG_DIR=str(logs))
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        _, argv = self.recorded()
        self.assertEqual(argv[argv.index('--') + 2:], ['--gc-oracle', str(self.golden),
                                                        '--gc-oracle', str(self.home / 'ron-data'),
                                                        '--gc-oracle', str(logs)])

    def test_dir_replaces_the_defaults_and_a_missing_one_is_skipped(self):
        one = self.tmp / 'one'
        one.mkdir()
        p = self.run_disk('--dir', str(one), '--dir', str(self.tmp / 'missing'))
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        _, argv = self.recorded()
        self.assertEqual(argv[argv.index('--'):], ['--', '/the/install', '--gc-oracle', str(one)])
        self.assertIn('missing: not a directory; skipped', p.stdout)

    def test_a_failing_cargo_is_a_failed_step(self):
        p = self.run_disk(FAKE_CARGO_EXIT='3')
        self.assertNotEqual(p.returncode, 0)
        self.assertIn('Disk: step failed:', p.stdout)
        self.assertRegex(p.stdout, r'(?m)^Disk: \d+ GB free$')


class GateLog(unittest.TestCase):
    """`tools/gate_run.sh` appends the disk steps' lines to the gate's log;
    `tools/gates.py`'s table of that log is the table without them."""

    GATE = """Gate tree: /x/.claude/worktrees/gate at abc
Gate pid: 1 — kill this gate by its pid
Booking gate scoped to abc: 12 timed tests left to the next whole gate
Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)
Admission wait: 4.0 s over 975 leases, longest 0.0 s
Host load: 5.5 at start, 8.2 at end, on 16 cores
Test summary: sim ok 3 passed
"""
    BUILDERS = """cleaned       agent-a1  0123456789  target/ 4.20 GiB freed
kept          agent-a2  0123456789  on abc's first-parent line (fresh, or cut from a booked tip)
Disk: 41 GB free
"""
    ORACLE = """gc-oracle /h/ron-golden: 133 cache(s), 94 stale generation(s) removed; kept g1 and g2
Disk: step failed: cargo exited 1
Disk: 12 GB free
Disk: LOW — under 30 GB
"""

    def test_the_disk_lines_leave_the_table_unchanged(self):
        import gates
        with tempfile.TemporaryDirectory() as d:
            plain, with_disk = Path(d) / 'gate-plain.log', Path(d) / 'gate-disk.log'
            plain.write_text(self.GATE + 'gate exit 0\n')
            with_disk.write_text(self.GATE.splitlines(keepends=True)[0] + self.BUILDERS
                                 + ''.join(self.GATE.splitlines(keepends=True)[1:]) + self.ORACLE + 'gate exit 0\n')
            for f in (plain, with_disk):
                os.utime(f, (1_000_000, 1_000_000))
            a, b = gates.row(plain), gates.row(with_disk)
            for r in (a, b):
                r.pop('name'), r.pop('path')
            self.assertEqual(a, b)
            self.assertEqual(b['verdict'], 'green')
            # The scope is read off the gate's opening lines, which the
            # builders step now stands ahead of.
            self.assertEqual(b['scope'], 'scoped, 12 left')
            self.assertTrue(with_disk.read_text().endswith('gate exit 0\n'))


STUB_GATE = """import os, sys
print('Gate pid: 1 — kill this gate by its pid')
print('Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)')
sys.exit(int(os.environ.get('STUB_GATE_EXIT', '0')))
"""


class GateRun(unittest.TestCase):
    """`tools/gate_run.sh` itself, on a fixture repository with its own `gate`
    tree, a stub `release_gate.py` and a fake `cargo`: the builders step runs
    before the gate, the oracle step after a green one only, and neither
    moves the exit code or the log's last line."""

    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.tmp.cleanup)
        d, tmp = self.f.root, Path(self.f.tmp.name)
        (d / 'tools').mkdir()
        for name in ('gate_run.sh', 'disk.py', 'archive_census.py', 'census.py'):
            (d / 'tools' / name).write_bytes((ROOT / 'tools' / name).read_bytes())
        (d / 'tools' / 'release_gate.py').write_text(STUB_GATE)
        git(d, 'add', 'tools')
        git(d, 'commit', '-q', '-m', 'the tools')
        self.sha = git(d, 'rev-parse', 'HEAD')
        bin_ = tmp / 'bin'
        bin_.mkdir()
        (bin_ / 'cargo').write_text(FAKE_CARGO)
        (bin_ / 'cargo').chmod(0o755)
        home = tmp / 'home'
        (home / 'ron-data').mkdir(parents=True)
        self.cargo_log = tmp / 'cargo.log'
        self.env = dict(os.environ, PATH=f'{bin_}:{os.environ["PATH"]}', HOME=str(home),
                        RON_GOLDEN_DIR=str(home / 'ron-data'), RON_GAMELOG_DIR=str(home / 'ron-data'),
                        RON_INSTALL='/the/install', FAKE_CARGO_LOG=str(self.cargo_log))
        self.log = tmp / 'gate-x.log'

    def gate(self, **env):
        # gate_run.sh puts $HOME/.cargo/bin first on PATH; HOME is the
        # fixture's, which has none, so the fake is the cargo it finds.
        p = subprocess.run(['zsh', str(self.f.root / 'tools' / 'gate_run.sh'), self.sha, str(self.log)],
                           env=dict(self.env, **env), capture_output=True, text=True, cwd=self.f.root)
        return p.returncode, self.log.read_text()

    def test_a_green_gate_runs_both_steps_and_ends_on_its_exit(self):
        code, log = self.gate()
        self.assertEqual(code, 0, log)
        lines = log.splitlines()
        self.assertEqual(lines[-1], 'gate exit 0')
        self.assertTrue(lines[0].startswith('Gate tree: '))
        self.assertRegex(log, r'(?m)^cleaned +agent-merged ')
        self.assertLess(log.index('cleaned '), log.index('Gate pid: '))
        self.assertIn('gc-oracle fake: 2 cache(s)', log)
        self.assertGreater(log.index('gc-oracle fake'), log.index('Gate steps: '))
        self.assertFalse(self.f.targets['agent-merged'].exists())
        self.assertTrue(self.f.targets['agent-fresh'].exists())

    def test_a_red_gate_skips_the_oracle_step(self):
        code, log = self.gate(STUB_GATE_EXIT='3')
        self.assertEqual(code, 3, log)
        self.assertEqual(log.splitlines()[-1], 'gate exit 3')
        self.assertNotIn('gc-oracle', log)
        self.assertFalse(self.cargo_log.exists())

    def test_a_failed_disk_step_leaves_the_verdict(self):
        code, log = self.gate(FAKE_CARGO_EXIT='1')
        self.assertEqual(code, 0, log)
        self.assertEqual(log.splitlines()[-1], 'gate exit 0')
        self.assertIn('Disk: step failed:', log)
        import gates
        self.assertEqual(gates.row(self.log)['verdict'], 'green')


if __name__ == '__main__':
    unittest.main()
