#!/usr/bin/env python3
"""Acquire a bounded two-map capture without clicks (Wine/display still required).

All captures and install-derived artifacts stay in a fresh output directory.
One process per map; no same-process state reset is assumed.
"""
import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time
from types import SimpleNamespace

import live_session
from autostart_receipt import receipt_file

ROOT = Path(__file__).resolve().parents[2]
LAUNCH_ARGS = ['-automation', '+skipIntro']
LAUNCH = '''source "$1"
ron_wine "$2" "$3" "${@:4}"
wait $RON_WINE_PID
'''


def set_map(profile, style):
    path = profile / 'PlayerProfile' / 'Player.dat'
    text = path.read_bytes().decode('utf-8')
    text, a = re.subn(r'<MAP_STYLE>\d+</MAP_STYLE>', f'<MAP_STYLE>{style}</MAP_STYLE>', text)
    text, b = re.subn(r'<MAP_STYLE value="\d+"/>', f'<MAP_STYLE value="{style}"/>', text)
    if a != 1 or b != 2:
        raise ValueError(f'expected three map settings, found {a}+{b}')
    path.write_bytes(text.encode('utf-8'))


def mute(profile):
    # The game plays sound at whoever is near the machine (parked 343, the
    # sixth pass 2026-09-19). Same file, same mechanism and same restore as
    # set_map: three volume tags, each expected exactly once.
    path = profile / 'PlayerProfile' / 'Player.dat'
    text = path.read_bytes().decode('utf-8')
    for tag in ('MUSIC_VOL', 'SOUND_VOL', 'TAUNT_VOL'):
        text, n = re.subn(rf'<{tag} value="\d+"/>', f'<{tag} value="0"/>', text)
        if n != 1:
            raise ValueError(f'expected one {tag} setting, found {n}')
    path.write_bytes(text.encode('utf-8'))


def set_lobby(profile, pairs):
    # A lobby field the map style does not carry — the second pair's
    # `DIFFICULTY=5` (DECISIONS 53) — in both of the profile's game blocks,
    # the same write `tools/gamelog/profile.py` makes for the queue lane.
    # Same file as set_map, so the staged `PlayerProfile` backup restores it.
    path = profile / 'PlayerProfile' / 'Player.dat'
    text = path.read_bytes().decode('utf-8')
    for pair in pairs:
        key, value = pair.split('=')
        text, n = re.subn(rf'<{key} value="-?\d+"/>', f'<{key} value="{int(value)}"/>', text)
        if n < 2:
            raise ValueError(f'expected {key} in <SOLO> and <MULTI>, found {n}')
    path.write_bytes(text.encode('utf-8'))


def stalled_before_frame_zero(gamelog):
    """True while the game has written no gamelog at all — what a launch
    stalled in DXVK's device setup looks like (parked 762). A gamelog with
    any bytes is a game past the device, and it is left alone: the
    per-frame wait is the caller's `--timeout`."""
    try:
        return gamelog.stat().st_size == 0
    except FileNotFoundError:
        return True


def verify_game(path, style, end, seed=None, detail=None):
    # Read back the game's identity, never infer it from requested settings.
    styles = set()
    seeds = set()
    closing = False
    frame = None
    groupdata = 0
    with path.open(errors='strict') as f:
        for line in f:
            m = re.fullmatch(r'\s*MAP_STYLE (\d+)\s*', line)
            if m: styles.add(int(m[1]))
            m = re.fullmatch(r'\s*\(int\)seed (\d+)\s*', line)
            if m: seeds.add(int(m[1]))
            m = re.fullmatch(r'\s*BEGIN FRAME (\d+)\s*', line)
            if m: frame = int(m[1])
            if line.strip() == 'GameInfo closing' and frame == end + 1:
                closing = True
            if line.strip() == 'BEGIN GROUPDATA':
                groupdata += 1
    if styles != {style} or not closing:
        raise ValueError(f'game identity/closing dump mismatch: maps={styles}, closing={closing}')
    if seed is not None and seeds != {seed}:
        raise ValueError(f'seed read-back mismatch: {seeds}')
    # A capture that asked for the group pool and printed none of it is a
    # failed capture (parked 735): run210 and run223 lost the pool silently,
    # and the chapters they were run for are about the pool.
    asked = any(cats.get('GROUPS', 0) > 0 for cats in live_session.parse_detail(detail or ()).values())
    if asked and groupdata == 0:
        raise ValueError('GROUPS was asked for and no GROUPDATA block was printed (parked 735)')
    return {'map_style': style, 'closing_frame': end+1, 'seed_observed': sorted(seeds),
            'groupdata_blocks': groupdata}


