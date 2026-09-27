#!/usr/bin/env python3
"""stamp.py — which `rontrace.dll` is in the install, and is it this tree's?

    stamp.py write <install> [defs]      build.sh's last act
    stamp.py check <install> [--issuer]  longtrace.sh's, before the launch

The queue lane runs whatever `build.sh` last wrote into the install, from
whichever branch last ran it (parked 936). `write` leaves
`<install>/rontrace.dll.stamp` beside the DLL:

    built   2026-09-27T09:23:11Z
    source  <sha256 of tools/trace/tracer.c as built>
    defs    <TRACER_DEFS, or ->
    dll     <sha256 of rontrace.dll>

`check` prints the stamp and three verdicts, so the capture's log is the
receipt: the DLL is the one the stamp describes (or it was copied in by
hand — refused); the stamp's source is this tree's `tracer.c` (or the build
is another branch's — said, and refused for a stanza with `@` lines, whose
verbs are the thing that differs); and a DLL with no stamp at all is named
as one, and refused for an `@` stanza. The remedy is one line and the
refusal prints it.

The source is compared by content, never by commit: a worktree's build is
often of an uncommitted `tracer.c`, and two branches with the same file are
the same build.
"""
import hashlib
import sys
import time
from pathlib import Path

SOURCE = Path(__file__).resolve().parent / 'tracer.c'
REMEDY = 'rebuild it from this tree: zsh tools/trace/build.sh <install>'


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(install, source=SOURCE, defs=''):
    install = Path(install)
    lines = [
        'built   ' + time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'source  ' + sha256(source),
        'defs    ' + (defs.strip() or '-'),
        'dll     ' + sha256(install / 'rontrace.dll'),
    ]
    (install / 'rontrace.dll.stamp').write_text('\n'.join(lines) + '\n')
    return lines


def read(install):
    path = Path(install) / 'rontrace.dll.stamp'
    if not path.is_file():
        return None
    return dict(line.split(None, 1) for line in path.read_text().splitlines() if line.strip())


def check(install, source=SOURCE, issuer=False):
    """(ok, lines): whether the capture may launch, and what its log says."""
    install = Path(install)
    dll = install / 'rontrace.dll'
    if not dll.is_file():
        return False, [f'tracer: no rontrace.dll in {install} — {REMEDY}']
    actual = sha256(dll)
    lines = [f'tracer: rontrace.dll sha256 {actual}']
    stamp = read(install)
    if stamp is None:
        lines.append('tracer: no stamp — a build from before the stamp, or a copy; '
                     'what it was built from is not recorded')
        if issuer:
            lines.append(f'tracer: REFUSED — the stanza has `@` lines and the build is unknown; {REMEDY}')
        return not issuer, lines
    lines.append('tracer: built {built}, defs {defs}, source {source}'.format(
        built=stamp.get('built', '?'), defs=stamp.get('defs', '?').strip(),
        source=stamp.get('source', '?').strip()))
    if stamp.get('dll', '').strip() != actual:
        lines.append(f'tracer: REFUSED — the DLL is not the build its stamp describes; {REMEDY}')
        return False, lines
    if stamp.get('source', '').strip() != sha256(source):
        lines.append(f"tracer: the build's tracer.c differs from this tree's ({sha256(source)})")
        if issuer:
            lines.append(f'tracer: REFUSED — the stanza has `@` lines, and the verbs are what differs; {REMEDY}')
            return False, lines
    return True, lines


def main(argv):
    if len(argv) >= 3 and argv[1] == 'write':
        print('\n'.join(write(argv[2], defs=' '.join(argv[3:]))))
        return 0
    if len(argv) >= 3 and argv[1] == 'check':
        ok, lines = check(argv[2], issuer='--issuer' in argv[3:])
        print('\n'.join(lines))
        return 0 if ok else 1
    sys.exit(__doc__)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
