#!/usr/bin/env python3
"""split.py — a step R record past a gigabyte, kept in pieces the archive's compression takes.

    python3 tools/trace/split.py <rontrace.ranges> […]     # split, compress, verify, swap
    python3 tools/trace/split.py --archive                  # every record past two billion bytes under ~/ron-golden
    python3 tools/trace/split.py --check <rontrace.ranges>  # the record's bytes and SHA-256, whole or in pieces
    python3 tools/trace/split.py --join <rontrace.ranges>   # the undo: one file again

The archives are compressed in place (`afsctool -c -T LZFSE`, the pass's
first act), and `afsctool` takes no file of 2^31 bytes or more: it prints
`Skipping file … with unsupportable size` and goes on. On 2026-11-21
fourteen records stood whole for that, 48.8 GiB between them, while a
1.9 GB piece of one compressed to 151 MiB in ten seconds; the takes' floor
of 60 GB had stopped every seat (DECISIONS 149 §1).

So a record past `WHOLE` bytes is stored, `PIECE` bytes a file, as `rontrace.ranges`,
`rontrace.ranges.1`, `rontrace.ranges.2`, …: the pieces in order are the
record, byte for byte, and both readers (`rondata::trace::ranges`,
`tools/trace/ranges.py`) open them as one. Nothing is dropped and the
ledger's figures stand: a record's bytes and SHA-256 in `docs/RUNS.md` are
the concatenation's, which `--check` prints.

The order is what makes it safe to run on a capture. The pieces are
written beside the record as `.part` files, compressed, and read back; only
when their concatenation has the record's length and SHA-256 are they
renamed into place, the first piece last, over the whole file. A run that
dies before the renames leaves the record whole and `.part` files the next
run removes. A record written in the last ten minutes is refused, because
a take may still be writing it (`--min-age 0` when the take has exited, as
`tools/gamelog/filerun.py` knows it has). A tree older than the readers'
change reads a split record as cut short and says so; `--join` puts it back.
"""
import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

PIECE = 1 << 30
# The largest record left in one file. `afsctool` refuses at 2^31 bytes; a
# record under two billion compresses whole and gains nothing in pieces
# (run1754's 1.3 GB read 0.15 GiB on disk either way).
WHOLE = 2_000_000_000
BLOCK = 8 << 20
GOLDEN = Path(os.environ.get('RON_GOLDEN_DIR', os.path.expanduser('~/ron-golden')))
AFSCTOOL = ('afsctool', '/opt/homebrew/bin/afsctool', '/usr/local/bin/afsctool')


class Refused(Exception):
    pass


def pieces(path):
    """`[path, path.1, path.2, …]`: the record's files, in order."""
    path = Path(path)
    out, k = [path], 1
    while (p := Path(f'{path}.{k}')).is_file():
        out.append(p)
        k += 1
    return out


def digest(paths):
    """`(bytes, SHA-256)` of the files' concatenation."""
    h, size = hashlib.sha256(), 0
    for p in paths:
        with open(p, 'rb') as f:
            while block := f.read(BLOCK):
                h.update(block)
                size += len(block)
    return size, h.hexdigest()


def allocated(paths):
    return sum(os.stat(p).st_blocks * 512 for p in paths)


