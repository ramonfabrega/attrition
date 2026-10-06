"""Runner failure and restoration tests; all install/profile data is authored."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import json
import os
import subprocess
import unattended_capture as runner


class RunnerTest(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        # The lane lock the runner takes (parked 1234) lives in the prefix;
        # here the prefix is the temporary directory, never `~/wine-ron`.
        env=patch.dict(os.environ,{'RON_WINEPREFIX':str(self.root),'RON_LANE_LOCK':str(self.root/'.lane.lock'),
                                   'RON_WINE_BIN':'/usr/bin/true','RON_LANE_HOLDER':'unattended_capture'})
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
            self.assertTrue(state.startswith('held by unattended_capture since'),state)
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
                             cover='cover=0',cmd_file=Path('chapter.cmd'),ffwd_minute=2)
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
        # A caller-supplied ffwd minute must not also trip the blanket one.
        self.assertFalse(staged['fast_forward'])
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertEqual(receipt['map_requested'],26)
        self.assertEqual(receipt['staged']['log_window'],[1900,2000])
        self.assertEqual(receipt['staged']['rontrace.cmd'],['0 !ai off','2000 !quit'])

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
