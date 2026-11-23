#!/usr/bin/env python3
"""filerun.py — file a windowed capture's dump and trace into the archive
under the harness's names, from the runner's output directory.

    python3 tools/gamelog/filerun.py run893c eastindies-german-diff3-window-6579-6837
    python3 tools/gamelog/filerun.py run893c <tag> --map 18 --dry-run

`~/ron-golden/<run>/map-<N>/gamelog.txt` (a playback's `gamelog_p.txt`) becomes
`<Logs>/gamelog-<run>-<tag>.txt` and `rontrace.log` beside it
`<Logs>/rontrace-<run>.log` — the names `crates/rondata` pins (Loop 2933,
2904's line: every journal copied the two by hand under the archive's
name, and no tool owned the step; `archive.sh` is `live.sh`'s). A
destination that exists is refused, never overwritten (`--force` moves it
aside as `<name>.replaced-<time>` first). **A filed capture is one file
under two names**: the archive's name is a hard link to the take's file,
and a copy only where a link cannot be made (another volume), which the
line it prints says. A copy held every capture twice: 1,785 twins, 19.2
GiB on disk, on 2026-11-23 (item 4600a; DECISIONS 155 §6). The archive's
compression keeps a link, and nothing rewrites a filed capture in place. The map directory is the one
`map-*` under the run unless `--map` names it; two without `--map` refuse.
A ranges take's `rontrace.ranges` past a gigabyte is split in place into
pieces and compressed (`tools/trace/split.py`), and the line it prints
carries the record's bytes and SHA-256 for the ledger.
"""
import argparse
import os
import shutil
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'trace'))
import split  # noqa: E402

GOLDEN = Path(os.environ.get('RON_GOLDEN_DIR', os.path.expanduser('~/ron-golden')))
LOGS = Path(os.environ.get('RON_GAMELOG_DIR', os.path.expanduser(
    '~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs')))


class Refused(Exception):
    pass


def map_dir(golden, run, style=None):
    """The capture's `map-<N>` directory under `golden/<run>`."""
    root = golden / run
    if not root.is_dir():
        raise Refused(f'no capture directory {root}')
    if style is not None:
        d = root / f'map-{style}'
        if not d.is_dir():
            raise Refused(f'no {d}')
        return d
    maps = sorted(p for p in root.glob('map-*') if p.is_dir())
    if len(maps) != 1:
        raise Refused(f'{root} holds {len(maps)} map directories ({", ".join(p.name for p in maps)}); name one with --map')
    return maps[0]


def plan(golden, logs, run, tag, style=None):
    """`[(source, destination)]` for the dump and the trace."""
    d = map_dir(golden, run, style)
    # A recorded game's playback writes `gamelog_p.txt` (`GameLog::init`;
    # `docs/RECGAME.md` §7), and a run holds one or the other (item 3428).
    dump = d / 'gamelog.txt'
    if not dump.is_file() and (d / 'gamelog_p.txt').is_file():
        dump = d / 'gamelog_p.txt'
    pairs = [(dump, logs / f'gamelog-{run}-{tag}.txt'),
             (d / 'rontrace.log', logs / f'rontrace-{run}.log')]
    for src, _ in pairs:
        if not src.is_file():
            raise Refused(f'no {src}')
    return pairs


def place(src, dst, link=os.link):
    """`dst` as a hard link to `src`, or a copy where no link can be made;
    the word for which."""
    try:
        link(src, dst)
        return 'linked'
    except OSError:
        shutil.copy2(src, dst)
        return 'copied'


def file_run(golden, logs, run, tag, style=None, force=False, dry_run=False, out=sys.stdout, link=os.link):
    pairs = plan(golden, logs, run, tag, style)
    for _, dst in pairs:
        if dst.exists() and not force:
            raise Refused(f'{dst} exists; --force moves it aside first')
    for src, dst in pairs:
        if dry_run:
            print(f'{src} -> {dst}', file=out)
            continue
        if dst.exists():
            aside = dst.with_name(f'{dst.name}.replaced-{int(time.time())}')
            dst.replace(aside)
            print(f'moved aside: {aside}', file=out)
        how = place(src, dst, link)
        print(f'{dst} ({dst.stat().st_size} bytes, {how})', file=out)
    # A ranges take's record stays where the take left it, and past a
    # gigabyte it is kept in pieces the archive's compression takes
    # (`tools/trace/split.py`, DECISIONS 149 §1). The take has exited, so no
    # age is asked. A refusal leaves the record whole and is said, never fatal.
    record = pairs[1][0].with_name('rontrace.ranges')
    if record.is_file() and not dry_run:
        try:
            done = split.split(record, min_age=0)
        except split.Refused as e:
            print(f'the record stays whole: {e}', file=out)
        else:
            if done:
                print(f"{record}: {done['size']} bytes in {done['pieces']} pieces, SHA-256 {done['sha256']}; "
                      f"{split.gib(done['before'])} on disk before, {split.gib(done['after'])} after"
                      + ('' if done['compressed'] else '; afsctool is not on this PATH, the pieces are not compressed'),
                      file=out)
    return pairs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('run', help='runNNN[x], the directory under ~/ron-golden')
    ap.add_argument('tag', help="the archive's tag: <map>-<nation>-diff<N>-window-<lo>-<hi>")
    ap.add_argument('--map', type=int, metavar='N', help='the map-N directory when the run holds several')
    ap.add_argument('--golden', type=Path, default=GOLDEN)
    ap.add_argument('--logs', type=Path, default=LOGS)
    ap.add_argument('--force', action='store_true', help='move an existing destination aside first')
    ap.add_argument('--dry-run', action='store_true')
    a = ap.parse_args(argv)
    try:
        file_run(a.golden, a.logs, a.run, a.tag, a.map, a.force, a.dry_run)
    except Refused as e:
        print(f'filerun.py refuses: {e}', file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
