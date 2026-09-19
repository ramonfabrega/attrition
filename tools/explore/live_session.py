#!/usr/bin/env python3
"""Prepare/restore the bounded live probes without overwriting installed tracers.

stage INSTALL OUTPUT PROFILE_DIRECTORY [--hide-scene]
restore OUTPUT
Run with the game closed. Build into OUTPUT after staging, then launch its
riseofnations_trace.exe using the existing Wine settings and lobby helpers.
PROFILE_DIRECTORY contains rise.ini, rise2.ini, gamelog.ini, PlayerProfile.

The dump window, the logging detail and the command file are caller-supplied
(`--log-window`, `--detail`, `--cmd-file`); their defaults are the startup
receipt the autostart lane was written for. `--detail` takes setlog.py's
`SECTION:CAT[=N],...` spelling, every category in a named section that is not
listed goes to 0, and a section that is not named goes to 0 entirely.
"""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess

NAMES = ('rise.ini', 'rise2.ini', 'gamelog.ini')
# setlog.py's section spelling, so one vocabulary covers both lanes.
SECTIONS = {'end': '[End Frame]', 'start': '[Start Game]', 'misc': '[Misc Logging]',
            'endgame': '[End Game]', 'startframe': '[Start Frame]'}
DEFAULT_DETAIL = ('end:UNITS=3', 'misc:COMMANDMANAGER=1')
DEFAULT_WINDOW = (18, 36)


def parse_detail(specs):
    """setlog.py's `SECTION:CAT[=N],...` into {ini section: {category: level}}."""
    wanted = {}
    for spec in specs:
        name, _, cats = spec.partition(':')
        if name not in SECTIONS:
            raise ValueError(f'unknown detail section {name!r}; want one of {sorted(SECTIONS)}')
        section = wanted.setdefault(SECTIONS[name], {})
        for cat in cats.split(','):
            if not cat:
                continue
            key, _, value = cat.partition('=')
            if not re.fullmatch(r'[A-Za-z0-9_]+', key):
                raise ValueError(f'bad category name {key!r} in {spec!r}')
            if not re.fullmatch(r'[0-9]', value or '1'):
                raise ValueError(f'bad detail level {value!r} in {spec!r}')
            section[key] = int(value or 1)
    return wanted


