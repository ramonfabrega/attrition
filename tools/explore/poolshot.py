#!/usr/bin/env python3
"""poolshot.py <run> <frame> [--out PATH] [--lane N] — photograph a running
click-free-pool take's game window, and nothing else (Loop 3469, 3451).

    python3 -I tools/explore/poolshot.py recon3460-3 911

The take pauses itself on the frame to be photographed with a console line
in its staging — `911 pause 1` — or draws a still enough frame unpaused
(3463's 9930). This waits for the take's own gamelog to print `BEGIN FRAME
<frame>` (or later), finds the game window, and photographs that window
alone with `screencapture -x -o -l <id>`, run through
`tools/gamelog/viadriver.sh` so it runs under RonDriver.app's Screen
Recording grant. The gamelog is `<run>/gamelog.txt` for a windowed capture and
`<run>/map-<N>/gamelog.txt` for a pool take — the one `map-*`, or the one
`--map N` names; two without `--map` refuse, and no gamelog within
`--appear` seconds refuses (Loop 3487). It writes `<run>-<frame>.png`
beside the run directory (or
`--out`) and prints the window id, its owner pid and the path.

Graduated from three lanes' scratch shooters (items 3444, 3460, 3463), and
built around the traps each one paid for once:

- **Window-only, with no fallback.** A desktop grab caught other apps' private
  windows (3444's first shot). `shoot` refuses a request with no window id,
  and the shim viadriver runs (`poolshot_grab.sh`) refuses it again; when no
  window matches, this exits non-zero rather than guess.
- **The window is found by the run's directory in its owner's command
  line**, not by title or size: another lane's game can be on screen beside
  this one (att-3455's beside 3460's). Wine writes that directory with
  backslashes and a drive letter (`Z:\\Users\\…\\run1090\\…`).
- **The wait keys on the gamelog's frame line, which ends in CR** — a
  `911$` pattern never matched (3460). A paused `cover=0` trace keeps
  growing, so a quiet trace is no signal (3460's first recon); and a
  sparse-detail take buffers its gamelog, so the take's `gamelog.ini` must
  print the frame on the frame (3444's first) — `--timeout` ends the wait.
- **Cropping is the caller's.** `sips --cropOffset` ignores the offset on
  this macOS and crops the centre (3463); read a corner off the full window
  image, or crop with CoreGraphics' `CGImageCreateWithImageInRect`.

`--lane N` narrows the window to that pool lane's game: the pid on the
first line of the lane's `.lane.lock` (lane 1 `~/wine-ron`, lane N
`~/wine-ron-N`) or one of its descendants.

The window lister is `poolshot_windows.swift` (window ids, pids and bounds
need no grant; only the photograph does). Tests:
`tools/explore/test_poolshot.py`, with `screencapture` stubbed.
"""
import argparse
import os
import re
import subprocess
import sys
import time
from collections import namedtuple
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
GRAB = HERE / 'poolshot_grab.sh'
LISTER = HERE / 'poolshot_windows.swift'
VIADRIVER = ROOT / 'tools/gamelog/viadriver.sh'
GOLDEN = Path.home() / 'ron-golden'

Window = namedtuple('Window', 'id pid layer width height owner')


class Refused(Exception):
    """A request this tool will not turn into a capture."""


# --- the window ------------------------------------------------------------

def _norm(path):
    """A path as Wine or a posix argv might spell it, folded to one form:
    forward slashes, no drive letter, lower case (both file systems here
    are case-insensitive)."""
    s = str(path).replace('\\', '/').lower()
    s = re.sub(r'(?<![a-z0-9])[a-z]:/', '/', s)
    return s.rstrip('/')


def cmdline_names_dir(cmdline, run_dir):
    """True when `cmdline` names `run_dir` itself (or a file inside it) —
    never a sibling whose name merely starts the same (`run10900`)."""
    text = _norm(cmdline)
    for d in {_norm(run_dir), _norm(os.path.realpath(run_dir))}:
        for m in re.finditer(re.escape(d), text):
            end = m.end()
            if end == len(text) or text[end] in '/ "\'':
                return True
    return False


def pick_window(windows, cmdlines, run_dir, pids=None):
    """The game window of the process whose command line names `run_dir`:
    the largest layer-0 window those pids own. None when no process names
    the run — the caller refuses rather than guess."""
    owners = {pid for pid, cmd in cmdlines.items() if cmdline_names_dir(cmd, run_dir)}
    if pids is not None:
        owners &= set(pids)
    found = [w for w in windows if w.pid in owners and w.layer == 0 and w.width > 1 and w.height > 1]
    if not found:
        return None
    return max(found, key=lambda w: (w.width * w.height, -w.id))


