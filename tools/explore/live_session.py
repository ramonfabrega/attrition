#!/usr/bin/env python3
"""Prepare/restore the bounded live probes without overwriting installed tracers.

stage INSTALL OUTPUT PROFILE_DIRECTORY [--hide-scene]
restore OUTPUT
Run with the game closed. Build into OUTPUT after staging, then launch its
riseofnations_trace.exe using the existing Wine settings and lobby helpers.
PROFILE_DIRECTORY contains rise.ini, rise2.ini, gamelog.ini, PlayerProfile.
"""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess

NAMES = ('rise.ini', 'rise2.ini', 'gamelog.ini')


def require_closed():
    result = subprocess.run(['ps', '-axo', 'comm='], capture_output=True, text=True, check=True)
    if 'riseofnations' in result.stdout.lower():
        raise RuntimeError('close the original game before staging/restoring')


def key(text, name, value):
    text, count = re.subn(r'^' + re.escape(name) + r'=.*$', lambda _: f'{name}={value}', text, flags=re.M)
    if count != 1:
        raise ValueError(f'expected one {name}, found {count}')
    return text


def restore(output):
    metadata = json.loads((output / 'session.json').read_text())
    profile = Path(metadata['profile'])
    backup = output / 'settings-backup'
    for name in NAMES:
        shutil.copy2(backup / name, profile / name)
        assert (backup / name).read_bytes() == (profile / name).read_bytes()
    # Restore existing profile files; never remove new user/game files.
    for p in (backup / 'PlayerProfile').rglob('*'):
        if p.is_file():
            target = profile / 'PlayerProfile' / p.relative_to(backup / 'PlayerProfile')
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(p, target)
    print('Restored backed-up settings; capture outputs retained:', output)


def stage(args):
    install, output, profile = args.install.resolve(), args.output.resolve(), args.profile.resolve()
    if output.is_relative_to(install) or output.is_relative_to(profile):
        raise ValueError('output must be outside the install and profile trees')
    if not (install / 'riseofnations.exe').is_file():
        raise ValueError('install has no riseofnations.exe')
    # Validate all edits before changing shared settings.
    rise = key((profile / 'rise.ini').read_text(), 'InitialDump', 0)
    rise2 = key(key((profile / 'rise2.ini').read_text(), 'LogStartFrame', 18), 'LogEndFrame', 36)
    # Wine Z: maps the host root. Keep backslashes out of re.sub replacement strings.
    wine_output = 'Z:' + str(output).replace('/', '\\')
    log = key(key(key((profile / 'gamelog.ini').read_text(), 'DUMP_ALL', 0),
                  'LogFile', wine_output + '\\gamelog.txt'),
              'DumpFileName', wine_output + '\\dumplog.txt')
    lines, section = [], ''
    for line in log.splitlines():
        if line.startswith('['):
            section = line
        if '=' in line and section != '[Logging Options]':
            name = line.split('=', 1)[0]
            value = 3 if section == '[End Frame]' and name == 'UNITS' else \
                1 if section == '[Misc Logging]' and name == 'COMMANDMANAGER' else 0
            line = f'{name}={value}'
        lines.append(line)
    output.mkdir(parents=True, exist_ok=False)
    backup = output / 'settings-backup'
    backup.mkdir()
    for name in NAMES:
        shutil.copy2(profile / name, backup / name)
    shutil.copytree(profile / 'PlayerProfile', backup / 'PlayerProfile')
    (output / 'session.json').write_text(json.dumps({'profile': str(profile), 'install': str(install)}, indent=2))
    try:
        for source in install.iterdir():
            if source.name.startswith('rontrace') or source.name in ('Logs', 'riseofnations_trace.exe'):
                continue
            target = output / source.name
            if source.suffix.lower() in ('.ini', '.exe'):
                shutil.copy2(source, target)
            elif source.suffix.lower() != '.log':
                target.symlink_to(source, target_is_directory=source.is_dir())
        (output / 'Logs').mkdir()
        (output / 'rontrace.cfg').write_text('cover=0\ncallwin=0-36\n')
        (output / 'rontrace.cmd').write_text('36 !quit\n')
        for name, text in zip(NAMES, (rise, rise2, '\n'.join(lines) + '\n')):
            (profile / name).write_text(text)
    except BaseException:
        restore(output)
        raise
    print('Staged:', output)
    print('Build with TRACER_DEFS="-DRON_COMMAND_PROBE -DRON_TURN_PROBE' +
          (' -DRON_HIDE_SCENE' if args.hide_scene else '') + '" into this directory.')
    print('After the run, close the game and invoke restore with this output directory.')


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    modes = ap.add_subparsers(dest='mode', required=True)
    s = modes.add_parser('stage')
    s.add_argument('install', type=Path)
    s.add_argument('output', type=Path)
    s.add_argument('profile', type=Path)
    s.add_argument('--hide-scene', action='store_true')
    r = modes.add_parser('restore')
    r.add_argument('output', type=Path)
    args = ap.parse_args()
    require_closed()
    if args.mode == 'stage':
        stage(args)
    else:
        restore(args.output)


if __name__ == '__main__':
    main()
