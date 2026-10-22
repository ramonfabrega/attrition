"""Runner failure and restoration tests; all install/profile data is authored."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import json
import os
import subprocess
import sys
import unattended_capture as runner


class RunnerTest(unittest.TestCase):
    def test_a_launch_under_a_background_manager_is_refused_and_names_the_driver(self):
        # Loop 3334 (the forty-fifth pass): a click-free capture launched
        # straight from a session dies in 3.8 s on `nodrv_CreateWindow` and
        # reads as a renderer failure (ORACLE, "The click-free lane needs a
        # window"); 3329 lost two launches to it. `launchctl managername`
        # reads `Aqua` through RonDriver.app and `Background` from a
        # session's tree (measured 2026-10-19, both ways), so the tool
        # refuses before it stages and names `viadriver.sh`.
        why = runner.refused_launch_context('Background')
        self.assertIn('Background', why)
        self.assertIn('viadriver.sh', why)
        self.assertIn('nodrv_CreateWindow', why)
        self.assertIsNone(runner.refused_launch_context('Aqua'))
        # Unknown (not macOS, or launchctl missing): not refused here.
        self.assertIsNone(runner.refused_launch_context(None))
        def run(command, **kwargs):
            self.assertEqual(command, ['launchctl', 'managername'])
            return SimpleNamespace(stdout='Aqua\n', returncode=0)
        self.assertEqual(runner.launchd_manager(run=run), 'Aqua')
        def missing(command, **kwargs):
            raise FileNotFoundError(command[0])
        self.assertIsNone(runner.launchd_manager(run=missing))

    def test_the_pace_line_projects_the_end_frame_against_the_timeout(self):
        # Loop 2874 (2832's line): a golden take with `WORLD=6,CITIES=5` ran
        # at about a frame a second and met the lane's hour at 1219 of 1260
        # with no receipt. The runner says the pace once it has a minute of
        # frames, and says when the endpoint lies past the timeout.
        self.assertIsNone(runner.pace_line(10, 20.0, 1260, 3600), 'under a minute of frames: no estimate')
        self.assertIsNone(runner.pace_line(0, 90.0, 1260, 3600), 'no frame yet: no estimate')
        self.assertEqual(runner.pace_line(61, 61.0, 1260, 3600),
                         'pace: 61 frames in 61 s (1.0/s); --end-frame 1260 at this pace in about 1260 s, --timeout 3600')
        self.assertEqual(runner.pace_line(60, 120.0, 1260, 600),
                         'pace: 60 frames in 120 s (0.5/s); --end-frame 1260 at this pace in about 2520 s, '
                         'which exceeds --timeout 600 — trim the `end:` set or raise the timeout')
        with tempfile.TemporaryDirectory() as d:
            trace = Path(d) / 'rontrace.log'
            self.assertIsNone(runner.trace_frames_so_far(trace), 'no file')
            import struct
            rows = [struct.pack('<8I', 0x544E4F52, 2, 0x400000, 0, 0, 0, 0, 0)]
            for f in range(0, 7):
                rows.append(struct.pack('<8I', 2, f, 0, 0, 0, 0, 0, f))
                rows.append(struct.pack('<8I', 1, f, 9, 9, 9, 9, 9, 9))
            trace.write_bytes(b''.join(rows))
            self.assertEqual(runner.trace_frames_so_far(trace), 6)
            trace.write_bytes(b''.join(rows) + b'\x00' * 7)
            self.assertEqual(runner.trace_frames_so_far(trace), 6, 'a torn tail is read up to its last whole record')

    def test_a_default_timeout_extends_to_the_pace_lines_estimate(self):
        # Loop 3401b (item 3393): run1014's first detail take printed a pace
        # line past the default 180 s and was killed at it, and the leftover
        # output refused a retake under its name. A timeout nobody gave
        # extends itself, once, to the estimate x 1.25 + 60 s; an explicit
        # one is never changed, and none is ever shortened.
        self.assertEqual(runner.DEFAULT_TIMEOUT, 180)
        self.assertEqual(runner.effective_timeout(True, 180, 400), 560, '400 x 1.25 + 60')
        self.assertEqual(runner.effective_timeout(True, 180, 401), 562, 'rounded up, never short')
        self.assertEqual(runner.effective_timeout(False, 180, 400), 180, 'an explicit --timeout stands')
        self.assertEqual(runner.effective_timeout(True, 180, 50), 180, 'a short estimate never shortens it')
        self.assertEqual(runner.effective_timeout(True, 180, None), 180, 'no estimate yet')
        self.assertEqual(runner.pace_estimate(60, 120.0, 1260), 2520)
        self.assertIsNone(runner.pace_estimate(10, 20.0, 1260))
        said = runner.pace_line(60, 120.0, 1260, 3210, raised_from=180)
        self.assertEqual(said, 'pace: 60 frames in 120 s (0.5/s); --end-frame 1260 at this pace in about '
                               '2520 s, --timeout raised to 3210 from the default 180')
        ap = runner.argparse.ArgumentParser()
        ap.add_argument('--timeout', type=int, default=None)
        for argv, want in (([], (180, True)), (['--timeout', '180'], (180, False)),
                           (['--timeout', '900'], (900, False))):
            args = ap.parse_args(argv)
            runner.settle_timeout(args)
            self.assertEqual((args.timeout, args.timeout_defaulted), want, argv)

    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        # The lane lock the runner takes (parked 1234) lives in the prefix;
        # here the prefix is the temporary directory, never `~/wine-ron`.
        env=patch.dict(os.environ,{'RON_WINEPREFIX':str(self.root),'RON_LANE_LOCK':str(self.root/'.lane.lock'),
                                   'RON_WINE_BIN':'/usr/bin/true','RON_LANE_HOLDER':'unattended_capture',
                                   'RON_LANES_ROOT':str(self.root)})
        env.start();self.addCleanup(env.stop)
        os.environ.pop('RON_LANE_TAKEN',None)

    def lane_state(self):
        return subprocess.run(['zsh','-c',f'source {runner.ROOT}/tools/gamelog/winelaunch.sh; ron_lane_state'],
                              capture_output=True,text=True).stdout.strip()

    def test_the_lane_is_held_for_the_runner_through_its_restore(self):
        # Parked 1234: run467's game had exited, the launch line's lock read
        # `stale`, and 1221's long trace launched while the runner's
        # `finally` was restoring the profile. The runner takes the lane for
        # its own pid, so the lane is held while the runner lives — the
        # restore included — its own launches carry the pid and go, and it
        # releases the lane at the end, so a waiter sees `free`.
        self.assertEqual(self.lane_state(),'free')
        with runner.capture_lane(self.root):
            state=self.lane_state()
            # The runner's takes are the pool's (item 1569), and say so.
            self.assertTrue(state.startswith('held by pool unattended_capture since'),state)
            self.assertIn(f'(pid {os.getpid()},',state)
            self.assertEqual(os.environ.get('RON_LANE_TAKEN'),str(os.getpid()))
            log=self.root/'wine.log'
            run=subprocess.run(['zsh','-c',runner.LAUNCH,'test',str(runner.ROOT/'tools/gamelog/winelaunch.sh'),
                                str(log),'/usr/bin/true'],capture_output=True,text=True)
            self.assertEqual(run.returncode,0,run.stderr)
            self.assertEqual((self.root/'.lane.lock').read_text().split('\n')[2],str(os.getpid()))
            other=subprocess.run(['zsh','-c',f'source {runner.ROOT}/tools/gamelog/winelaunch.sh; ron_wine {log}; echo rc=$?'],
                                 env={k:v for k,v in os.environ.items() if k!='RON_LANE_TAKEN'},
                                 capture_output=True,text=True)
            self.assertIn('rc=75',other.stdout,other.stderr)
        self.assertEqual(self.lane_state(),'free')
        self.assertIsNone(os.environ.get('RON_LANE_TAKEN'))

    def test_a_held_lane_refuses_the_runner_before_it_stages(self):
        (self.root/'.lane.lock').write_text(f'{os.getppid()}\nanother lane since now\n')
        with self.assertRaises(BlockingIOError):
            with runner.capture_lane(self.root): pass

    def test_launch_forwards_exact_arguments_with_spaces(self):
        helper=self.root/'launch helper.zsh';log=self.root/'launch args.log'
        helper.write_text('setopt NO_BG_NICE\nron_wine() {\n printf "%s\\n" "$@" > "$1"\n /usr/bin/true &\n RON_WINE_PID=$!\n}\n')
        subprocess.run(['zsh','-c',runner.LAUNCH,'test',str(helper),str(log),
                        '/authored game.exe',*runner.LAUNCH_ARGS],check=True)
        self.assertEqual(log.read_text().splitlines(),
                         [str(log),'/authored game.exe','-automation','+skipIntro'])

    def test_capture_lane_refuses_second_owner_then_releases(self):
        with runner.capture_lane(self.root):
            with self.assertRaises(BlockingIOError):
                with runner.capture_lane(self.root): pass
        with runner.capture_lane(self.root): pass

    def test_receipt_reads_the_call_window_from_the_cfg(self):
        # 649: run157's receipt said `callwin: null` off the arguments while
        # its `rontrace.cfg` carried `callwin=0-1200` and the trace held the
        # calls. The receipt reads the file the tracer reads.
        (self.root/'rontrace.cfg').write_text('cover=0\ncallwin=0-1200\n')
        self.assertEqual(runner.staged_callwin(self.root), [0, 1200])
        (self.root/'rontrace.cfg').write_text('cover=0\n')
        self.assertIsNone(runner.staged_callwin(self.root))
        self.assertIsNone(runner.staged_callwin(self.root/'never-staged'))

    def test_missing_backup_cannot_claim_restoration(self):
        with self.assertRaisesRegex(ValueError,'incomplete settings backup'):
            runner.verify_restored(self.root/'missing',self.root)

    def test_map_edits_validate_before_writing(self):
        profile=self.root/'profile';(profile/'PlayerProfile').mkdir(parents=True)
        path=profile/'PlayerProfile'/'Player.dat'
        original=b'<MAP_STYLE>14</MAP_STYLE>\r\n<MAP_STYLE value="14"/>\r\n<MAP_STYLE value="14"/>\r\n'
        path.write_bytes(original)
        runner.set_map(profile,18)
        self.assertEqual(path.read_bytes(),original.replace(b'14',b'18'))
        path.write_bytes(original.replace(b'<MAP_STYLE>14</MAP_STYLE>',b''))
        before=path.read_bytes()
        with self.assertRaises(ValueError):runner.set_map(profile,18)
        self.assertEqual(path.read_bytes(),before)

    def test_identity_and_closing_are_required(self):
        log=self.root/'game.log'
        log.write_text(' MAP_STYLE 18\nBEGIN FRAME 37\n GameInfo closing\n')
        self.assertEqual(runner.verify_game(log,18,36)['map_style'],18)
        with self.assertRaises(ValueError):runner.verify_game(log,18,36,12345)
        log.write_text(log.read_text()+' (int)seed 12345\n')
        self.assertEqual(runner.verify_game(log,18,36,12345)['seed_observed'],[12345])
        with self.assertRaises(ValueError):runner.verify_game(log,14,36)
        with self.assertRaises(ValueError):runner.verify_game(log,18,37)
        log.write_text('MAP_STYLE 18\n')
        with self.assertRaises(ValueError):runner.verify_game(log,18,36)

    def test_ai_nation_edit_requires_one_computer_slot_and_changes_only_its_tribe(self):
        profile=self.root/'profile';(profile/'PlayerProfile').mkdir(parents=True)
        path=profile/'PlayerProfile'/'Player.dat'
        original=b'<LAST_SLOT1 value="1"/><XPACK_LAST_TRIBE0 value="4"/><XPACK_LAST_TRIBE1 value="24"/>'
        path.write_bytes(original)
        runner.set_ai_tribe(profile,10)
        self.assertEqual(path.read_bytes(),original.replace(b'TRIBE1 value="24"',b'TRIBE1 value="10"'))
        for bad in (original.replace(b'SLOT1 value="1"',b'SLOT1 value="0"'),
                    original+original,original.replace(b'XPACK_LAST_TRIBE1',b'missing')):
            path.write_bytes(bad)
            with self.assertRaises(ValueError):runner.set_ai_tribe(profile,10)
            self.assertEqual(path.read_bytes(),bad)

    def test_a_rise_key_is_written_on_every_take_with_its_default(self):
        # Item 3492 (Loop 3488's rule): `--rise AutoSaveLoad=650` writes the
        # key; a take without it writes the stated default, so a later take
        # on the lane never inherits a 650; a key with no stated default is
        # refused before a byte is written.
        profile=self.root/'profile';profile.mkdir()
        rise=profile/'rise.ini'
        original='[RISE OF NATIONS]\nAllowLogs=1\nAutoSaveLoad=0\nSeed (0 for random)=12345\n'
        rise.write_text(original)
        self.assertEqual(runner.set_rise(profile,['AutoSaveLoad=650']),{'AutoSaveLoad':650})
        self.assertEqual(rise.read_text(),original.replace('AutoSaveLoad=0','AutoSaveLoad=650'))
        self.assertEqual(runner.set_rise(profile,[]),{'AutoSaveLoad':0})
        self.assertEqual(rise.read_text(),original)
        rise.write_text(original.replace('AutoSaveLoad=0','AutoSaveLoad=650'))
        runner.set_rise(profile,[])
        self.assertEqual(rise.read_text(),original)
        for pairs in (['StartReveal=1'],['AllowLogs=0']):
            with self.assertRaises(ValueError):runner.set_rise(profile,pairs)
            self.assertEqual(rise.read_text(),original)
        rise.write_text('[RISE OF NATIONS]\nAllowLogs=1\n')
        with self.assertRaises(ValueError):runner.set_rise(profile,[])

    def test_a_take_removes_only_the_saves_it_added(self):
        # Item 3492: `AutoSaveLoad`'s save lands in the launch's working
        # directory (run1115's `autosaveload.svx`); the runner removes what
        # the take added and names it with its digest.
        output=self.root/'map-14';output.mkdir()
        kept=output/'kept.svx';kept.write_text('x')
        other=output/'rontrace.log'
        before=runner.save_files(output)
        new=output/'autosaveload.svx';new.write_text('y');other.write_text('z')
        removed=runner.remove_new_saves(output,before)
        self.assertEqual([r['path'] for r in removed],[str(new)])
        self.assertEqual(removed[0]['bytes'],1)
        self.assertFalse(new.exists())
        self.assertTrue(kept.exists() and other.exists())
        self.assertEqual(runner.remove_new_saves(output,runner.save_files(output)),[])

    def test_a_bound_key_is_written_into_the_empty_keys_and_nothing_else(self):
        # Item 3442: `--bindkey` writes bindkey.py's element into the lane's
        # staged profile; anything but one empty <KEYS/> or a known action is
        # refused before a byte is written.
        install=self.root/'install';(install/'data').mkdir(parents=True)
        (install/'data'/'playerprofile.xml').write_text(
            '<KEY enum="FORM_E_RIGHT" dependent="0"></KEY><KEY enum="OPTION_FORM"></KEY>')
        profile=self.root/'profile';(profile/'PlayerProfile').mkdir(parents=True)
        path=profile/'PlayerProfile'/'Player.dat'
        original=b'<ROOT><MAP_STYLE value="14"/><KEYS/></ROOT>'
        path.write_bytes(original)
        runner.bind_keys(profile,install,['FORM_E_RIGHT=120','OPTION_FORM=122'])
        self.assertEqual(path.read_bytes(),
            b'<ROOT><MAP_STYLE value="14"/><KEYS>'
            b'<KEY enum="FORM_E_RIGHT" dependent="-1"><INPUT key="120" mouse="0" ctrl="0" shift="0" alt="0"/></KEY>'
            b'<KEY enum="OPTION_FORM" dependent="-1"><INPUT key="122" mouse="0" ctrl="0" shift="0" alt="0"/></KEY>'
            b'</KEYS></ROOT>')
        for bad,pairs in ((original.replace(b'<KEYS/>',b'<KEYS></KEYS>'),['FORM_E_RIGHT=120']),
                          (original,['FORM_E_LEFT=120']),
                          (original,['FORM_E_RIGHT=120','FORM_E_RIGHT=121']),
                          (original,['FORM_E_RIGHT=0'])):
            path.write_bytes(bad)
            with self.assertRaises(ValueError):runner.bind_keys(profile,install,pairs)
            self.assertEqual(path.read_bytes(),bad)
        path.write_bytes(original)
        runner.bind_keys(profile,install,[])
        self.assertEqual(path.read_bytes(),original)

    def test_nation_receipt_reads_players_instead_of_trusting_the_profile(self):
        log=self.root/'game.log'
        good=('  BEGIN PLAYER\n   flags 7\n   tribe 4\n   who 0\n  Player\n'
              '  BEGIN PLAYER\n   flags 1\n   tribe 10\n   who 1\n  AI\n'
              'BEGIN GAME\n MAP_STYLE 18\n (int)seed 12345\n'
              'BEGIN FRAME 37\n GameInfo closing\n')
        log.write_text(good)
        self.assertEqual(runner.verify_game(log,18,36,12345,ai_tribe=10)['players'],
                         [{'who':0,'tribe':4,'flags':7},{'who':1,'tribe':10,'flags':1}])
        for bad in (good.replace('tribe 10','tribe 11'),good.replace('flags 1','flags 7'),
                    good.replace('tribe 4','tribe 5'),good.replace('who 1','who 2')):
            log.write_text(bad)
            with self.assertRaisesRegex(ValueError,'player'):runner.verify_game(log,18,36,12345,ai_tribe=10)

    def test_a_stall_is_no_gamelog_at_all(self):
        # Parked 762: the two stalls on record had no gamelog.txt; a game
        # past the device has bytes there and is the timeout's to judge.
        log=self.root/'gamelog.txt'
        self.assertTrue(runner.stalled_before_frame_zero(log))
        log.write_text('')
        self.assertTrue(runner.stalled_before_frame_zero(log))
        log.write_text(' MAP_STYLE 18\n')
        self.assertFalse(runner.stalled_before_frame_zero(log))

    def test_a_second_stall_gives_up_instead_of_waiting_out_the_timeout(self):
        # Parked 1469: run584's relaunch sat 48 minutes behind a permission
        # prompt; the stall rule relaunched once and then only the timeout
        # ended it. The verdict is pure: within the stall window nothing,
        # then one relaunch, then a give-up, and never a third.
        report={}
        self.assertIsNone(runner.stall_verdict(report,100,300,True))
        self.assertIsNone(runner.stall_verdict(report,400,300,False))
        self.assertIsNone(runner.stall_verdict(report,400,0,True))
        self.assertEqual(runner.stall_verdict(report,400,300,True),'relaunch')
        report['relaunched_after_seconds']=400
        self.assertIsNone(runner.stall_verdict(report,100,300,True))
        self.assertEqual(runner.stall_verdict(report,300,300,True),'give_up')
        report['stalled_twice_after_seconds']=300
        self.assertIsNone(runner.stall_verdict(report,900,300,True))

    def test_a_closing_before_the_endpoint_is_accepted_only_when_allowed(self):
        # run676's game ended itself at 4340 of 24000 (parked 1551): the
        # closing block sits at the last frame plus one, not at the endpoint.
        log=Path(self.root)/"gamelog.txt"
        log.write_text(' MAP_STYLE 7\nBEGIN FRAME 4340\nBEGIN FRAME 4341\n GameInfo closing\n')
        with self.assertRaises(ValueError):runner.verify_game(log,7,24000)
        got=runner.verify_game(log,7,24000,allow_early_end=True)
        self.assertEqual((got['closing_frame'],got['ended_early']),(4341,True))
        # A closing past the endpoint is no early end.
        log.write_text(' MAP_STYLE 7\nBEGIN FRAME 37\n GameInfo closing\n')
        with self.assertRaises(ValueError):runner.verify_game(log,7,35,allow_early_end=True)
        self.assertEqual(runner.verify_game(log,7,36,allow_early_end=True)['ended_early'],False)

    def test_a_groups_capture_must_print_groupdata(self):
        # Parked 735: run210, run215 and run223 asked for `GROUPS` and two
        # of them printed no `GROUPDATA` block, silently; the pool is what
        # the chapter was captured for.
        log=self.root/'game.log'
        log.write_text(' MAP_STYLE 18\nBEGIN FRAME 37\n GameInfo closing\n')
        self.assertEqual(runner.verify_game(log,18,36,detail=['end:GUYS=4'])['groupdata_blocks'],0)
        with self.assertRaises(ValueError):runner.verify_game(log,18,36,detail=['end:GUYS=4,GROUPS=1'])
        log.write_text(' MAP_STYLE 18\nBEGIN FRAME 36\n  BEGIN GROUPDATA\n  END GROUPDATA\nBEGIN FRAME 37\n GameInfo closing\n')
        self.assertEqual(runner.verify_game(log,18,36,detail=['end:GROUPS=1'])['groupdata_blocks'],1)
        # A level of zero asks for nothing.
        log.write_text(' MAP_STYLE 18\nBEGIN FRAME 37\n GameInfo closing\n')
        self.assertEqual(runner.verify_game(log,18,36,detail=['end:GROUPS=0'])['groupdata_blocks'],0)

    def test_groups_with_deaths_at_one_detail_is_refused_before_the_launch(self):
        # Parked 1885 and 1931: run791's first two takes died in the
        # verifier because `GROUPS=1` beside `DEATHS=1` at one section
        # prints no GROUPDATA; the pairing is refused before anything is
        # staged, and the refusal names the parked row.
        self.assertIsNone(runner.refused_detail(None))
        self.assertIsNone(runner.refused_detail(['end:GUYS=4,GROUPS=1']))
        self.assertIsNone(runner.refused_detail(['end:GROUPS=1','start:DEATHS=1']))
        self.assertIsNone(runner.refused_detail(['end:GROUPS=0,DEATHS=1']))
        why=runner.refused_detail(['end:GUYS=4,GROUPS=1,DEATHS=1'])
        self.assertIn('1885',why)
        self.assertIn('GROUPS',why)
        self.assertIn('DEATHS',why)

    def test_a_long_window_with_no_detail_is_refused_before_the_launch(self):
        # Loop 2199 (2107's line): run830 was the long with no `--detail`,
        # so the runner staged the default `end:UNITS=3` on every frame — 37
        # minutes, 717 MB — and the walker could not stand on it; run831
        # became the long. A window of a thousand frames or more names its
        # detail or is refused, naming the long's own line.
        self.assertIsNone(runner.refused_window(None, None))
        self.assertIsNone(runner.refused_window([2283, 2539], None))
        self.assertIsNone(runner.refused_window([0, 24001], ['end:MISC']))
        why = runner.refused_window([0, 24001], None)
        self.assertIn('2199', why)
        self.assertIn('--detail', why)
        self.assertIn('run830', why)

    def test_a_frame_define_without_its_plan_is_refused_before_the_lane(self):
        # Parked 2006, Loop 2054: run810's first take died in the tracer
        # build — `RON_STATE_FRAME` without `RON_STATE_PLAN` — after the lane
        # was taken. Refused up front, naming the parked row and the fix.
        self.assertIsNone(runner.refused_tracer_defs(None))
        self.assertIsNone(runner.refused_tracer_defs([]))
        self.assertIsNone(runner.refused_tracer_defs(['RON_AUTOSTART=1']))
        self.assertIsNone(runner.refused_tracer_defs(['RON_STATE_FRAME=3631','RON_STATE_PLAN="/p/plan.h"']))
        why=runner.refused_tracer_defs(['RON_AUTOSTART=1','RON_STATE_FRAME=3631'])
        self.assertIn('2006',why)
        self.assertIn('RON_STATE_PLAN',why)

    def test_a_take_that_exits_with_no_gamelog_is_retaken_once(self):
        # Parked 1987: run801's first take died on a Wine page fault before
        # frame 0, with no gamelog.txt, and the retake was by hand under
        # parked 1150's rule. One retake, never a second.
        report={}
        self.assertIsNone(runner.exit_verdict(report,False))
        self.assertEqual(runner.exit_verdict(report,True),'retake')
        report['retaken_after_exit_seconds']=12.0
        self.assertIsNone(runner.exit_verdict(report,True))

    def test_a_staged_line_past_the_end_frame_is_one_line_not_a_traceback(self):
        # Parked 1987 (1988's reach): `--end-frame` below a staged line's
        # frame died with a traceback where a one-line hint saves a take.
        cmd=self.root/'chapter.cmd'
        cmd.write_text('100 !quit\n640 @walk 1\n')
        r=subprocess.run([sys.executable,runner.__file__,'-',str(self.root/'out'),'-',
                          '--end-frame','100','--cmd-file',str(cmd)],capture_output=True,text=True)
        self.assertEqual(r.returncode,2,r.stderr)
        self.assertNotIn('Traceback',r.stderr)
        self.assertIn('640',r.stderr)
        self.assertIn('--end-frame',r.stderr)

    def test_starting_groups_do_not_satisfy_a_later_window(self):
        log=self.root/'game.log'
        text=(' MAP_STYLE 18\n BEGIN GROUPDATA\n END GROUPDATA\n'
              'BEGIN FRAME 10\nBEGIN FRAME 11\nBEGIN FRAME 37\n GameInfo closing\n')
        log.write_text(text)
        with self.assertRaisesRegex(ValueError,'GROUPDATA'):
            runner.verify_game(log,18,36,detail=['end:GROUPS=1'],log_window=[10,12])
        text=text.replace('BEGIN FRAME 10\n','BEGIN FRAME 10\n BEGIN GROUPDATA\n END GROUPDATA\n')
        log.write_text(text)
        with self.assertRaisesRegex(ValueError,'GROUPDATA'):
            runner.verify_game(log,18,36,detail=['end:GROUPS=1'],log_window=[10,12])
        log.write_text(text.replace('BEGIN FRAME 11\n','BEGIN FRAME 11\n BEGIN GROUPDATA\n END GROUPDATA\n'))
        self.assertEqual(runner.verify_game(log,18,36,detail=['end:GROUPS=1'],
                                           log_window=[10,12])['groupdata_frames'],[10,11])

    def test_one_map_and_the_staged_knobs_reach_the_receipt(self):
        # The golden record is one map, a late window and a command file; the
        # receipt must say what ran, not what the caller typed.
        output=self.root/'one';output.mkdir()
        (output/'rontrace.cmd').write_text('0 !ai off\n2000 !quit\n')
        args=SimpleNamespace(install=self.root,profile=self.root,end_frame=2000,
                             log_window=[1900,2000],detail=['end:BUILDS=7'],
                             cover='cover=0',cmd_file=Path('chapter.cmd'),ffwd_minute=2,stride=4)
        staged={}
        with patch.object(runner.live_session,'require_closed'), \
             patch.object(runner.live_session,'stage',side_effect=lambda a:staged.update(vars(a))), \
             patch.object(runner,'set_map',side_effect=ValueError('stop here')), \
             patch.object(runner.live_session,'restore'), \
             patch.object(runner,'verify_restored',return_value=5):
            with self.assertRaisesRegex(ValueError,'stop here'):runner.capture(args,output,26)
        self.assertEqual(staged['log_window'],[1900,2000])
        self.assertEqual(staged['detail'],['end:BUILDS=7'])
        self.assertEqual(staged['ffwd_minute'],2)
        self.assertEqual(staged['stride'],4)
        # A caller-supplied ffwd minute must not also trip the blanket one.
        self.assertFalse(staged['fast_forward'])
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertEqual(receipt['map_requested'],26)
        self.assertEqual(receipt['staged']['log_window'],[1900,2000])
        self.assertEqual(receipt['staged']['stride'],4)
        self.assertEqual(receipt['staged']['rontrace.cmd'],['0 !ai off','2000 !quit'])

    def test_the_receipt_stamps_the_launch_and_the_exit_by_the_wall_clock(self):
        # Item 1568: runs 685 and 686 ran on two lanes at once, and their
        # overlap was read off file mtimes — the receipt had seconds and no
        # instant. The verification after the exit fails here, on purpose.
        output=self.root/'stamped';output.mkdir()
        args=SimpleNamespace(install=self.root,profile=self.root,end_frame=36,seed=12345,timeout=60)
        class Game:
            pid=0
            def wait(self,timeout=None):return 0
            def poll(self):return 0
        with patch.object(runner.live_session,'require_closed'), \
             patch.object(runner.live_session,'stage'), \
             patch.object(runner,'set_map'),patch.object(runner,'mute'), \
             patch.object(runner,'set_lobby'),patch.object(runner,'set_ai_tribe'), \
             patch.object(runner.live_session,'key',return_value=''), \
             patch.object(runner,'sha',return_value=''), \
             patch.object(runner.subprocess,'run'), \
             patch.object(runner.subprocess,'Popen',return_value=Game()), \
             patch.object(runner.live_session,'restore'), \
             patch.object(runner,'verify_restored',return_value=5), \
             patch.dict(runner.HELD,{'lane':2,'prefix':'/h/wine-ron-2'}):
            (self.root/'rise.ini').write_text('Seed (0 for random)=1\n')
            with self.assertRaises(Exception):runner.capture(args,output,7)
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertRegex(receipt['launched_at'],r'^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d{4}$')
        self.assertLessEqual(receipt['launched_at'],receipt['exited_at'])
        # And the lane it ran on (item 1569): no caller names one any more.
        self.assertEqual((receipt['lane'],receipt['lane_prefix']),(2,'/h/wine-ron-2'))

    def test_failure_after_stage_still_restores_and_records(self):
        output=self.root/'output';output.mkdir()
        args=SimpleNamespace(install=self.root,profile=self.root,end_frame=36)
        with patch.object(runner.live_session,'require_closed'), \
             patch.object(runner.live_session,'stage'), \
             patch.object(runner,'set_map',side_effect=ValueError('bad profile')), \
             patch.object(runner.live_session,'restore') as restore, \
             patch.object(runner,'verify_restored',return_value=5):
            with self.assertRaisesRegex(ValueError,'bad profile'):runner.capture(args,output,14)
        restore.assert_called_once_with(output)
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertFalse(receipt['success']);self.assertTrue(receipt['settings_restored'])
        self.assertEqual(receipt['restored_files'],5)

    def test_a_refusal_before_anything_is_written_leaves_no_directory(self):
        # Parked 1108: a window past `!quit` + 1 was refused after the output
        # directory was made, and the relaunch was refused by the directory.
        output=self.root/'run';args=SimpleNamespace(output=output,maps=(14,))
        with patch.object(runner,'capture',side_effect=ValueError('log window')):
            with self.assertRaisesRegex(ValueError,'log window'):runner.capture_all(args)
        self.assertFalse(output.exists())

    def test_a_failure_that_wrote_a_receipt_keeps_its_directory(self):
        output=self.root/'run';args=SimpleNamespace(output=output,maps=(14,))
        def wrote(args,out,style):
            out.mkdir();(out/'receipt.json').write_text('{}');raise ValueError('bad profile')
        with patch.object(runner,'capture',side_effect=wrote):
            with self.assertRaisesRegex(ValueError,'bad profile'):runner.capture_all(args)
        self.assertTrue((output/'map-14/receipt.json').is_file())

    def test_live_game_blocks_restoration_and_records_cleanup_failure(self):
        output=self.root/'output';output.mkdir()
        args=SimpleNamespace(install=self.root,profile=self.root,end_frame=36)
        with patch.object(runner.live_session,'require_closed',side_effect=[None,RuntimeError('game remains')]), \
             patch.object(runner.live_session,'stage'), \
             patch.object(runner,'set_map',side_effect=ValueError('bad profile')), \
             patch.object(runner.live_session,'restore') as restore:
            with self.assertRaisesRegex(RuntimeError,'game remains'):runner.capture(args,output,14)
        restore.assert_not_called()
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertFalse(receipt['settings_restored']);self.assertIn('game remains',receipt['cleanup_error'])

    def test_every_backed_up_file_is_verified(self):
        output=self.root/'output';profile=self.root/'profile'
        for root in (output/'settings-backup',profile):
            (root/'PlayerProfile').mkdir(parents=True)
            for name in runner.live_session.NAMES:
                (root/name).write_bytes(b'ini')
            (root/'PlayerProfile'/'Player.dat').write_bytes(b'profile')
        self.assertEqual(runner.verify_restored(output,profile),4)
        (profile/'PlayerProfile'/'Player.dat').write_bytes(b'changed')
        with self.assertRaises(ValueError):runner.verify_restored(output,profile)


if __name__=='__main__':unittest.main()