def parse_windows(text):
    out = []
    for line in text.splitlines():
        parts = line.split('\t', 5)
        if len(parts) < 5:
            continue
        try:
            n, pid, layer, w, h = (int(x) for x in parts[:5])
        except ValueError:
            continue
        out.append(Window(n, pid, layer, w, h, parts[5] if len(parts) > 5 else ''))
    return out


def list_windows():
    done = subprocess.run(['swift', str(LISTER)], capture_output=True, text=True, timeout=120)
    if done.returncode != 0:
        raise SystemExit(f'poolshot: the window lister failed: {done.stderr.strip()}')
    return parse_windows(done.stdout)


def process_table():
    """{pid: command line} and {pid: parent pid} for every process."""
    done = subprocess.run(['ps', '-axww', '-o', 'pid=,ppid=,command='],
                          capture_output=True, text=True, check=True)
    cmd, parent = {}, {}
    for line in done.stdout.splitlines():
        parts = line.strip().split(None, 2)
        if len(parts) < 2:
            continue
        pid, ppid = int(parts[0]), int(parts[1])
        cmd[pid] = parts[2] if len(parts) > 2 else ''
        parent[pid] = ppid
    return cmd, parent


def descendants(root, parent):
    """`root` and every pid whose parent chain reaches it."""
    out = {root}
    for pid in parent:
        seen, p = set(), pid
        while p and p not in seen:
            if p == root:
                out.add(pid)
                break
            seen.add(p)
            p = parent.get(p)
    return out


def lane_pid(lane):
    prefix = Path.home() / ('wine-ron' if lane == 1 else f'wine-ron-{lane}')
    lock = Path(os.environ.get('RON_LANE_LOCK', prefix / '.lane.lock'))
    try:
        first = lock.read_text().splitlines()[0].strip()
    except (OSError, IndexError):
        raise SystemExit(f'poolshot: lane {lane} holds no lock ({lock}) — no game is up on it')
    if not first.isdigit():
        raise SystemExit(f'poolshot: lane {lane}\'s lock names no pid ({lock})')
    return int(first)


# --- the frame -------------------------------------------------------------

FRAME = re.compile(rb'BEGIN FRAME (\d+)[ \t]*$')


class FrameWatch:
    """Reads a growing gamelog in chunks. A line counts only once its end —
    CR, LF or CRLF — has been written: the gamelog's lines end in CR (3460),
    and a half-written `BEGIN FRAME 91` is not frame 91."""

    def __init__(self):
        self.rest = b''
        self.last = None

    def feed(self, chunk):
        data = self.rest + chunk
        lines = re.split(rb'\r\n|\r|\n', data)
        self.rest = lines.pop()
        seen = None
        for line in lines:
            m = FRAME.search(line)
            if m:
                seen = int(m.group(1))
                self.last = seen if self.last is None else max(self.last, seen)
        return seen

    def reached(self, frame):
        return self.last is not None and self.last >= frame


LOGS = ('gamelog.txt', 'gamelog_p.txt')


def map_dirs(run_dir, style=None):
    """The take's `map-<N>` directories under `run_dir` — the one `--map`
    names, or every `map-*` — mirroring `tools/gamelog/filerun.py`'s
    `map_dir` (Loop 3487)."""
    if style is not None:
        d = run_dir / f'map-{style}'
        if not d.is_dir():
            raise Refused(f'no {d}')
        return [d]
    return sorted(p for p in run_dir.glob('map-*') if p.is_dir())


def gamelog_of(run_dir, style=None):
    """The take's gamelog, or None while the game has not written one yet.

    A bare `<run>/gamelog.txt` (or a playback's `gamelog_p.txt`) is a
    windowed capture's and is read first; a click-free pool take writes
    `<run>/map-<N>/gamelog.txt`, one `map-*` per take (Loop 3487: a wait
    on the bare path sat 30 minutes and lost three photographs). Two
    `map-*` directories without `--map` refuse, as filerun.py's do."""
    if style is None:
        for name in LOGS:
            if (run_dir / name).exists():
                return run_dir / name
    maps = map_dirs(run_dir, style)
    if len(maps) > 1:
        raise Refused(f'{run_dir} holds {len(maps)} map directories '
                      f'({", ".join(p.name for p in maps)}); name one with --map')
    for d in maps:
        for name in LOGS:
            if (d / name).exists():
                return d / name
    return None