def afsctool(paths):
    """Compress the files in place; `False` when the tool is not on this machine."""
    exe = next((e for e in AFSCTOOL if shutil.which(e)), None)
    if exe is None:
        return False
    subprocess.run([exe, '-c', '-f', '-T', 'LZFSE', '-J8', *map(str, paths)],
                   check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return True


def split(path, piece=None, compress=afsctool, min_age=600, now=None, whole=None):
    """Split a record past `whole` bytes into pieces of `piece`. Returns what
    it did: `None` for a record that is already in pieces or stays whole."""
    path = Path(path)
    piece = piece or PIECE
    whole = max(piece, WHOLE if whole is None else whole)
    if not path.is_file():
        raise Refused(f'no {path}')
    if len(pieces(path)) > 1:
        return None
    st = path.stat()
    if st.st_size <= whole:
        return None
    age = (now if now is not None else time.time()) - st.st_mtime
    if age < min_age:
        raise Refused(f'{path} was written {int(age)} s ago, under {min_age}: a take may be writing it')
    free = shutil.disk_usage(path.parent).free
    if free < st.st_size + (1 << 30):
        raise Refused(f'{path}: {free >> 30} GiB free, and the pieces need the record\'s {st.st_size >> 30} beside it')
    for stale in path.parent.glob(path.name + '.*.part'):
        stale.unlink()
    before = allocated([path])
    sha, parts = hashlib.sha256(), []
    with open(path, 'rb') as f:
        while True:
            part = Path(f'{path}.{len(parts)}.part')
            left = piece
            with open(part, 'wb') as out:
                while left and (block := f.read(min(BLOCK, left))):
                    sha.update(block)
                    out.write(block)
                    left -= len(block)
            if left == piece:
                part.unlink()
                break
            parts.append(part)
    compressed = bool(compress and compress(parts))
    # What is on disk now, after the compressor rewrote it, against what
    # was read: the one check between the record and its removal.
    got = digest(parts)
    if got != (st.st_size, sha.hexdigest()) or path.stat().st_size != st.st_size:
        for p in parts:
            p.unlink()
        raise Refused(f'{path}: the pieces read back as {got[0]} bytes {got[1][:12]}…, '
                      f'the record as {st.st_size} bytes {sha.hexdigest()[:12]}…; the record stands whole')
    for k in range(len(parts) - 1, 0, -1):
        os.replace(parts[k], f'{path}.{k}')
    os.replace(parts[0], path)
    return dict(path=str(path), size=st.st_size, sha256=sha.hexdigest(), pieces=len(parts),
                compressed=compressed, before=before, after=allocated(pieces(path)))


def join(path):
    """One file again: the undo. Returns `(bytes, SHA-256)`, or `None` for a record already whole."""
    path = Path(path)
    parts = pieces(path)
    if len(parts) == 1:
        return None
    want = digest(parts)
    free = shutil.disk_usage(path.parent).free
    if free < want[0] + (1 << 30):
        raise Refused(f'{path}: {free >> 30} GiB free, and the whole record is {want[0] >> 30}')
    whole = Path(f'{path}.whole.part')
    with open(whole, 'wb') as out:
        for p in parts:
            with open(p, 'rb') as f:
                shutil.copyfileobj(f, out, BLOCK)
    if digest([whole]) != want:
        whole.unlink()
        raise Refused(f'{path}: the joined file does not read back as its pieces; they stand')
    os.replace(whole, path)
    for p in parts[1:]:
        p.unlink()
    return want


def archive(golden):
    """Every record under `golden/<run>/map-<N>/`, the largest last."""
    return sorted(golden.glob('*/map-*/rontrace.ranges'), key=lambda p: p.stat().st_size)


def gib(n):
    return f'{n / (1 << 30):.2f} GiB'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('records', nargs='*', type=Path)
    ap.add_argument('--archive', action='store_true', help='every record under the golden archive')
    ap.add_argument('--golden', type=Path, default=GOLDEN)
    ap.add_argument('--check', action='store_true', help="print each record's bytes and SHA-256 and change nothing")
    ap.add_argument('--join', action='store_true', help='put each record back in one file')
    ap.add_argument('--min-age', type=int, default=600, metavar='S',
                    help='refuse a record written in the last S seconds (default 600)')
    a = ap.parse_args(argv)
    records = list(a.records) + (archive(a.golden) if a.archive else [])
    if not records:
        ap.error('name a record, or --archive')
    status = 0
    for r in records:
        try:
            if a.check:
                size, sha = digest(pieces(r))
                print(f'{r}: {size} bytes, SHA-256 {sha}, {len(pieces(r))} piece(s), {gib(allocated(pieces(r)))} on disk')
            elif a.join:
                done = join(r)
                print(f'{r}: ' + (f'one file again, {done[0]} bytes, SHA-256 {done[1]}' if done else 'already one file'))
            else:
                done = split(r, min_age=a.min_age)
                if done is None:
                    print(f'{r}: {len(pieces(r))} piece(s), nothing to split')
                else:
                    print(f"{r}: {done['size']} bytes in {done['pieces']} pieces, SHA-256 {done['sha256']}; "
                          f"{gib(done['before'])} on disk before, {gib(done['after'])} after"
                          + ('' if done['compressed'] else '; afsctool is not on this PATH, the pieces are not compressed'))
        except Refused as e:
            print(f'split.py refuses: {e}', file=sys.stderr)
            status = 2
    return status


if __name__ == '__main__':
    sys.exit(main())
