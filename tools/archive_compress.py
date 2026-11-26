#!/usr/bin/env python3
"""archive_compress.py — the pass's first act: the archives compressed in
place, one tree at a time, and the damage a compressor can leave looked for.

`afsctool -c -f -T LZFSE -J8 <tree>` on `~/ron-data`, then on `~/ron-golden`,
never both in one run. Since 2026-11-23 a filed take is a hard link
(`tools/gamelog/filerun.py`, DECISIONS 155 §6), so one inode has a name in
each tree. Given both trees at once the compressor queues both names, two of
its workers take the one inode together, and the loser leaves the file with
its data in the resource fork, a valid `com.apple.decmpfs` header and **no
`UF_COMPRESSED` flag**: it reads as zero bytes. Measured on 24 linked pairs
of 44 MB: 17 came out so (DECISIONS 156 §6). Four trace logs in the archive
did on the pass of 2026-11-23, third.

Nothing is lost in that state. `--repair` checks the header, sets the flag
and reads the file back to the length the header declares.

    python3 tools/archive_compress.py            # split, compress, scan
    python3 tools/archive_compress.py --check    # scan only
    python3 tools/archive_compress.py --repair   # scan, and mend what it finds

The compressor also takes a file that is open for writing: the writer's
next write fails, and a script whose log it was ends with nothing said. The
pass of 2026-11-25, second, lost this tool's own output and its acts' log
that way, both under `~/ron-data`. So a run that would compress refuses,
exit 2, while its own stdout or stderr is a file under one of its trees,
and nothing else of a pass writes under them until it exits.

Exit 0 when the scan finds nothing (or mended everything), 1 when a file
stands damaged, 2 when the run is refused.
"""
import argparse
import fcntl
import os
import shutil
import stat
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TREES = (Path.home() / 'ron-data', Path.home() / 'ron-golden')
AFSCTOOL = ('afsctool', '/opt/homebrew/bin/afsctool', '/usr/local/bin/afsctool')
UF_COMPRESSED = 0x20
DECMPFS = 'com.apple.decmpfs'


class Refused(Exception):
    pass


def damaged(st):
    """A regular file that reads empty, holds blocks, and is not flagged
    compressed: the state the compressor's race leaves."""
    return (stat.S_ISREG(st.st_mode) and st.st_size == 0 and st.st_blocks > 0
            and not st.st_flags & UF_COMPRESSED)


def scan(trees):
    """`{inode: [paths]}` of the damaged files under `trees`."""
    found = {}
    for tree in trees:
        for d, _, files in os.walk(tree):
            for f in files:
                p = os.path.join(d, f)
                try:
                    st = os.lstat(p)
                except OSError:
                    continue
                if damaged(st):
                    found.setdefault((st.st_dev, st.st_ino), []).append(p)
    return found


def header(path):
    """The uncompressed length the file's `com.apple.decmpfs` declares."""
    out = subprocess.run(['xattr', '-px', DECMPFS, str(path)], capture_output=True, text=True)
    if out.returncode != 0:
        raise Refused(f'{path}: no {DECMPFS}')
    raw = bytes.fromhex(''.join(out.stdout.split()))
    if len(raw) < 16 or raw[:4] != b'fpmc':
        raise Refused(f'{path}: {DECMPFS} is not a compression header')
    return struct.unpack('<Q', raw[8:16])[0]


def repair(path):
    """Set the flag the compressor did not, and read the file back whole.
    Returns its length. The flag comes off again if the read is short."""
    st = os.lstat(path)
    if not damaged(st):
        raise Refused(f'{path}: not in the damaged state')
    want = header(path)
    os.chflags(path, st.st_flags | UF_COMPRESSED)
    got = 0
    try:
        with open(path, 'rb') as f:
            while chunk := f.read(1 << 20):
                got += len(chunk)
    except OSError:
        got = -1
    if got != want:
        os.chflags(path, st.st_flags)
        raise Refused(f'{path}: read {got} bytes where the header declares {want}; the flag is off again')
    return got


def output_under(trees, fds=(1, 2)):
    """The first of `fds` that is a regular file under one of `trees`, as
    its path; `None` when none is. A pipe, a terminal and a file elsewhere
    are none of the compressor's."""
    roots = [Path(t).resolve() for t in trees]
    for fd in fds:
        try:
            if not stat.S_ISREG(os.fstat(fd).st_mode):
                continue
            raw = fcntl.fcntl(fd, fcntl.F_GETPATH, b'\0' * 1024)
        except (OSError, AttributeError, ValueError):
            continue
        path = Path(os.fsdecode(raw.split(b'\0', 1)[0])).resolve()
        if any(path.is_relative_to(root) for root in roots):
            return path
    return None


def compress(trees, run=subprocess.run, quiet=False):
    """One compressor run a tree, in turn. `False` with no `afsctool` here."""
    exe = next((e for e in AFSCTOOL if shutil.which(e)), None)
    if exe is None:
        return False
    sink = {'stdout': subprocess.DEVNULL, 'stderr': subprocess.DEVNULL} if quiet else {}
    for tree in trees:
        run([exe, '-c', '-f', '-T', 'LZFSE', '-J8', str(tree)], check=False, **sink)
    return True


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--check', action='store_true', help='scan only')
    ap.add_argument('--repair', action='store_true', help='scan, and mend what it finds')
    ap.add_argument('trees', nargs='*', type=Path, help='default: ~/ron-data ~/ron-golden')
    args = ap.parse_args(argv)
    trees = args.trees or [t for t in TREES if t.is_dir()]
    if not (args.check or args.repair):
        inside = output_under(trees)
        if inside is not None:
            print(f'refused: this run writes to {inside}, a file under a tree it would compress; the '
                  'compressor takes an open file and every later write to it fails. Send the output '
                  'outside the trees.', file=sys.stderr)
            return 2
        if not args.trees:
            subprocess.run([sys.executable, str(ROOT / 'trace/split.py'), '--archive'], check=False)
        if not compress(trees):
            print('afsctool is not on this PATH: nothing compressed')
    found = scan(trees)
    left = 0
    for names in found.values():
        if args.repair:
            try:
                print(f'mended, {repair(names[0])} bytes: {" = ".join(names)}')
                continue
            except Refused as e:
                print(e)
        else:
            print(f'reads empty and holds its data: {" = ".join(names)}')
        left += 1
    print(f'scan: {len(found)} damaged, {left} standing' + ('; --repair mends them' if left and not args.repair else ''))
    return 1 if left else 0


if __name__ == '__main__':
    sys.exit(main())