def wait_frame(run_dir, frame, timeout, poll=1.0, style=None, appear=120.0):
    """Wait for `BEGIN FRAME <frame>` (or later) in the take's gamelog.
    Refused when no gamelog has appeared after `appear` seconds — the
    take's directory is wrong, not slow — rather than after `timeout`."""
    watch, offset, gamelog = FrameWatch(), 0, None
    start = time.monotonic()
    deadline = start + timeout
    while time.monotonic() < deadline:
        gamelog = gamelog or gamelog_of(run_dir, style)
        if gamelog is None and time.monotonic() - start >= appear:
            maps = map_dirs(run_dir, style)
            where = (f'{maps[0]} holds no gamelog.txt' if maps else
                     f'{run_dir} holds no map-* directory and no gamelog.txt')
            raise Refused(f'no gamelog after {appear:g} s: {where} (a click-free pool take '
                          f'writes <run>/map-<N>/gamelog.txt; Loop 3487)')
        if gamelog is not None:
            with gamelog.open('rb') as f:
                f.seek(offset)
                chunk = f.read()
            offset += len(chunk)
            watch.feed(chunk)
            if watch.reached(frame):
                return watch.last
        time.sleep(poll)
    where = gamelog or run_dir / 'gamelog.txt'
    raise SystemExit(f'poolshot: no BEGIN FRAME {frame} in {where} after {timeout} s '
                     f'(last {watch.last}); a sparse-detail gamelog buffers its frames (3444)')


# --- the photograph --------------------------------------------------------

def _window_id(window_id):
    if isinstance(window_id, bool) or not isinstance(window_id, int) or window_id <= 0:
        raise Refused(f'no window id ({window_id!r}): poolshot photographs one window or nothing, '
                      'never the desktop')
    return window_id


def grab_argv(window_id, out):
    return ['zsh', str(VIADRIVER), str(GRAB), str(_window_id(window_id)), str(out)]


def shoot(window_id, out, runner=None):
    """Hand the one `screencapture -l` to RonDriver. Refuses before the
    runner is called when there is no window to name."""
    argv = grab_argv(window_id, out)
    if runner is None:
        return subprocess.run(argv, capture_output=True, text=True, check=True).stdout
    return runner(argv)


def wait_png(out, timeout, launched):
    log = None
    m = re.search(r'^log: (.+)$', launched or '', re.M)
    if m:
        log = Path(m.group(1).strip())
    deadline = time.monotonic() + timeout
    last = -1
    while time.monotonic() < deadline:
        if out.exists():
            size = out.stat().st_size
            if size > 0 and size == last:
                with out.open('rb') as f:
                    if f.read(8) != b'\x89PNG\r\n\x1a\n':
                        raise SystemExit(f'poolshot: {out} is not a PNG')
                return
            last = size
        if log and log.exists() and 'could not create image' in log.read_text(errors='replace'):
            raise SystemExit(f'poolshot: screencapture refused ({log}): RonDriver.app has lost its '
                             'Screen Recording grant — a human re-grants it (docs/ORACLE.md)')
        time.sleep(0.5)
    raise SystemExit(f'poolshot: no image at {out} after {timeout} s; read {log or "the viadriver log"}')


def resolve_run(arg):
    p = Path(arg).expanduser()
    if not p.is_dir():
        p = GOLDEN / arg
    if not p.is_dir():
        raise SystemExit(f'poolshot: no run directory {arg} (nor {GOLDEN / arg})')
    return p.resolve()


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    ap.add_argument('run', help='the take\'s output directory, or its name under ~/ron-golden')
    ap.add_argument('frame', type=int, help='the frame to photograph (the take\'s `pause 1` frame)')
    ap.add_argument('--out', type=Path, help='the PNG to write (default <run>-<frame>.png beside the run)')
    ap.add_argument('--lane', type=int, help='only the window of this pool lane\'s game')
    ap.add_argument('--timeout', type=float, default=1800, help='seconds to wait for the frame')
    ap.add_argument('--settle', type=float, default=2.0, help='seconds after the frame before the shot')
    ap.add_argument('--map', type=int, metavar='N', help='the map-N directory when the run holds several')
    ap.add_argument('--appear', type=float, default=120.0,
                    help='seconds for the take\'s gamelog to appear before refusing')
    args = ap.parse_args(argv)

    run_dir = resolve_run(args.run)
    out = (args.out or run_dir.parent / f'{run_dir.name}-{args.frame}.png').expanduser().resolve()
    if out.exists():
        raise SystemExit(f'poolshot: {out} exists; name another with --out (nothing under the archive is deleted)')
    try:
        seen = wait_frame(run_dir, args.frame, args.timeout, style=args.map, appear=args.appear)
    except Refused as e:
        raise SystemExit(f'poolshot refuses: {e}')
    time.sleep(args.settle)
    cmdlines, parent = process_table()
    pids = descendants(lane_pid(args.lane), parent) if args.lane else None
    window = pick_window(list_windows(), cmdlines, run_dir, pids)
    if window is None:
        raise SystemExit(f'poolshot: no on-screen window belongs to a process naming {run_dir}'
                         f'{f" on lane {args.lane}" if args.lane else ""}; refusing to guess')
    launched = shoot(window.id, out)
    wait_png(out, 60, launched)
    print(f'window {window.id} pid {window.pid} {window.width}x{window.height} '
          f'frame {seen} -> {out}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
