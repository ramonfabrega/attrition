#!/usr/bin/env python3
"""lanes.py — what the click-free pool did over a tranche: the number a pass
moves `RON_LANES_MAX` on (item 1569).

    python3 tools/gamelog/lanes.py --since <commit|YYYY-MM-DD[THH:MM]>

Reads every lane's `.lane.log` (beside its `.lane.lock`, written by
`winelaunch.sh`'s `ron_lane_take`/`ron_lane_release` and by the runner) and
prints:

  * **peak** — the most pool lanes held at once, and when;
  * **waited** — minutes the pool waited at its cap, and how many times;
  * **rate** — each capture's frames per wall second, launch to exit, from
    the runner's `capture` line (the receipt's `launched_at`, `exited_at`
    and frame count), and the tranche's median.

A cap is raised only when the waits are high and the rate held, and
lowered when the rate fell: lanes share one box, and a starved game is the
failure a slow box produces — the stall relaunch trips on wall-clock
timeouts before anything reads as wrong.

A line is `epoch instant verb lane pid rest…`; verbs `take` (rest: `pool`
or `queue`), `release`, `waited` (lane 0, the pool's, in lane 1's log;
rest: seconds) and `capture` (rest:
`frames=`, `launched=`, `exited=`, `success=`, `out=`). A take on a lane
closes that lane's open interval — a runner that died never released.
"""
import argparse
import datetime as dt
import glob
import os
import statistics
import subprocess
import sys
from pathlib import Path


def stamp(text):
    return dt.datetime.strptime(text, '%Y-%m-%dT%H:%M:%S%z').timestamp()


def since_epoch(value, cwd=None):
    """A git revision's commit time, or an ISO date or instant (local)."""
    rev = subprocess.run(['git', 'log', '-1', '--format=%ct', value], capture_output=True,
                         text=True, cwd=cwd)
    if rev.returncode == 0 and rev.stdout.strip():
        return int(rev.stdout.strip())
    for fmt in ('%Y-%m-%dT%H:%M:%S%z', '%Y-%m-%dT%H:%M', '%Y-%m-%d'):
        try:
            return dt.datetime.strptime(value, fmt).timestamp()
        except ValueError:
            pass
    raise ValueError(f'--since takes a git revision or a date: {value}')


def read(paths, since=0):
    rows = []
    for path in paths:
        for line in Path(path).read_text(errors='replace').splitlines():
            parts = line.split()
            if len(parts) < 5 or not parts[0].isdigit():
                continue
            epoch = int(parts[0])
            if epoch < since:
                continue
            rows.append({'t': epoch, 'verb': parts[2], 'lane': parts[3], 'pid': parts[4],
                         'rest': parts[5:]})
    return sorted(rows, key=lambda r: r['t'])


def summarize(rows):
    open_ = {}
    held, peak, peak_at = 0, 0, None
    waits = []
    rates = []
    for r in rows:
        if r['verb'] == 'take' and r['rest'][:1] == ['pool']:
            if r['lane'] in open_:
                held -= 1
            open_[r['lane']] = r['pid']
            held += 1
            if held > peak:
                peak, peak_at = held, r['t']
        elif r['verb'] == 'release' and open_.get(r['lane']) == r['pid']:
            del open_[r['lane']]
            held -= 1
        elif r['verb'] == 'waited' and r['rest']:
            waits.append(float(r['rest'][0]))
        elif r['verb'] == 'capture':
            kv = dict(x.split('=', 1) for x in r['rest'] if '=' in x)
            try:
                seconds = stamp(kv['exited']) - stamp(kv['launched'])
                frames = int(kv['frames'])
            except (KeyError, ValueError):
                continue
            if seconds > 0 and frames > 0:
                rates.append({'lane': r['lane'], 'frames': frames, 'seconds': seconds,
                              'rate': frames / seconds, 'out': kv.get('out', '?'),
                              'success': kv.get('success')})
    return {'peak': peak, 'peak_at': peak_at, 'waits': waits, 'rates': rates,
            'median_rate': statistics.median([x['rate'] for x in rates]) if rates else None}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--since', default='1970-01-01', help='a git revision or a date')
    ap.add_argument('--root', default=os.environ.get('RON_LANES_ROOT', str(Path.home())),
                    help='where the lanes live (RON_LANES_ROOT)')
    args = ap.parse_args(argv)
    since = since_epoch(args.since)
    logs = sorted(glob.glob(f'{args.root}/wine-ron/.lane.log') + glob.glob(f'{args.root}/wine-ron-*/.lane.log'))
    s = summarize(read(logs, since))
    when = dt.datetime.fromtimestamp(s['peak_at']).isoformat(timespec='minutes') if s['peak_at'] else '-'
    print(f"lane logs: {len(logs)}; since {dt.datetime.fromtimestamp(since).isoformat(timespec='minutes')}")
    print(f"peak: {s['peak']} pool lanes held at once (first at {when})")
    print(f"waited at the cap: {sum(s['waits']) / 60:.1f} minutes over {len(s['waits'])} waits")
    for x in s['rates']:
        print(f"  lane {x['lane']}: {x['frames']} frames in {x['seconds']:.0f} s = {x['rate']:.1f} frames/s"
              f"{'' if x['success'] == 'true' else ' (not a success)'}  {x['out']}")
    print('rate: ' + (f"median {s['median_rate']:.1f} frames per wall second over {len(s['rates'])} captures"
                      if s['rates'] else 'no capture line'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
