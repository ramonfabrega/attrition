"""`tools/emu_receipts.py`: the sweep cache's receipts, listed and verified.

`crates/sim/src/sweep/support.rs` keys each `emu_rows` run on the SHA-256
of the executable, the script and the `tools/emu/` modules it can reach,
and writes the receipt as the cache file's header. The layout below is the
Rust unit tests' own (`lay_out`), so the pinned key is the same literal on
both sides: a drift in either key text fails one of the two.
"""
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import emu_receipts  # noqa: E402

# `support.rs`'s `the_key_moves_with_every_input_and_not_with_a_path`
# pins the same literal for the same layout.
PINNED_KEY = '6cd8297036e1e4c4fbe9175973c94410c17929c99222fdb7f81ab42c8fe4d2df'


def lay_out(root):
    """The Rust tests' layout: a script importing one sibling sweep, one
    sweep it does not import, two libraries and a non-module."""
    emu = root / 'tools/emu'
    emu.mkdir(parents=True)
    (emu / 'sweep_a.py').write_text(
        'import sys\nfrom hooks import Harness  # noqa: E402\nfrom sweep_b import run\nrun()\n')
    (emu / 'sweep_b.py').write_text("def run():\n    print('1 -> 2')\n")
    (emu / 'sweep_unrelated.py').write_text("print('unrelated')\n")
    (emu / 'hooks.py').write_text('class Harness: pass\n')
    (emu / 'callfn.py').write_text('STACK_BASE = 0\n')
    (emu / 'README.md').write_text('not a module\n')
    exe = root / 'install/riseofnations.exe'
    exe.parent.mkdir(parents=True)
    exe.write_text('MZ the executable')
    return emu, exe


def write_entry(cache, emu, exe, script='sweep_a.py', rows='1 -> 2\n3 -> 4\n'):
    """A cache file as `support.rs`'s `store` writes it."""
    key, exe_sha, script_sha, modules = emu_receipts.key_for(emu, exe, script)
    head = ['# emu-cache v1', f'# key {key}', f'# executable {exe_sha} {exe}',
            f'# script {script_sha} {script}']
    head += [f'# module {sha} {name}' for name, sha in modules]
    head += ['# date 2026-10-19T00:00:00Z', '# uv 0.11.16', '# python 3.14.7 (uv run --no-project)',
             f'# rows {sum(1 for r in rows.splitlines() if r.strip())}', '# end']
    cache.mkdir(parents=True, exist_ok=True)
    path = cache / f'{key}.rows'
    path.write_text('\n'.join(head) + '\n' + rows)
    return path


def run(argv):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = emu_receipts.main(argv)
    return code, out.getvalue()


class Receipts(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.emu, self.exe = lay_out(self.root)
        self.cache = self.root / 'cache'

    def tearDown(self):
        self.tmp.cleanup()

    def args(self, *more):
        return ['--cache', str(self.cache), '--emu-dir', str(self.emu), *more]

    def test_the_key_is_the_rust_pin(self):
        key, *_ = emu_receipts.key_for(self.emu, self.exe, 'sweep_a.py')
        self.assertEqual(key, PINNED_KEY)

    def test_the_modules_are_the_libraries_and_the_imported_sweeps(self):
        self.assertEqual(emu_receipts.shared_modules(self.emu, 'sweep_a.py'),
                         ['callfn.py', 'hooks.py', 'sweep_b.py'])

    def test_a_receipt_lists_as_a_row(self):
        path = write_entry(self.cache, self.emu, self.exe)
        code, out = run(self.args())
        self.assertEqual(code, 0)
        self.assertIn('sweep_a.py', out)
        self.assertIn(path.stem[:12], out)
        self.assertIn(' 2 ', out, 'the row count')

    def test_verify_passes_a_fresh_entry(self):
        write_entry(self.cache, self.emu, self.exe)
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 0, out)

    def test_verify_names_a_stale_module(self):
        path = write_entry(self.cache, self.emu, self.exe)
        (self.emu / 'hooks.py').write_text('class Harness: x = 1\n')
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 1)
        self.assertIn(path.name, out)
        self.assertIn('hooks.py', out)

    def test_verify_names_a_changed_or_missing_executable(self):
        path = write_entry(self.cache, self.emu, self.exe)
        self.exe.write_text('MZ another executable')
        code, out = run(self.args('--verify'))
        self.assertEqual((code, path.name in out, 'executable' in out), (1, True, True), out)
        self.exe.unlink()
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 1)
        self.assertIn('missing', out)

    def test_verify_names_a_new_library(self):
        write_entry(self.cache, self.emu, self.exe)
        (self.emu / 'newlib.py').write_text('X = 1\n')
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 1)
        self.assertIn('newlib.py', out)

    def test_verify_names_a_header_that_is_not_its_own_key(self):
        path = write_entry(self.cache, self.emu, self.exe)
        text = path.read_text().replace('# script ', '# script 0', 1)
        path.write_text(text)
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 1)
        self.assertIn('key', out)

    def test_an_empty_cache_is_said(self):
        code, out = run(self.args('--verify'))
        self.assertEqual(code, 0)
        self.assertIn('no receipts', out)


if __name__ == '__main__':
    unittest.main()
