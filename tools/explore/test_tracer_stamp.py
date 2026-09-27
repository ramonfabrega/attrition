"""The queue lane says which tracer build it ran (parked 936).

`build.sh` writes `rontrace.dll` into the install and the queue lane runs
whatever is there: run314's first take ran a build four days older than the
verbs its stanza used, and nothing on either side said so (item 923). Four
later landings recorded the DLL's sha256 by hand. `tools/trace/stamp.py`
writes a stamp beside the DLL at build time — the source's hash, the
defines, the DLL's hash — and checks it at capture time against the DLL
that is there and the `tracer.c` of the tree that is asking.
"""
import importlib.util
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('stamp', ROOT / 'tools/trace/stamp.py')
stamp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stamp)


class TracerStamp(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.install = Path(self.tmp.name) / 'install'
        self.install.mkdir()
        self.source = Path(self.tmp.name) / 'tracer.c'
        self.source.write_text('/* the verbs as of the build */\n')
        (self.install / 'rontrace.dll').write_bytes(b'MZ built from the source above')

    def tearDown(self):
        self.tmp.cleanup()

    def check(self, issuer=False):
        return stamp.check(self.install, self.source, issuer=issuer)

    def test_a_build_and_its_capture_agree(self):
        stamp.write(self.install, self.source, defs='-DRON_AUTOSTART=1')
        ok, lines = self.check(issuer=True)
        self.assertTrue(ok, lines)
        text = '\n'.join(lines)
        self.assertIn('defs -DRON_AUTOSTART=1', text)
        self.assertIn(stamp.sha256(self.install / 'rontrace.dll'), text)

    def test_a_dll_with_no_stamp_is_named_and_refuses_an_issuer_stanza(self):
        ok, lines = self.check()
        self.assertTrue(ok, 'a stanza with no `@` line runs on any build, and says which')
        self.assertIn('no stamp', '\n'.join(lines))
        ok, lines = self.check(issuer=True)
        self.assertFalse(ok)
        self.assertIn('rebuild', '\n'.join(lines))

    def test_a_dll_swapped_after_its_stamp_is_caught(self):
        stamp.write(self.install, self.source, defs='')
        (self.install / 'rontrace.dll').write_bytes(b'MZ another build, copied in by hand')
        ok, lines = self.check()
        self.assertFalse(ok)
        self.assertIn('not the build its stamp describes', '\n'.join(lines))

    def test_a_build_older_than_the_tree_s_verbs_refuses_an_issuer_stanza(self):
        # run314's first take.
        stamp.write(self.install, self.source, defs='')
        self.source.write_text('/* the verbs as of the build, and @gatherpoint */\n')
        ok, lines = self.check()
        self.assertTrue(ok, 'without an `@` line the difference is said, not refused')
        self.assertIn('differs from this tree', '\n'.join(lines))
        ok, lines = self.check(issuer=True)
        self.assertFalse(ok)

    def test_the_build_writes_the_stamp_and_the_capture_reads_it(self):
        self.assertIn('stamp.py write', (ROOT / 'tools/trace/build.sh').read_text())
        capture = (ROOT / 'tools/gamelog/longtrace.sh').read_text()
        call = 'tools/trace/stamp.py" check "$G"'
        self.assertIn(call, capture)
        self.assertLess(capture.index(call), capture.index('# --- stage.'),
                        'the check runs before anything is staged, so a refusal leaves nothing to restore')


if __name__ == '__main__':
    unittest.main()
