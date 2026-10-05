"""`tools/mutate.py`: one mutation on a clean tree, with teeth (parked
1391, the twenty-third pass).

The fixture is a git repository with one source file and a "walk" that
reads it: the walk fails when the source is mutated and passes when it is
not. The refusals are the shapes the tranches met — a dirty tree, a
mutation that took nothing, a source edited while the command ran.
"""
import os
import subprocess
import sys
import tempfile
import unittest
from io import StringIO
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import mutate  # noqa: E402

# The walk: exit 0 when the source still says `let x = 1;`.
WALK = [sys.executable, '-c',
        'import sys; sys.exit(0 if "let x = 1;" in open("src.rs").read() else 1)']
# A command that edits the source while it runs.
EDITOR = [sys.executable, '-c', 'open("src.rs", "a").write("// edited under the run\\n")']


def repo():
    d = tempfile.mkdtemp()
    subprocess.run(['git', 'init', '-q', d], check=True)
    subprocess.run(['git', '-C', d, 'config', 'user.email', 't@t'], check=True)
    subprocess.run(['git', '-C', d, 'config', 'user.name', 't'], check=True)
    (Path(d) / 'src.rs').write_text('fn f() {\n    let x = 1;\n}\n')
    subprocess.run(['git', '-C', d, 'add', '.'], check=True)
    subprocess.run(['git', '-C', d, 'commit', '-q', '-m', 'base'], check=True)
    return d


class Mutate(unittest.TestCase):
    def setUp(self):
        self.d = repo()
        self.out = StringIO()

    def edit(self, old='let x = 1;', new='let x = 2;'):
        return lambda r: mutate.apply_edit(r, 'src.rs', old, new)

    def test_a_mutation_the_walk_holds_restores_the_tree_and_reports_the_exit(self):
        code = mutate.run(self.d, self.edit(), WALK, out=self.out)
        self.assertEqual(code, 1)
        self.assertIn('mutation: held', self.out.getvalue())
        self.assertIn('let x = 1;', (Path(self.d) / 'src.rs').read_text())
        self.assertEqual(mutate.dirty(self.d), [])

    def test_a_mutation_that_fails_nothing_says_so(self):
        # An edit the walk does not read is the "failed nothing" shape.
        code = mutate.run(self.d, self.edit('}\n', '}\n// comment\n'), WALK, out=self.out)
        self.assertEqual(code, 0)
        self.assertIn('mutation: failed nothing', self.out.getvalue())

    def test_a_dirty_tree_is_refused_before_anything_is_written(self):
        (Path(self.d) / 'src.rs').write_text('fn f() {}\n')
        with self.assertRaises(mutate.Refused) as cm:
            mutate.run(self.d, self.edit(), WALK, out=self.out)
        self.assertIn('dirty', str(cm.exception))
        self.assertEqual((Path(self.d) / 'src.rs').read_text(), 'fn f() {}\n')

    def test_a_mutation_that_took_nothing_is_refused(self):
        with self.assertRaises(mutate.Refused) as cm:
            mutate.run(self.d, self.edit('let x = 1;', 'let x = 1;'), WALK, out=self.out)
        self.assertIn('took nothing', str(cm.exception))

    def test_a_mutated_python_source_leaves_no_bytecode_behind(self):
        # The twenty-fourth pass's own: a mutation of the same length,
        # restored inside the second it was written in, left a `.pyc`
        # whose recorded mtime and size still matched the restored
        # source — and the next, unmutated run failed on the mutant.
        d = Path(self.d)
        (d / 'mod.py').write_text('X = 1\n')
        (d / '.gitignore').write_text('__pycache__/\n')
        subprocess.run(['git', '-C', self.d, 'add', '.'], check=True)
        subprocess.run(['git', '-C', self.d, 'commit', '-q', '-m', 'a module'], check=True)
        walk = [sys.executable, '-c', 'import mod, sys; sys.exit(0 if mod.X == 1 else 1)']
        env = {k: v for k, v in os.environ.items() if k != 'PYTHONDONTWRITEBYTECODE'}
        code = mutate.run(self.d, lambda r: mutate.apply_edit(r, 'mod.py', 'X = 1', 'X = 2'), walk, out=self.out)
        self.assertEqual(code, 1)
        again = subprocess.run(walk, cwd=self.d, env=env).returncode
        self.assertEqual(again, 0, 'the restored source ran as the mutant: stale bytecode')

    def test_a_tree_that_changed_under_the_run_has_no_verdict(self):
        with self.assertRaises(mutate.Refused) as cm:
            mutate.run(self.d, self.edit(), EDITOR, out=self.out)
        self.assertIn('changed under the run', str(cm.exception))
        self.assertNotIn('mutation:', self.out.getvalue())
        # The mutated file is restored even so.
        self.assertIn('let x = 1;', (Path(self.d) / 'src.rs').read_text())

    def test_the_restored_file_is_touched_so_cargo_rebuilds(self):
        p = Path(self.d) / 'src.rs'
        old = p.stat().st_mtime - 100
        os.utime(p, (old, old))
        mutate.run(self.d, self.edit(), WALK, out=self.out)
        self.assertGreater(p.stat().st_mtime, old + 50)

    def test_the_command_line_scores_by_the_command_s_exit(self):
        code = mutate.main(['--root', self.d, '--edit', 'src.rs', 'let x = 1;', 'let x = 2;',
                            '--', *WALK])
        self.assertEqual(code, 1)
        code = mutate.main(['--root', self.d, '--edit', 'src.rs', 'nothing here', 'x', '--', *WALK])
        self.assertEqual(code, 2)


if __name__ == '__main__':
    unittest.main()
