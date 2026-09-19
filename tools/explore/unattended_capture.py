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


def verify_game(path, style, end, seed=None):
    # Read back the game's identity, never infer it from requested settings.
    styles = set()
    seeds = set()
    closing = False
    frame = None
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
    if styles != {style} or not closing:
        raise ValueError(f'game identity/closing dump mismatch: maps={styles}, closing={closing}')
    if seed is not None and seeds != {seed}:
        raise ValueError(f'seed read-back mismatch: {seeds}')
    return {'map_style': style, 'closing_frame': end+1, 'seed_observed': sorted(seeds)}


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
                                           cmd_file=getattr(args, 'cmd_file', None)))
        # The receipt carries what was staged, so a run's window and detail are
        # read back from the run rather than from the command that asked for it.
        report['staged'] = {'log_window': getattr(args, 'log_window', None) or list(live_session.DEFAULT_WINDOW),
                            'detail': getattr(args, 'detail', None) or list(live_session.DEFAULT_DETAIL),
                            'cover': getattr(args, 'cover', None) or 'cover=0',
                            'rontrace.cmd': (output/'rontrace.cmd').read_text().splitlines()
                                            if (output/'rontrace.cmd').is_file() else None}
        staged = True
        set_map(args.profile, style)
        rise = args.profile / 'rise.ini'
        rise.write_text(live_session.key(rise.read_text(), 'Seed (0 for random)', args.seed))
        env = os.environ.copy()
        report['startup_probe'] = getattr(args, 'startup_probe', False)
        env['TRACER_DEFS'] = '-DRON_AUTOSTART' + (' -DRON_STARTUP_PROBE' if report['startup_probe'] else '')
        with (output / 'build.log').open('w') as log:
            subprocess.run(['zsh', str(ROOT/'tools/trace/build.sh'), str(output)],
                           env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
        report['build_seconds'] = time.monotonic()-started
        report['sha256'] = {name: sha(output/name) for name in
                            ('riseofnations.exe','riseofnations_trace.exe','rontrace.dll','rontrace.cmd','rontrace.cfg')}
        report['launch_args'] = LAUNCH_ARGS[:]
        report['wine_debug'] = os.environ.get('WINEDEBUG', '-all')
        launch = time.monotonic()
        process = subprocess.Popen(['zsh','-c',LAUNCH,'unattended',str(ROOT/'tools/gamelog/winelaunch.sh'),
                                    str(output/'wine.log'),str(output/'riseofnations_trace.exe'),*LAUNCH_ARGS],
                                   cwd=output, start_new_session=True)
        report['exit_code'] = process.wait(timeout=args.timeout)
        report['launch_to_exit_seconds'] = time.monotonic()-launch
        report.update(receipt_file(output/'rontrace.log',args.end_frame,report['exit_code']))
        report.update(verify_game(output/'gamelog.txt',style,args.end_frame,args.seed))
        report['map_verified'] = True
        report['seed_requested'] = args.seed
        report['success'] = True
    except BaseException as exc:
        report['error'] = f'{type(exc).__name__}: {exc}'
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


@contextmanager
def capture_lane(profile):
    # Keep the inode: unlinking a lock file allows concurrent owners.
    with (profile/'.attrition-capture.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


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
    ap.add_argument('--startup-probe',action='store_true',help='observe WinMain Media Foundation calls')
    ap.add_argument('--map',type=int,action='append',dest='maps',metavar='STYLE',
                    help='map style, repeatable; default 14 then 18')
    ap.add_argument('--log-window',type=int,nargs=2,metavar=('START','END'))
    ap.add_argument('--detail',action='append',metavar='SECTION:CAT[=N],...')
    ap.add_argument('--cover')
    ap.add_argument('--cmd-file',type=Path)
    ap.add_argument('--ffwd-minute',type=int)
    args=ap.parse_args()
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
        args.output.mkdir(parents=True,exist_ok=False)
        reports=[]
        for style in args.maps:
            reports.append(capture(args,args.output/f'map-{style}',style))
            print(json.dumps(reports[-1]),flush=True)
        (args.output/'receipt.json').write_text(json.dumps(reports,indent=2)+'\n')


if __name__=='__main__': main()
