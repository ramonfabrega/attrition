"""Safety checks for temporary live-probe staging; never uses a real install."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('live_session', Path(__file__).with_name('live_session.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class SessionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.install = self.root / 'install'
        self.profile = self.root / 'profile'
        self.output = self.root / 'output'
        self.install.mkdir()
        self.profile.mkdir()
        (self.profile / 'PlayerProfile').mkdir()
        (self.profile / 'PlayerProfile' / 'player').write_bytes(b'profile')
        (self.install / 'riseofnations.exe').write_bytes(b'fixture only')
        (self.install / 'Data').mkdir()
        self.original = {
            'rise.ini': b'InitialDump=1\r\n',
            'rise2.ini': b'LogStartFrame=4\r\nLogEndFrame=8\r\n',
            'gamelog.ini': b'[Logging Options]\nDUMP_ALL=1\nLogFile=old\nDumpFileName=old\n[End Frame]\nUNITS=0\n',
        }
        for name, data in self.original.items():
            (self.profile / name).write_bytes(data)
        self.args = SimpleNamespace(install=self.install, profile=self.profile,
                                    output=self.output, hide_scene=False)

    def assert_restored(self):
        for name, data in self.original.items():
            self.assertEqual((self.profile / name).read_bytes(), data)
        self.assertEqual((self.install / 'riseofnations.exe').read_bytes(), b'fixture only')
        self.assertEqual((self.profile / 'PlayerProfile' / 'player').read_bytes(), b'profile')

    def test_long_run_fast_forwards_after_the_dump_window(self):
        self.args.end_frame = 8000
        self.args.fast_forward = True
        module.stage(self.args)
        self.assertEqual((self.output/'rontrace.cmd').read_text(), '37 !ffwd 9\n8000 !quit\n')
        self.assertEqual((self.output/'rontrace.cfg').read_text(), 'cover=0\ncallwin=0-8000\n')
        self.assertIn('LogEndFrame=36', (self.profile/'rise2.ini').read_text())
        module.restore(self.output)
        self.assert_restored()

    def test_invalid_long_run_never_changes_settings(self):
        for end, fast, hide in [(35, False, False), (24001, False, False),
                                 (36, True, False), (37, True, False), (8000, True, True)]:
            self.args.end_frame, self.args.fast_forward, self.args.hide_scene = end, fast, hide
            with self.assertRaises(ValueError):
                module.stage(self.args)
            self.assertFalse(self.output.exists())
            self.assert_restored()

    def test_restore_preserves_original_bytes_and_keeps_outputs(self):
        module.stage(self.args)
        (self.output / 'rontrace.log').write_bytes(b'capture')
        module.restore(self.output)
        self.assert_restored()
        self.assertEqual((self.output / 'rontrace.log').read_bytes(), b'capture')
        with self.assertRaises(FileExistsError):
            module.stage(self.args)
        self.assert_restored()

    def test_same_key_in_another_section_is_not_ambiguous(self):
        self.original['gamelog.ini'] += b'DumpFileName=other-section\n'
        (self.profile / 'gamelog.ini').write_bytes(self.original['gamelog.ini'])
        module.stage(self.args)
        text = (self.profile / 'gamelog.ini').read_text()
        self.assertIn('DumpFileName=other-section', text)
        self.assertIn('\\dumplog.txt', text)
        module.restore(self.output)
        self.assert_restored()

    def test_partial_shared_write_is_rolled_back(self):
        write = Path.write_text
        def fail_second(path, text, *args, **kwargs):
            if path.resolve() == (self.profile / 'rise2.ini').resolve():
                raise OSError('injected second-write failure')
            return write(path, text, *args, **kwargs)
        with patch.object(Path, 'write_text', fail_second):
            with self.assertRaisesRegex(OSError, 'second-write'):
                module.stage(self.args)
        self.assert_restored()

    def test_missing_key_does_not_stage_or_touch_shared_state(self):
        (self.profile / 'rise2.ini').write_bytes(b'bad input')
        with self.assertRaises(ValueError):
            module.stage(self.args)
        self.assertFalse(self.output.exists())
        self.assertEqual((self.profile / 'rise.ini').read_bytes(), self.original['rise.ini'])

    def test_output_cannot_live_inside_install(self):
        self.args.output = self.install / 'nested'
        with self.assertRaises(ValueError):
            module.stage(self.args)
        self.assertFalse(self.args.output.exists())
        self.assert_restored()


if __name__ == '__main__':
    unittest.main()