def parse_commands(path, end):
    """`<sim-frame> <text>` lines, as rontrace.dll reads them; `#` comments kept."""
    lines = []
    previous = 0
    for number, raw in enumerate(Path(path).read_text().splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith('#'):
            continue
        frame, _, text = line.partition(' ')
        if not frame.isdigit() or not text.strip():
            raise ValueError(f'{path}:{number}: want `<sim-frame> <text>`, got {raw!r}')
        if int(frame) > end:
            raise ValueError(f'{path}:{number}: frame {frame} is past the !quit frame {end}')
        # rontrace.dll clamps a frame below its predecessor; refuse rather than reorder.
        if int(frame) < previous:
            raise ValueError(f'{path}:{number}: frame {frame} is below the previous {previous}')
        previous = int(frame)
        lines.append(f'{int(frame)} {text.strip()}')
    return lines


def require_closed():
    result = subprocess.run(['ps', '-axo', 'comm='], capture_output=True, text=True, check=True)
    if 'riseofnations' in result.stdout.lower():
        raise RuntimeError('close the original game before staging/restoring')


def key(text, name, value):
    text, count = re.subn(r'^' + re.escape(name) + r'=.*$', lambda _: f'{name}={value}', text, flags=re.M)
    if count != 1:
        raise ValueError(f'expected one {name}, found {count}')
    return text


def section_key(text, section, name, value):
    lines = text.splitlines(keepends=True)
    current, count = '', 0
    for i, line in enumerate(lines):
        if line.startswith('['):
            current = line.strip()
        if current == section and line.startswith(name + '='):
            ending = '\r\n' if line.endswith('\r\n') else '\n' if line.endswith('\n') else ''
            lines[i] = name + '=' + str(value) + ending
            count += 1
    if count != 1:
        raise ValueError(f'expected one {section} {name}, found {count}')
    return ''.join(lines)


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
    end = getattr(args, 'end_frame', 36)
    fast = getattr(args, 'fast_forward', False)
    window = tuple(getattr(args, 'log_window', None) or DEFAULT_WINDOW)
    detail = list(getattr(args, 'detail', None) or DEFAULT_DETAIL)
    cover = getattr(args, 'cover', None) or 'cover=0'
    callwin = getattr(args, 'callwin', None)
    cmd_file = getattr(args, 'cmd_file', None)
    minute = getattr(args, 'ffwd_minute', None)
    if not 36 <= end <= 24000 or (fast and end <= 37):
        raise ValueError('end-frame must be 36..24000; fast-forward requires at least 38')
    if args.hide_scene and end != 36:
        raise ValueError('render-suppression experiment requires end-frame 36')
    # The window is [start, end) in the engine's own keys; a capture that asks for
    # frames past its !quit gets a short window and no warning, so refuse here.
    if not 0 <= window[0] <= window[1] <= end + 1:
        raise ValueError(f'log window {window} must satisfy 0 <= start <= end <= quit frame + 1 ({end + 1})')
    if fast and minute is not None:
        raise ValueError('fast-forward and ffwd-minute both schedule frame 37; pick one')
    if minute is not None and not 1 <= minute <= 27:
        raise ValueError('ffwd-minute must be 1..27 (fast_forward_frame = minute * 900)')
    wanted = parse_detail(detail)
    if output.is_relative_to(install) or output.is_relative_to(profile):
        raise ValueError('output must be outside the install and profile trees')
    if not (install / 'riseofnations.exe').is_file():
        raise ValueError('install has no riseofnations.exe')
    commands = parse_commands(cmd_file, end) if cmd_file else []
    # Validate all edits before changing shared settings.
    rise = key((profile / 'rise.ini').read_text(), 'InitialDump', 0)
    rise2 = key(key((profile / 'rise2.ini').read_text(), 'LogStartFrame', window[0]),
                'LogEndFrame', window[1])
    # Wine Z: maps the host root. Keep backslashes out of re.sub replacement strings.
    wine_output = 'Z:' + str(output).replace('/', '\\')
    log = (profile / 'gamelog.ini').read_text()
    for name, value in [('DUMP_ALL', 0), ('LogFile', wine_output + '\\gamelog.txt'),
                        ('DumpFileName', wine_output + '\\dumplog.txt')]:
        log = section_key(log, '[Logging Options]', name, value)
    lines, section, seen = [], '', {}
    for line in log.splitlines():
        if line.startswith('['):
            section = line
        if '=' in line and section != '[Logging Options]':
            name = line.split('=', 1)[0]
            if name in ('DumpFileName', 'LogFile', 'DUMP_ALL'):
                lines.append(line)
                continue
            # A section nobody asked for goes to 0 entirely; so does an unlisted
            # category in one that was asked for. Nothing is left at the profile's.
            line = f'{name}={wanted.get(section, {}).get(name, 0)}'
            seen.setdefault(section, set()).add(name)
        lines.append(line)
    missing = {s: sorted(set(c) - seen.get(s, set())) for s, c in wanted.items()}
    missing = {s: c for s, c in missing.items() if c}
    if missing:
        raise ValueError(f'gamelog.ini has no such categories: {missing}')
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
        # The proxies log every call in the window, so a probe that only needs
        # a few frames says so rather than paying for the whole run.
        cw = f'{callwin[0]}-{callwin[1]}' if callwin else f'0-{end}'
        (output / 'rontrace.cfg').write_text(f'{cover}\ncallwin={cw}\n')
        if fast:
            minute = (end + 899) // 900
        # The channel runs lines in file order and clamps a frame below its
        # predecessor, so the fast-forward goes in at frame 37's own place —
        # after a `!ai off` at frame 0, before the first staged line above it.
        script = list(commands)
        if minute is not None:
            at = sum(1 for line in script if int(line.split(' ', 1)[0]) <= 37)
            script.insert(at, f'37 !ffwd {minute}')
        (output / 'rontrace.cmd').write_text('\n'.join(script + [f'{end} !quit']) + '\n')
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
    s.add_argument('--end-frame', type=int, default=36)
    s.add_argument('--fast-forward', action='store_true',
                   help='schedule native ffwd at frame 37, after the detailed logging window')
    s.add_argument('--log-window', type=int, nargs=2, metavar=('START', 'END'),
                   help=f'rise2.ini LogStartFrame/LogEndFrame; default {DEFAULT_WINDOW}')
    s.add_argument('--detail', action='append', metavar='SECTION:CAT[=N],...',
                   help="setlog.py's spelling, repeatable; default " + ' '.join(DEFAULT_DETAIL))
    s.add_argument('--cover', help="rontrace.cfg's first line; default cover=0")
    s.add_argument('--callwin', type=int, nargs=2, metavar=('LO', 'HI'),
                   help='sim-frames the call proxies log over; default 0-END')
    s.add_argument('--cmd-file', type=Path,
                   help='rontrace.cmd lines to stage before the !quit, `<sim-frame> <text>`')
    s.add_argument('--ffwd-minute', type=int,
                   help='schedule `!ffwd MINUTE` (fast_forward_frame = MINUTE * 900) at frame 37')
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
