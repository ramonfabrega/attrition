#!/usr/bin/env python3
"""photo_srgb.py <png> [--out PATH] — a photograph's pixels in sRGB (item 4223).

    python3 -I tools/render/photo_srgb.py ~/ron-data/lab-experiments/2026-11-12-item-4160/run1564-3294.png \
        --out ~/ron-data/lab-experiments/2026-11-13-item-4223/run1564-3294-srgb.png

`screencapture -l` writes the window as the display shows it, tagged with
that display's ICC profile. On an sRGB display the bytes are the game's. On
another, the window server has already matched the game's sRGB to the
display: run1564's photograph is tagged `Mi monitor` (a near-P3 gamut at
gamma 1.96), and its lake's saturated cyan reads red 47 where the frame
held about 20 (`docs/RENDERER.md` §3.18, item 4223). Every other photograph
on disk is tagged `sRGB IEC61966-2.1`.

This reads the PNG's `iCCP` profile name. Tagged sRGB, it prints `srgb` and
writes nothing. Otherwise it matches the image to the system's sRGB profile
with `sips -m` (ColorSync) into `--out` (default `<stem>-srgb.png` beside
it), and prints the profile it came from. It never overwrites a file.
Cropping the title bar and the TGA the tests read stay the caller's
(`tools/explore/poolshot.py`'s note on `sips --cropOffset`).

Stdlib only; macOS for `sips`.
"""
import argparse
import subprocess
import sys
import zlib
from pathlib import Path

SRGB = 'sRGB IEC61966-2.1'
SRGB_ICC = Path('/System/Library/ColorSync/Profiles/sRGB Profile.icc')


def chunks(data):
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError('not a PNG')
    at = 8
    while at + 8 <= len(data):
        n = int.from_bytes(data[at:at + 4], 'big')
        kind = data[at + 4:at + 8]
        yield kind, data[at + 8:at + 8 + n]
        at += 12 + n
        if kind == b'IEND':
            return


def profile_name(path):
    """The `iCCP` chunk's profile, by its `desc` tag when the profile
    carries one (that is the name `sips -g profile` prints), else the
    chunk's own keyword; `None` with no `iCCP` (`sRGB` chunk or untagged)."""
    for kind, body in chunks(Path(path).read_bytes()):
        if kind == b'sRGB':
            return SRGB
        if kind == b'iCCP':
            keyword, rest = body.split(b'\0', 1)
            icc = zlib.decompress(rest[1:])
            desc = icc_desc(icc)
            return desc if desc is not None else keyword.decode('latin-1')
        if kind == b'IDAT':
            return None
    return None


def icc_desc(icc):
    count = int.from_bytes(icc[128:132], 'big')
    for i in range(count):
        e = icc[132 + 12 * i:144 + 12 * i]
        if e[:4] != b'desc':
            continue
        off, size = int.from_bytes(e[4:8], 'big'), int.from_bytes(e[8:12], 'big')
        t = icc[off:off + size]
        if t[:4] == b'mluc':
            rec = t[16:28]
            ln, at = int.from_bytes(rec[4:8], 'big'), int.from_bytes(rec[8:12], 'big')
            return t[at:at + ln].decode('utf-16-be')
        if t[:4] == b'desc':
            ln = int.from_bytes(t[8:12], 'big')
            return t[12:12 + ln].rstrip(b'\0').decode('latin-1')
    return None


def to_srgb(png, out=None, runner=subprocess.run):
    """`(profile, out)`: `out` is `None` when the photograph is already sRGB."""
    png = Path(png)
    name = profile_name(png)
    if name == SRGB:
        return name, None
    out = Path(out) if out else png.with_name(f'{png.stem}-srgb.png')
    if out.exists():
        raise SystemExit(f'photo_srgb: {out} exists; nothing is overwritten')
    runner(['sips', '-m', str(SRGB_ICC), str(png), '--out', str(out)],
           check=True, capture_output=True)
    if profile_name(out) != SRGB:
        raise SystemExit(f'photo_srgb: {out} is not tagged {SRGB} after sips -m')
    return name, out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    ap.add_argument('png')
    ap.add_argument('--out', type=Path)
    args = ap.parse_args(argv)
    name, out = to_srgb(args.png, args.out)
    if out is None:
        print(f'srgb {args.png}')
    else:
        print(f'matched {args.png} from {name!r} to {SRGB!r} -> {out}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
