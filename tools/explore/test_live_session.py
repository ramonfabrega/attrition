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
            'rise.ini': b'InitialDump=1\r\ncheck_all_level=0\r\n',
            'rise2.ini': b'LogStartFrame=4\r\nLogEndFrame=8\r\n',
            'gamelog.ini': (b'[Logging Options]\nDUMP_ALL=1\nLogFile=old\nDumpFileName=old\n'
                            b'[End Frame]\nUNITS=0\nBUILDS=9\nLEADERS=9\n'
                            b'[Start Game]\nWORLD=6\n[Misc Logging]\nCOMMANDMANAGER=0\nCHECKSUM=0\n'),
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

    def test_full_start_is_bounded_and_restores_the_profile(self):
        self.args.dump_all_start = True
        module.stage(self.args)
        self.assertIn('InitialDump=1', (self.profile/'rise.ini').read_text())
        self.assertIn('DUMP_ALL=1', (self.profile/'gamelog.ini').read_text())
        self.assertIn('WORLD=6', (self.profile/'gamelog.ini').read_text())
        self.assertIn('CHECKSUM=2', (self.profile/'gamelog.ini').read_text())
        self.assertIn('check_all_level=14', (self.profile/'rise.ini').read_text())
        self.assertIn('LogStartFrame=0', (self.profile/'rise2.ini').read_text())
        self.assertIn('LogEndFrame=2', (self.profile/'rise2.ini').read_text())
        module.restore(self.output)
        self.assert_restored()

    def test_full_start_refuses_a_long_or_late_window_before_writing(self):
        self.args.dump_all_start = True
        for window in ([0, 36], [1, 2]):
            self.args.log_window = window
            with self.assertRaisesRegex(ValueError, 'full start'):
                module.stage(self.args)
            self.assertFalse(self.output.exists())
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

    def test_caller_supplies_the_window_and_the_detail(self):
        self.args.log_window = [9000, 9300]
        self.args.detail = ['end:BUILDS=7', 'start:WORLD=6']
        self.args.end_frame = 9400
        self.args.ffwd_minute = 10
        module.stage(self.args)
        rise2 = (self.profile / 'rise2.ini').read_text()
        self.assertIn('LogStartFrame=9000', rise2)
        self.assertIn('LogEndFrame=9300', rise2)
        log = (self.profile / 'gamelog.ini').read_text()
        # Asked for, and everything else in the same section flattened to 0.
        self.assertIn('BUILDS=7', log)
        self.assertIn('WORLD=6', log)
        self.assertIn('LEADERS=0', log)
        self.assertIn('UNITS=0', log)
        self.assertIn('COMMANDMANAGER=0', log)
        self.assertEqual((self.output / 'rontrace.cmd').read_text(),
                         '37 !ffwd 10\n9400 !quit\n')
        module.restore(self.output)
        self.assert_restored()

    def test_a_category_the_ini_does_not_have_is_refused(self):
        # The failure this catches is a window that dumps nothing: a mistyped
        # category writes no key, and the run comes back with empty blocks.
        self.args.detail = ['end:UNIT=3']
        with self.assertRaisesRegex(ValueError, 'no such categories'):
            module.stage(self.args)
        self.assertFalse(self.output.exists())
        self.assert_restored()
        self.args.detail = ['fin:UNITS=3']
        with self.assertRaisesRegex(ValueError, 'unknown detail section'):
            module.stage(self.args)
        self.assert_restored()

    def test_the_command_file_is_staged_in_frame_order_around_the_ffwd(self):
        script = self.root / 'chapter.cmd'
        script.write_text('# a comment\n0 !ai off\n\n1200 age who=0 5\n1300 war\n')
        self.args.cmd_file = script
        self.args.end_frame = 2000
        self.args.ffwd_minute = 2
        module.stage(self.args)
        self.assertEqual((self.output / 'rontrace.cmd').read_text(),
                         '0 !ai off\n37 !ffwd 2\n1200 age who=0 5\n1300 war\n2000 !quit\n')
        module.restore(self.output)
        self.assert_restored()

    def test_a_window_or_a_command_past_the_quit_frame_is_refused(self):
        for attribute, value, pattern in [
                ('log_window', [18, 500], 'log window'),
                ('log_window', [500, 18], 'log window'),
                ('ffwd_minute', 0, 'ffwd-minute'),
        ]:
            setattr(self.args, attribute, value)
            with self.assertRaisesRegex(ValueError, pattern):
                module.stage(self.args)
            self.assertFalse(self.output.exists())
            delattr(self.args, attribute)
        script = self.root / 'bad.cmd'
        self.args.cmd_file = script
        for text, pattern in [('90 war\n', 'past the !quit frame'),
                              ('20 war\n10 peace\n', 'below the previous'),
                              ('war\n', 'want `<sim-frame> <text>`')]:
            script.write_text(text)
            with self.assertRaisesRegex(ValueError, pattern):
                module.stage(self.args)
            self.assertFalse(self.output.exists())
        self.assert_restored()

    def test_output_cannot_live_inside_install(self):
        self.args.output = self.install / 'nested'
        with self.assertRaises(ValueError):
            module.stage(self.args)
        self.assertFalse(self.args.output.exists())
        self.assert_restored()


if __name__ == '__main__':
    unittest.main()