def verify_restored(output, profile):
    required = [output/'settings-backup'/name for name in live_session.NAMES]
    required.append(output/'settings-backup/PlayerProfile/Player.dat')
    if not all(p.is_file() for p in required):
        raise ValueError('incomplete settings backup')
    count = 0
    for backup in (output / 'settings-backup').rglob('*'):
        if backup.is_file():
            target = profile / backup.relative_to(output / 'settings-backup')
            if target.read_bytes() != backup.read_bytes():
                raise ValueError(f'settings restoration mismatch: {target}')
            count += 1
    return count


def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024*1024), b''): h.update(chunk)
    return h.hexdigest()


def staged_callwin(output):
    """The proxies' call window as `rontrace.cfg` holds it, `[lo, hi]` or None.

    Read back from the file the tracer reads, never from the arguments
    (parked 649, the thirteenth pass): `live_session.stage` writes
    `callwin=0-<end>` when no `--callwin` was given, so a receipt that
    echoed the argument said `null` for run157 while the trace held 25,992
    proxied calls, and a worker who trusted it would not have looked.
    """
    cfg = output / 'rontrace.cfg'
    if not cfg.is_file():
        return None
    for line in cfg.read_text().splitlines():
        if line.startswith('callwin='):
            lo, _, hi = line[len('callwin='):].partition('-')
            try:
                return [int(lo), int(hi)]
            except ValueError:
                return None
    return None


def capture(args, output, style):
    report = {'map_requested': style, 'success': False, 'settings_restored': False}
    staged = False
    process = None
    started = time.monotonic()
    try:
        live_session.require_closed()
        minute = getattr(args, 'ffwd_minute', None)
        live_session.stage(SimpleNamespace(install=args.install, output=output, profile=args.profile,
                                           end_frame=args.end_frame,
                                           fast_forward=args.end_frame>37 and minute is None,
                                           hide_scene=False, ffwd_minute=minute,
                                           log_window=getattr(args, 'log_window', None),
                                           detail=getattr(args, 'detail', None),
                                           cover=getattr(args, 'cover', None),
                                           callwin=getattr(args, 'callwin', None),
                                           cmd_file=getattr(args, 'cmd_file', None)))
        # The receipt carries what was staged, so a run's window and detail are
        # read back from the run rather than from the command that asked for it.
        report['staged'] = {'log_window': getattr(args, 'log_window', None) or list(live_session.DEFAULT_WINDOW),
                            'detail': getattr(args, 'detail', None) or list(live_session.DEFAULT_DETAIL),
                            'cover': getattr(args, 'cover', None) or 'cover=0',
                            'callwin': staged_callwin(output),
                            'tracer_defs': getattr(args, 'tracer_defs', None),
                            'rontrace.cmd': (output/'rontrace.cmd').read_text().splitlines()
                                            if (output/'rontrace.cmd').is_file() else None}
        staged = True
        set_map(args.profile, style)
        mute(args.profile)
        report['lobby'] = getattr(args, 'lobby', None) or []
        set_lobby(args.profile, report['lobby'])
        rise = args.profile / 'rise.ini'
        rise.write_text(live_session.key(rise.read_text(), 'Seed (0 for random)', args.seed))
        env = os.environ.copy()
        report['startup_probe'] = getattr(args, 'startup_probe', False)
        env['TRACER_DEFS'] = ('-DRON_AUTOSTART'
                              + (' -DRON_STARTUP_PROBE' if report['startup_probe'] else '')
                              + ''.join(' -D' + d for d in getattr(args, 'tracer_defs', None) or []))
        with (output / 'build.log').open('w') as log:
            subprocess.run(['zsh', str(ROOT/'tools/trace/build.sh'), str(output)],
                           env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
        report['build_seconds'] = time.monotonic()-started
        report['sha256'] = {name: sha(output/name) for name in
                            ('riseofnations.exe','riseofnations_trace.exe','rontrace.dll','rontrace.cmd','rontrace.cfg')}
        report['launch_args'] = LAUNCH_ARGS[:]
        report['wine_debug'] = os.environ.get('WINEDEBUG', '-all')
        launch = time.monotonic()
        def start():
            return subprocess.Popen(['zsh','-c',LAUNCH,'unattended',str(ROOT/'tools/gamelog/winelaunch.sh'),
                                    str(output/'wine.log'),str(output/'riseofnations_trace.exe'),*LAUNCH_ARGS],
                                   cwd=output, start_new_session=True)
        process = start()
        # A launch that has written no gamelog by `--stall-seconds` is
        # DXVK's device setup stalled before frame 0 (parked 762: run157 and
        # run223 each sat for the whole timeout with `wine.log` ending at
        # MoltenVK's VkInstance); it is killed and started once more.
        stall = getattr(args, 'stall_seconds', 0) or 0
        deadline = launch + args.timeout
        while True:
            try:
                report['exit_code'] = process.wait(timeout=5)
                break
            except subprocess.TimeoutExpired:
                pass
            now = time.monotonic()
            if stall and 'relaunched_after_seconds' not in report and now - launch >= stall \
                    and stalled_before_frame_zero(output/'gamelog.txt'):
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=10)
                (output/'wine.log').replace(output/'wine-stalled.log')
                report['relaunched_after_seconds'] = now - launch
                launch = time.monotonic()
                process = start()
                continue
            if now >= deadline:
                raise subprocess.TimeoutExpired(LAUNCH, args.timeout)
        report['launch_to_exit_seconds'] = time.monotonic()-launch
        report.update(receipt_file(output/'rontrace.log',args.end_frame,report['exit_code']))
        report.update(verify_game(output/'gamelog.txt',style,args.end_frame,args.seed,
                                  detail=getattr(args, 'detail', None) or live_session.DEFAULT_DETAIL))
        report['map_verified'] = True
        report['seed_requested'] = args.seed
        report['success'] = True
    except BaseException as exc:
        # A timeout lands here too (parked 565): the receipt is still written
        # below, with the error and an explicit failure, so a wait keyed on
        # the receipt sees a verdict. A wait keys on this process's exit
        # regardless; the receipt is the record, not the signal.
        report['error'] = f'{type(exc).__name__}: {exc}'
        report['success'] = False
        raise
    finally:
        try:
            if process is not None and process.poll() is None:
                # Only our launch process group, never all Wine sessions.
                os.killpg(process.pid, signal.SIGTERM)
                try: process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=10)
            if staged:
                live_session.require_closed()
                live_session.restore(output)
                report['restored_files'] = verify_restored(output,args.profile)
                report['settings_restored'] = True
        except BaseException as exc:
            report['success'] = False
            report['cleanup_error'] = f'{type(exc).__name__}: {exc}'
            raise
        finally:
            report['total_seconds'] = time.monotonic()-started
            if output.is_dir():
                (output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


LANE = ROOT / 'tools/gamelog/winelaunch.sh'


def lane(verb):
    """The launch line's lane lock, spoken to: `ron_lane_take <pid>`,
    `ron_lane_release <pid>`, `ron_lane_state`."""
    return subprocess.run(['zsh', '-c', f'source {LANE}; {verb}'], capture_output=True, text=True)


@contextmanager
def capture_lane(profile):
    """This runner's lane, held from before its first write to after its
    restore (parked 974, 1234).

    Two locks were two answers: the profile's `flock` here, held for the
    runner's whole life, and the launch line's `.lane.lock`, which `ron_wine`
    wrote with the game's pid alone — so the moment run467's game exited the
    lane read `stale` while this process's `finally` was still restoring the
    profile, and item 1221's long trace launched into the restore. The lane
    is taken for this process's pid first, so it is held while the runner
    lives; the launches carry the pid in `RON_LANE_TAKEN` and go; and the
    lane is released at the end, so a waiter reads `free` and not `stale`.
    The `flock` stays as the cheap same-host check it always was.
    """
    # Keep the inode: unlinking a lock file allows concurrent owners.
    with (profile/'.attrition-capture.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        pid = os.getpid()
        os.environ.setdefault('RON_LANE_HOLDER', 'unattended_capture')
        took = lane(f'ron_lane_take {pid}')
        if took.returncode:
            raise BlockingIOError(took.stderr.strip() or f'the lane refused the take (rc {took.returncode})')
        os.environ['RON_LANE_TAKEN'] = str(pid)
        try:
            yield
        finally:
            os.environ.pop('RON_LANE_TAKEN', None)
            lane(f'ron_lane_release {pid}')


def capture_all(args):
    """Every map into its own directory under `args.output`, and the receipt.

    A refusal before anything was written leaves no directory (parked 1108,
    the nineteenth pass): item 1099's window past `!quit` + 1 was refused
    after the output directory was made, and the relaunch with the window
    put right was refused by the directory. `rmdir` takes only what is empty,
    so a failed capture's receipt keeps everything above it.
    """
    args.output.mkdir(parents=True,exist_ok=False)
    reports=[]
    try:
        for style in args.maps:
            reports.append(capture(args,args.output/f'map-{style}',style))
            print(json.dumps(reports[-1]),flush=True)
    except BaseException:
        for made in [*sorted(args.output.glob('map-*')),args.output]:
            try: made.rmdir()
            except OSError: pass
        raise
    (args.output/'receipt.json').write_text(json.dumps(reports,indent=2)+'\n')


def interrupted(signum, frame):
    raise InterruptedError(f'capture interrupted by signal {signum}')


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path)
    ap.add_argument('output',type=Path)
    ap.add_argument('profile',type=Path)
    ap.add_argument('--end-frame',type=int,default=36)
    ap.add_argument('--seed',type=int,default=12345)
    ap.add_argument('--timeout',type=int,default=180)
    ap.add_argument('--stall-seconds',type=int,default=300,
                    help='relaunch once when no gamelog has appeared by then (0 disables; parked 762)')
    ap.add_argument('--startup-probe',action='store_true',help='observe WinMain Media Foundation calls')
    ap.add_argument('--map',type=int,action='append',dest='maps',metavar='STYLE',
                    help='map style, repeatable; default 14 then 18')
    ap.add_argument('--log-window',type=int,nargs=2,metavar=('START','END'))
    ap.add_argument('--detail',action='append',metavar='SECTION:CAT[=N],...')
    ap.add_argument('--cover')
    ap.add_argument('--callwin', type=int, nargs=2, metavar=('LO', 'HI'),
                    help='sim-frames the call proxies log over; default 0-END')
    ap.add_argument('--tracer-def', action='append', dest='tracer_defs', metavar='NAME',
                    help='extra tracer.c define, repeatable (e.g. RON_TARGET_PROBE)')
    ap.add_argument('--cmd-file',type=Path)
    ap.add_argument('--ffwd-minute',type=int)
    ap.add_argument('--profile', action='append', dest='lobby', metavar='KEY=N',
                    help="a lobby field in the profile's <SOLO>/<MULTI> blocks, repeatable "
                         "(the queue lane's `profile:` key; e.g. DIFFICULTY=5)")
    args=ap.parse_args()
    if any(not re.fullmatch(r'[A-Z_0-9]+=-?\d+', p) for p in args.lobby or []):
        ap.error('--profile takes KEY=N')
    args.maps=tuple(args.maps or (14,18))
    signal.signal(signal.SIGTERM, interrupted)
    args.install,args.profile,args.output=(p.resolve() for p in (args.install,args.profile,args.output))
    if not 36<=args.end_frame<=24000 or args.timeout<=0 or not 1<=args.seed<=0x7fffffff:
        ap.error('invalid frame, timeout, or seed bound')
    if args.output.is_relative_to(args.install) or args.output.is_relative_to(args.profile):
        ap.error('output must be outside install and profile')
    if args.cmd_file is not None:
        args.cmd_file=args.cmd_file.resolve()
        if not args.cmd_file.is_file():
            ap.error(f'no such command file: {args.cmd_file}')
    # Cooperative lock: protects runners using this tool, not arbitrary GUI use.
    with capture_lane(args.profile):
        live_session.require_closed()
        capture_all(args)


if __name__=='__main__': main()
